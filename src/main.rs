#![no_std]
#![no_main]

use arduino_hal::hal::usart::Usart;
use arduino_hal::prelude::*;
use embedded_hal::i2c::I2c;
use embedded_io::Write as EmbeddedWrite;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use panic_halt as _;

use dvcdbg::logger::Logger;
use dvcdbg::log;

// ==== Logger 実装 (USART0 → dvcdbg) ====
struct UsartLogger<W> {
    inner: W,
}

impl<W> Logger for UsartLogger<W>
where
    W: arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
{
    fn log(&mut self, msg: &str) {
        let _ = self.inner.write_str(msg);
        let _ = self.inner.write_str("\r\n");
    }

    fn log_fmt(&mut self, args: core::fmt::Arguments) {
        use core::fmt::Write;
        let _ = self.inner.write_fmt(args);
        let _ = self.inner.write_str("\r\n");
    }
}

// `embedded-io::Write` 実装
impl<W> EmbeddedWrite for UsartLogger<W>
where
    W: arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
{
    type Error = core::convert::Infallible;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for &b in buf {
            nb::block!(self.inner.write(b)).unwrap();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ==== メイン処理 ====
#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UsartLogger { inner: serial };

    log!(logger, "=== Arduino Uno Logger/I2C Display Stub Start ===");

    // I²C 初期化
    let mut i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // I²Cスキャン（全アドレス確認）
    for addr in 0x03..=0x77 {
        match i2c.write(addr, &[]) {
            Ok(_) => log!(logger, "✅ Found device at 0x{:02X}", addr),
            Err(_) => {}
        }
    }

    // SH1107G 初期化コマンド（例）
    let init_cmds: [u8; 3] = [0xAE, 0xA4, 0xAF];
    logger.log_bytes("I2C CMD", &init_cmds);
    let _ = i2c.write(0x3C, &init_cmds);

    // 真っ白フレームバッファ作成
    const WIDTH: usize = 128;
    const HEIGHT: usize = 64;
    const BUFFER_SIZE: usize = WIDTH * HEIGHT / 8;
    let framebuffer = [0xFFu8; BUFFER_SIZE]; // 全ピクセルON
    logger.log_bytes("FB", &framebuffer[..16]); // 先頭16バイトだけログ

    // embedded-graphicsで全画面塗りつぶし矩形
    let style = PrimitiveStyle::with_fill(BinaryColor::On);
    let display_area = Rectangle::new(Point::new(0, 0), Size::new(WIDTH as u32, HEIGHT as u32));
    // display_area.draw(&mut display)  // 実際のDisplayドライバがあればここで描画

    log!(logger, "🖥️ Screen filled with white pixels");

    loop {}
}
