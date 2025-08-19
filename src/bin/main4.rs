#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _; // panic handler

use dvcdbg::logger::Logger;

use dvcdbg::adapt_serial;
#[cfg(feature = "logger")]
use dvcdbg::logger::SerialLogger;
#[cfg(feature = "logger")]
use dvcdbg::log;

// -------------------------
// Serial adapter
// -------------------------
adapt_serial!(UsartAdapter, nb_write = write);

#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut delay = arduino_hal::Delay::new();

    // デフォルトシリアルを取得
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // -------------------------
    // Logger を準備
    // -------------------------
    let adapter = UsartAdapter(serial);

    #[cfg(feature = "logger")]
    let mut logger = SerialLogger::new(&mut { adapter });

    // -------------------------
    // ログ出力
    // -------------------------
    #[cfg(feature = "logger")]
    log!(logger, "[info] Starting main4.rs example...");

    for i in 0..10 {
        #[cfg(feature = "logger")]
        log!(logger, "[count] {}", i);
    }

    #[cfg(feature = "logger")]
    log!(logger, "[info] Finished loop, entering infinite loop.");

    loop {
        delay.delay_ms(1000u16);
    }
}
