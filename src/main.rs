#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use panic_halt as _;
use sh1107g_rs::{Sh1107g, DefaultLogger};
use dvcdbg::logger::{Logger, SerialLogger};
use dvcdbg::log;
use core::fmt::Write;
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;
use dvcdbg::scanner::{scan_i2c, scan_i2c_with_ctrl, scan_init_sequence};

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

    let mut i2c = arduino_hal::I2c::new( // i2c を可変にする
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    // I2Cバスのスキャンを実行
    scan_i2c(&mut i2c, &mut logger);
    scan_init_sequence(&mut i2c, &mut logger, &[
    0xAD, 0x8B, 0xA8, 0x7F, 0xDC, 0xD5, 0x11, 0xC0, 0xDA, 0x12,
    0x81, 0x2F, 0xD9, 0x22, 0xDB, 0x35, 0xA1, 0xA4, 0xA6,]);
    
    let mut display: Sh1107g<
        arduino_hal::I2c,
        _,
    > = Sh1107g::new(i2c, 0x3C, Some(&mut logger));

    // log! マクロの呼び出しを display.with_logger でラップ
    display.with_logger(|logger_ref| log!(logger_ref, "Initializing..."));

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    display.with_logger(|logger_ref| log!(logger_ref, "Display initialized and cleared."));

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    display.with_logger(|logger_ref| log!(logger_ref, "Display filled with white."));

    loop {
        // 無限ループ
    }
}
