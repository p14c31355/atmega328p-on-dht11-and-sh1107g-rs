#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use panic_halt as _;
use sh1107g_rs::Sh1107g;
use dvcdbg::logger::{Logger, SerialLogger, NoopLogger}; // Logger, SerialLogger, NoopLogger をインポート
use dvcdbg::log; // log! マクロをインポート
use core::fmt::Write; // core::fmt::Write トレイトをインポート

// arduino_hal::DefaultSerial を core::fmt::Write に適合させるラッパー
struct SerialWriter<'a, W: embedded_hal::serial::Write<u8>> {
    writer: &'a mut W,
}

impl<'a, W: embedded_hal::serial::Write<u8>> SerialWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W: embedded_hal::serial::Write<u8>> Write for SerialWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            nb::block!(self.writer.write(byte)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    log!(&mut logger, "Initializing...");

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4,
        pins.a5,
        50000,
    );

    let mut display: Sh1107g<
        arduino_hal::I2c, // I2cInterface を削除し、arduino_hal::I2c を直接指定
        NoopLogger, // ロガーの型を NoopLogger に指定
    > = Sh1107g::new(i2c, 0x3C, Some(&mut logger)); // I2cInterface::new(i2c) を i2c に変更

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    log!(&mut logger, "Display initialized and cleared.");

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    log!(&mut logger, "Display filled with white.");

    loop {
        // 無限ループ
    }
}
