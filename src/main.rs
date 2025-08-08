#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::I2c;
use panic_halt as _;

use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
// 修正版 import
use embedded_hal::i2c::I2c; // blockingは不要
use embedded_io::Write as EmbeddedWrite; // serialはembedded-ioで代替
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

use nb::block;

use dvcdbg::logger::{Logger, SerialLogger};

/// UARTをembedded-io::Writeに変換するラッパー
pub struct UsartEmbeddedIo<W>(pub W);

impl<W> EmbeddedWrite for UsartEmbeddedIo<W>
where
    W: HalWrite<u8>,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, core::convert::Infallible> {
        for &b in buf {
            block!(self.0.write(b)).unwrap();
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), core::convert::Infallible> {
        Ok(())
    }
}


/// FmtWrite対応のラッパー
pub struct FmtWriteWrapper<'a, W: EmbeddedWrite> {
    inner: &'a mut W,
}

impl<'a, W: EmbeddedWrite> FmtWriteWrapper<'a, W> {
    pub fn new(inner: &'a mut W) -> Self {
        Self { inner }
    }
}

impl<'a, W: EmbeddedWrite> core::fmt::Write for FmtWriteWrapper<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.inner.write(s.as_bytes()).map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}

/// OLED I2Cアドレス（例: SH1107G）
const OLED_ADDR: u8 = 0x3C;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut usart_io = UsartEmbeddedIo(serial);
    let mut fmt_wrapper = FmtWriteWrapper::new(&mut usart_io);
    let mut logger = SerialLogger::new(&mut fmt_wrapper);

    Logger::log_fmt(&mut logger, format_args!("🔌 Logger initialized\n"));

    // I²C初期化
    let mut i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );

    // 画面を真っ白にするためのフレームバッファ生成
    // SH1107Gは128x128までだけど、ここでは128x64で例示
    const WIDTH: usize = 128;
    const HEIGHT: usize = 64;
    let buffer_size = WIDTH * HEIGHT / 8;
    let mut framebuffer = [0u8; 1024]; // 固定2048バイト例
    for byte in framebuffer.iter_mut() {
        *byte = 0xFF;
    }

    // ログにI2C送信バイト列を出す
    logger.log_bytes("FRAMEBUFFER", &framebuffer);

    // 実際にOLEDに送信（ページモードの例）
    // コマンド送信例: Page Addressing Mode
    let _ = i2c.write(OLED_ADDR, &[0x20, 0x02]);
    logger.log_bytes("CMD", &[0x20, 0x02]);

    // データ送信（0x40はGDDRAMデータの制御バイト）
    let mut data_packet = [0u8; 17];
    data_packet[0] = 0x40; // Co = 0, D/C# = 1
    for chunk in framebuffer.chunks(16) {
        data_packet[1..].copy_from_slice(chunk);
        let _ = i2c.write(OLED_ADDR, &data_packet);
        logger.log_bytes("DATA", &data_packet);
    }

    Logger::log_fmt(&mut logger, format_args!("✅ White screen draw complete\n"));

    loop {}
}
