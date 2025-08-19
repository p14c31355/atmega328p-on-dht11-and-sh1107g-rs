#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;

use dvcdbg::logger::SerialLogger;
use dvcdbg::scanner::scan_i2c;
use core::fmt::{Write, Result};
use core::fmt::{Write, Result};
use arduino_hal::Usart;

struct FmtSerial<'a, U: Write>(&'a mut U);

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


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut delay = arduino_hal::Delay::new();
    let pins = arduino_hal::pins!(dp);

    // デフォルトシリアルを取得
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
        let mut fmt_serial = FmtSerial(&mut serial);
        let mut logger = SerialLogger::new(&mut fmt_serial);

    let mut i2c = arduino_hal::I2c::new(dp.TWI, pins.a4, pins.a5, 100_000);

scan_i2c(&mut i2c, &mut logger);


    loop {
        delay.delay_ms(1000u16);
    }
}
