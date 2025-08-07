#![no_std]
#![no_main]

use arduino_hal::hal::i2c::I2c;
use arduino_hal::prelude::*;
use arduino_hal::{delay_ms, Peripherals};
use dvcdbg::logger::*; // ロガー構造体とトレイト
use panic_halt as _; // panic 時は停止

use sh1107g_rs::{builder::Sh1107gBuilder, Sh1107g}; // ドライバ本体

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化 (SDA = A4, SCL = A5)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,                      // 400kHz
    );

    // ロガー初期化
    let mut logger = BufferedLogger::<256>::new();

    // ドライバ構築
    let mut oled: Sh1107g<_, _> = Sh1107gBuilder::new()
        .with_i2c(i2c)
        .with_logger(&mut logger)
        .with_address(0x3C) // Grove OLED の I2C アドレス
        .build();

    // 初期化
    if let Err(e) = oled.init() {
        log!(logger, "Init failed: {:?}", e);
    } else {
        log!(logger, "Init successful!");
    }

    // 全面白表示 (fill_buf によるバッファ操作)
    for byte in oled.buffer_mut().iter_mut() {
        *byte = 0xFF;
    }

    if let Err(e) = oled.flush() {
        log!(logger, "Flush failed: {:?}", e);
    } else {
        log!(logger, "Flush successful!");
    }

    // ログ出力の確認（例：UART に流す場合などに備え）
    let log = logger.buffer();
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    for b in log.bytes() {
        nb::block!(serial.write(b)).ok();
    }

    // 無限ループ
    loop {
        delay_ms(1000);
    }
}
