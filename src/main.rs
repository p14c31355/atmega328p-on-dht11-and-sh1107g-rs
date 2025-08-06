#![no_std]
#![no_main]

use sh1107g_rs::{Sh1107gBuilder, cmds::*};
use dvcdbg::SerialLogger;
use arduino_hal::prelude::*;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    ufmt::uwriteln!(&mut serial, "Starting...").ok();

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    let mut logger = SerialLogger::new(serial);

    let mut display = Sh1107gBuilder::new(i2c)
        .with_address(0x3C)
        .with_logger(&mut logger)
        .init()
        .unwrap();

    display.clear();
    display.flush().unwrap();

    loop {}
}
