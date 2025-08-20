#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use core::fmt::Write;

use dvcdbg::prelude::*;

adapt_serial!(UnoSerial);

#[arduino_hal::entry]
fn main() -> ! {

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut serial_logger = UnoSerial(serial);

    writeln!(serial_logger, "Hello from dvcdbg on Arduino Uno!");

    loop {
        // your code
    }
}
