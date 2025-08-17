#![no_std]
#![no_main]

use dvcdbg::adapt_serial;
use core::fmt::Write;

adapt_serial!(UsartAdapter, nb_write = write, error = core::convert::Infallible);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut dbg_uart = UsartAdapter(serial);

    writeln!(dbg_uart, "Hello from embedded-io bridge!").ok();

    loop {}
}