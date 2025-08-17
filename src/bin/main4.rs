#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Usart;
use dvcdbg::adapt_serial; // マクロをインポート
use panic_halt as _;

adapt_serial!(UsartAdapter);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // USART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // マクロで作られたアダプタを使う
    let mut dbg_uart = UsartAdapter(serial);

    // write! マクロが使える
    use core::fmt::Write;
    writeln!(dbg_uart, "Hello from dvcdbg!").ok();

    loop {}
}
