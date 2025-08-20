#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use core::fmt::Write;

use dvcdbg::prelude::*;

adapt_serial!(SerialAdapter);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::default_pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // SerialCompatを実装する型を`SerialAdapter`でラップ
    let mut serial_logger = SerialAdapter(serial);

    // これで`writeln!`マクロを使用できる
    writeln!(serial_logger, "Hello from dvcdbg on Arduino Uno!").unwrap();

    loop {}
}
