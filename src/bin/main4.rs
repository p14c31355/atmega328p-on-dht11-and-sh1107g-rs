#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _; // panic handler
use dvcdbg::adapt_serial;
use core::fmt::Write;
use embedded_io::Write as eioWrite;
// -------------------------
// Serial adapter
// -------------------------
adapt_serial!(UsartAdapter, nb_write = write);

#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut delay = arduino_hal::Delay::new();

    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // -------------------------
    // Logger を準備
    // -------------------------
    let mut adapter = UsartAdapter(serial);

    #[cfg(feature = "logger")]
    let mut logger = SerialLogger::new(&mut adapter);

    // -------------------------
    // ログ出力
    // -------------------------
    writeln!(adapter, "[info] Starting main4.rs example...").ok();

    loop {
        delay.delay_ms(1000u16);
    }
}
