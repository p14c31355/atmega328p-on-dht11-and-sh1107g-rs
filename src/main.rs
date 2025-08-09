#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use core::fmt::Write;
use dvcdbg::log;
use dvcdbg::logger::{Logger, SerialLogger};
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;
use panic_halt as _;

use sh1107g_rs::Sh1107g;
use sh1107g_rs::Sh1107gBuilder;
use embedded_graphics::pixelcolor::BinaryColor; // BinaryColor をインポート

struct SerialWriter<'a, W: EmbeddedHalSerialWrite<u8>> {
    writer: &'a mut W,
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> SerialWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> Write for SerialWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            nb::block!(self.writer.write(byte)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    // シリアルとロガー初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    // I2C初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        50000,                      // 50kHz
    );

    log!(&mut logger, "I2Cスキャン開始");

    // スキャン
    let mut found_sh1107g_addr: Option<u8> = None;
    for addr in 0x03..=0x77 {
        if i2c.write(addr, &[]).is_ok() {
            log!(&mut logger, "Found device at 0x{:02X}", addr);
            if addr == 0x3C || addr == 0x3D {
                found_sh1107g_addr = Some(addr);
                break; // SH1107Gが見つかったらスキャンを終了
            }
        }
    }

    if let Some(addr) = found_sh1107g_addr {
        log!(&mut logger, "SH1107G 初期化開始 (アドレス: 0x{:02X})", addr);

        let mut display = Sh1107gBuilder::new(i2c, &mut logger)
            .with_address(addr)
            .build()
            .unwrap(); // build() の結果を unwrap()

        display.with_logger(|logger_ref| log!(logger_ref, "Display build() 成功"));

        display.init().unwrap(); // init() を呼び出す
        display.clear(BinaryColor::Off).unwrap(); // 初期化時に画面をクリア

        display.with_logger(|logger_ref| log!(logger_ref, "Display initialized and cleared."));

        // 画面を真っ白に塗りつぶす
        display.clear(BinaryColor::On).unwrap();
        display.flush().unwrap();

        display.with_logger(|logger_ref| log!(logger_ref, "Display filled with white."));
    } else {
        log!(&mut logger, "SH1107G ディスプレイが見つかりませんでした。");
    }

    loop {}
}
