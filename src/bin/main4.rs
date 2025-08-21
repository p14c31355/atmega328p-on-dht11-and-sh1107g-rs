#![no_std]
#![no_main]

use panic_halt as _;
use atmega328p_on_dht11_and_sh1107g_rs::*; // lib.rs の公開アイテムをインポート

adapt_serial!(UnoWrapper); // UnoWrapper を定義

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut logger = UnoWrapper(serial); // UnoWrapper を使用

    writeln!(logger, "Hello from dvcdbg on Arduino Uno!").unwrap();

    loop {
        arduino_hal::delay_ms(1000);
    }
}
