#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _; // panic handler
use dvcdbg::prelude::*;
use dvcdbg::log;
use core::fmt::Write; // <--- core::fmt::Write をuseする

// ehal_0_2 feature が有効な場合のみ
#[cfg(feature = "ehal_0_2")]
adapt_serial!(UsartAdapter, nb_write = write_byte);


#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let mut delay = arduino_hal::Delay::new();

    // デフォルトのシリアルを取得
    let pins = arduino_hal::pins!(dp);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // UsartAdapter を直接 fmt::Write として使用
    let mut serial_adapter = UsartAdapter(serial);
    let mut logger = &mut serial_adapter;

    // log! マクロに直接渡す
    writeln!(logger, "[info] Starting main4.rs example...").ok();
    
    for i in 0..10 {
        writeln!(logger, "[count] {}", i).ok();
    }

    writeln!(logger, "[info] Finished loop, entering infinite loop.").ok();

    loop {
        // 永久ループで終了しない
        delay.delay_ms(1000u16);
    }
}