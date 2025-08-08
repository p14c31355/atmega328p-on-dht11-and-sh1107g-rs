#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use panic_halt as _;

use dvcdbg::logger::{Logger, log_bytes};
use heapless::String;
use core::fmt::Write;

pub struct SerialLogger<TX> {
    tx: TX,
}

impl<TX> SerialLogger<TX>
where
    TX: embedded_hal::serial::Write<u8>,
{
    pub fn new(tx: TX) -> Self {
        Self { tx }
    }
}

impl<TX> Logger for SerialLogger<TX>
where
    TX: embedded_hal::serial::Write<u8>,
{
    fn log(&mut self, msg: &str) {
        for &b in msg.as_bytes() {
            nb::block!(self.tx.write(b)).ok();
        }
        nb::block!(self.tx.write(b'\n')).ok();
    }

    fn log_bytes(&mut self, label: &str, bytes: &[u8]) {
        let mut out: String<128> = String::new();
        if write!(&mut out, "{}: ", label).is_ok() {
            for &b in bytes {
                if write!(&mut out, "0x{:02X} ", b).is_err() {
                    let _ = out.push_str("...");
                    break;
                }
            }
        } else {
            let _ = out.push_str("...");
        }
        self.log(&out);
    }
}

pub struct LoggingI2c<I2C, L> {
    i2c: I2C,
    logger: L,
}

impl<I2C, L> LoggingI2c<I2C, L> {
    pub fn new(i2c: I2C, logger: L) -> Self {
        Self { i2c, logger }
    }
}

impl<I2C, L, E> embedded_hal::blocking::i2c::Write for LoggingI2c<I2C, L>
where
    I2C: embedded_hal::blocking::i2c::Write<Error = E>,
    L: Logger,
{
    type Error = E;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.logger.log_bytes("I2C Write", bytes);
        self.i2c.write(addr, bytes)
    }
}

use sh1107::{Builder, DisplayRotation};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART 初期化 (ロガー用)
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let logger = SerialLogger::new(serial);

    // I2C 初期化 (SCL=A5, SDA=A4)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a5.into_pull_up_input(),
        pins.a4.into_pull_up_input(),
        400_000,
