#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use dvcdbg::logger::{BufferedLogger, Logger};
use dvcdbg::log; // log! マクロをインポート
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder, Sh1107g};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut delay = arduino_hal::Delay::new();

    // I2C の初期化（SDA: PC4, SCL: PC5）
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000, // 100kHz
    );

    // デフォルトロガー（静的バッファサイズ 256）
    static mut LOG_BUFFER: BufferedLogger<256> = BufferedLogger::new();
    let logger = unsafe { &mut LOG_BUFFER };

    let mut oled: Sh1107g<_, _> = Sh1107gBuilder::new()
        .with_address(0x3C)
        .build_logger(i2c, logger);

    match oled.init() {
        Ok(_) => log!(logger, "Init successful!"),
        Err(e) => log!(logger, "Init failed: {:?}", e),
    }

    match oled.flush() {
        Ok(_) => log!(logger, "Flush successful!"),
        Err(e) => log!(logger, "Flush failed: {:?}", e),
    }

    loop {}
}
