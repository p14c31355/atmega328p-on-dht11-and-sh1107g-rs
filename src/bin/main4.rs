#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use dvcdbg::prelude::*;
use dvcdbg::log; // logマクロを使用するために必要

// adapt_serial! マクロで UsartAdapter に core::fmt::Write を実装
// nb::serial::Write をラップすることで core::fmt::Write の要件を満たす
adapt_serial!(UsartAdapter, nb_write = write_byte);


#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut delay = arduino_hal::Delay::new();

    // デフォルトのシリアルを取得
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // UsartAdapter を初期化
    let mut serial_adapter = UsartAdapter(serial);

    // log! マクロに直接アダプタを渡す
    log!(&mut serial_adapter, "[info] Starting main4.rs example...");

    for i in 0..10 {
        log!(&mut serial_adapter, "[count] {}", i);
    }

    log!(&mut serial_adapter, "[info] Finished loop, entering infinite loop.");

    loop {
        // 永久ループで終了しない
        delay.delay_ms(1000u16);
    }
}