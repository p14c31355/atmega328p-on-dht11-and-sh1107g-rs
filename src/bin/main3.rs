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
use dvcdbg::logger::{Logger, SerialLogger};
use dvcdbg::log;
use core::fmt::Write;
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;

// arduino_hal::DefaultSerial を core::fmt::Write に適合させるラッパー
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
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let mut display: Sh1107g<
        arduino_hal::I2c,
        _,
    > = Sh1107g::new(i2c, 0x3C, Some(&mut logger));

    // log! マクロの呼び出しを display.with_logger でラップ
    log!(logger_ref, "Initializing...");

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    log!(logger_ref, "Display initialized and cleared.");

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    log!(logger_ref, "Display filled with white.");

    loop {
        // 無限ループ
    }
}
