#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use panic_halt as _;
use embedded_io::Write;

adapt_serial!(UnoSerial);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoSerial(arduino_hal::default_serial!(dp, pins, 57600));
    writeln!(serial, "[scan] start").ok();

    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    scan_i2c(&mut i2c, &mut serial);

    for addr in 0x03..=0x77 {
        let _ = match i2c.write(addr, &[]) {
            Ok(_) => writeln!(serial, "[found] device at 0x{:02X}", addr).ok(),
            Err(_) => None // 無視
        };
    }


    writeln!(serial, "[scan] done").ok();

    loop {}
}
