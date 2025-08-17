#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Usart;
use dvcdbg::adapt_serial;
use core::fmt::Write;

adapt_serial!(UsartAdapter);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut dbg_uart = UsartAdapter(serial);
    writeln!(dbg_uart, "Hello!").ok();

    loop {}
}