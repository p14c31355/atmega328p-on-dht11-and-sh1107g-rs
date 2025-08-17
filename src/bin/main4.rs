#![no_std]
#![no_main]

use panic_halt as _; // panic時は停止
use dvcdbg::adapt_serial;
use dvcdbg::prelude::*;
use arduino_hal::default_serial;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = default_serial!(dp, pins, 57600);

    adapt_serial!(UsartAdapter);
    let mut dbg_uart = UsartAdapter(serial);
    let mut logger = SerialLogger::new(&mut dbg_uart);

    log!(logger, "Hello from AVR!");
    loop {}
}
