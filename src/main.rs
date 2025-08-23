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

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // -------------------------
    // シリアル初期化（115200固定で安定）
    // -------------------------
    let serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut serial_wrapper = UnoWrapper(serial);

    writeln!(serial_wrapper, "[log] Start Uno + SH1107G test").ok();

    // -------------------------
    // I2C 初期化
    // -------------------------
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // -------------------------
    // I2Cスキャンをバッファリング
    // -------------------------
    let mut scan_buf: heapless::String<256> = heapless::String::new();

    scan_buf.push_str("[scan] I2C scan start\n").ok();
    scan_i2c(&mut i2c, &mut scan_buf);
    scan_buf.push_str("[scan] I2C scan done\n").ok();

    // バッファまとめて出力
    writeln!(serial_wrapper, "{}", scan_buf).ok();

    // -------------------------
    // OLED 初期化
    // -------------------------
    let mut oled = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    if oled.init().is_ok() {
        writeln!(serial_wrapper, "[oled] init complete").ok();
    } else {
        writeln!(serial_wrapper, "[oled] init failed!").ok();
    }

    // -------------------------
    // 画面クリア & 十字描画
    // -------------------------
    oled.clear(BinaryColor::Off).ok();

    // 十字
    for x in 0..DISPLAY_WIDTH {
        let _ = oled.draw_iter([Pixel(Point::new(x as i32, (DISPLAY_HEIGHT/2) as i32), BinaryColor::On)]);
    }
    for y in 0..DISPLAY_HEIGHT {
        let _ = oled.draw_iter([Pixel(Point::new((DISPLAY_WIDTH/2) as i32, y as i32), BinaryColor::On)]);
    }

    // 矩形描画
    let rect = Rectangle::new(Point::new((DISPLAY_WIDTH/2 - 10) as i32, (DISPLAY_HEIGHT/2 - 10) as i32),
                              Size::new(20, 20));
    let _ = oled.draw_iter(rect.points().map(|p| Pixel(p, BinaryColor::On)));

    oled.flush().ok();
    writeln!(serial_wrapper, "[oled] cross + rect drawn").ok();

    loop {
        delay.delay_ms(1000u16);
    }
}
