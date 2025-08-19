#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use arduino_hal::pins;
use panic_halt as _;
use dvcdbg::scanner::{scan_i2c, I2C_SCAN_ADDR_START, I2C_SCAN_ADDR_END};
use dvcdbg::logger::Logger;
use core::fmt::{Write, Result};

// ===== SerialLogger wrapper =====
struct FmtSerial<'a, U>(&'a mut U);

impl<'a, U> Write for FmtSerial<'a, U>
where
    U: arduino_hal::prelude::_embedded_hal_serial_Write<u8>,
{
    fn write_str(&mut self, s: &str) -> Result {
        for &b in s.as_bytes() {
            nb::block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

// Implement Logger trait for our wrapper
impl<'a, U> Logger for FmtSerial<'a, U>
where
    U: arduino_hal::prelude::_embedded_hal_serial_Write<u8>,
{
    fn log(&mut self, args: core::fmt::Arguments<'_>) {
        let _ = self.write_fmt(args);
        let _ = self.write_str("\r\n");
    }
}

// ===== Main entry =====
#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);

    // Serial at 57600 bps
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = FmtSerial(&mut serial);

    // I2C initialization
    let mut i2c = arduino_hal::I2c::new(dp.TWI, pins.a4, pins.a5, 100_000);

    log!(logger, "[info] Starting I2C scan from 0x{:02X} to 0x{:02X}", I2C_SCAN_ADDR_START, I2C_SCAN_ADDR_END);

    // Scan I2C bus
    scan_i2c(&mut i2c, &mut logger);

    loop {}
}
