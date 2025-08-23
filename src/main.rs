#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use sh1107g_rs::{Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::draw_target::DrawTarget;
use core::fmt::Write;
use panic_halt as _;

// -------------------------
// シリアル互換アダプタ
// -------------------------
adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // -------------------------
    // シリアル初期化
    // -------------------------
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);

    writeln!(serial_wrapper, "[log] Start Uno + SH1107G test").ok();

    // -------------------------
    // I2C 初期化
    // -------------------------
    let mut i2c = match i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    ) {
        Ok(bus) => bus,
        Err(_) => {
            writeln!(serial_wrapper, "[error] I2C init failed!").ok();
            panic!(); // ここで止めてログ確認
        }
    };

    writeln!(serial_wrapper, "[scan] I2C scan start").ok();

    // I2C スキャン
    if let Err(_) = scan_i2c(&mut i2c, &mut serial_wrapper) {
        writeln!(serial_wrapper, "[error] I2C scan failed!").ok();
        panic!();
    }

    writeln!(serial_wrapper, "[scan] I2C scan done").ok();


    // -------------------------
    // OLED 初期化
    // -------------------------
    let mut oled = Sh1107gBuilder::new(&mut i2c)
        .clear_on_init(true)
        .build();

    // OLED 初期化
    if let Err(_) = oled.init() {
        writeln!(serial_wrapper, "[oled] init failed!").ok();
        panic!(); // ここで停止
    } else {
        writeln!(serial_wrapper, "[oled] init complete").ok();
    }

    // 画面クリア
    if let Err(_) = oled.clear(BinaryColor::Off) {
        writeln!(serial_wrapper, "[oled] clear failed!").ok();
        panic!();
    }

    // 十字描画
    for x in 0..DISPLAY_WIDTH {
        if let Err(_) = oled.draw_iter([Pixel(Point::new(x as i32, (DISPLAY_HEIGHT / 2) as i32), BinaryColor::On)]) {
            writeln!(serial_wrapper, "[oled] draw horizontal failed at x={}", x).ok();
            panic!();
        }
    }
    for y in 0..DISPLAY_HEIGHT {
        if let Err(_) = oled.draw_iter([Pixel(Point::new((DISPLAY_WIDTH / 2) as i32, y as i32), BinaryColor::On)]) {
            writeln!(serial_wrapper, "[oled] draw vertical failed at y={}", y).ok();
            panic!();
        }
    }

    // 矩形描画
    let rect = Rectangle::new(Point::new((DISPLAY_WIDTH/2 - 10) as i32, (DISPLAY_HEIGHT/2 - 10) as i32),Size::new(20, 20));
    if let Err(_) = oled.draw_iter(rect.points().map(|p| Pixel(p, BinaryColor::On))) {
        writeln!(serial_wrapper, "[oled] draw rect failed").ok();
        panic!();
    }

    // flush
    if let Err(_) = oled.flush() {
        writeln!(serial_wrapper, "[oled] flush failed").ok();
        panic!();
    } else {
        writeln!(serial_wrapper, "[oled] cross + rect drawn").ok();
    }

    loop {
        delay.delay_ms(1000u16);
    }
}
