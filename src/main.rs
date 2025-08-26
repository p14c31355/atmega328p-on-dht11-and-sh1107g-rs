#![no_std]
#![no_main]

use core::fmt::Write;

use arduino_hal::prelude::*;
use panic_halt as _;
use dvcdbg::prelude::*;
use embedded_graphics_core::{
    pixelcolor::BinaryColor,
    draw_target::DrawTarget,
    geometry::Point,
};

adapt_serial!(UnoWrapper);

use sh1107g_rs::{Sh1107gBuilder, error::Sh1107gError};
use dvcdbg::compat::I2cCompat;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // Arduino Uno の I2C 初期化
    let i2c0 = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // e-hal 互換ラッパーで I2C
    let i2c = I2cCompat::new(i2c0);

    // シリアルログ
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UnoWrapper(serial);

    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    let _ = writeln!(logger, "[log] Init SH1107G...");

    // 初期化時のエラー出力
    if let Err(e) = display.init() {
        log_error(&mut logger, "init failed", &e);
    }

    // 画面に8x8ドットのテスト描画
    for y in 0..8 {
        for x in 0..8 {
            let _ = display.draw_iter(core::iter::once(
                embedded_graphics_core::Pixel(Point::new(x, y), BinaryColor::On)
            ));
        }
    }

    if let Err(e) = display.flush() {
        log_error(&mut logger, "flush failed", &e);
    }

    loop {
        delay.delay_ms(1000u16);
    }
}

// Sh1107gError を UnoWrapper でログ出力
fn log_error<I2C, E>(logger: &mut UnoWrapper<impl core::fmt::Write>, msg: &str, err: &Sh1107gError<E>)
where
    E: core::fmt::Debug,
{
    let _ = writeln!(logger, "[error] {}: {:?}", msg, err);
}
