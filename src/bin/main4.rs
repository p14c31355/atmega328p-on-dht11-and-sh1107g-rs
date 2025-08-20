#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use core::fmt::Write;

use dvcdbg::prelude::*; // ← log! マクロとかまとめて import

// Serial を dvcdbg ロガーに適応させる
adapt_serial!(UnoSerial);

#[arduino_hal::entry]
fn main() -> ! {
    // デバイス初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Uno の UART 初期化 (9600 baud)
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // dvcdbg のアダプタで包む
    let mut serial_logger = UnoSerial(serial);

    // Hello world ログを出してみる
    writeln!(serial_logger, "Hello from dvcdbg on Arduino Uno!");

    loop {
        // ここにアプリ処理を書く
    }
}
