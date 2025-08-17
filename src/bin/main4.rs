#![no_std]
#![no_main]

use core::fmt::Write; // writeln! 用
use panic_halt as _;
use dvcdbg::adapt_serial;
use arduino_hal::prelude::*;
use arduino_hal::default_serial;

// embedded-io::Write と core::fmt::Write の衝突回避
use embedded_io::Write as IoWrite;

adapt_serial!(UsartAdapter, nb_write = write, flush = flush);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = default_serial!(dp, pins, 57600);

    let mut dbg_uart = UsartAdapter(serial);

    // core::fmt::Write 経由で文字列出力
    writeln!(dbg_uart, "Hello from embedded-io bridge!").ok();

    // embedded_io::Write 経由でバイト列送信
    dbg_uart.write_all(&[0x01, 0x02, 0x03]).ok();

    loop {}
}
