#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use embedded_io::Write;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, Rectangle},
};
use panic_halt as _;
use sh1107g_rs::{Sh1107g, Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    // writeln!(serial_wrapper, "[log] Start SH1107G test").unwrap();

    // I2C スキャン
    // scan_i2c(&mut i2c, &mut serial_wrapper);

    // SH1107G 初期化
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();
    display.init().unwrap();
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // embedded-graphics で描画
    // 十字線を描く
    let mid_x = DISPLAY_WIDTH as i32 / 2;
    let mid_y = DISPLAY_HEIGHT as i32 / 2;

    // 横線
    Line::new(Point::new(0, mid_y), Point::new(DISPLAY_WIDTH as i32 - 1, mid_y))
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    // 縦線
    Line::new(Point::new(mid_x, 0), Point::new(mid_x, DISPLAY_HEIGHT as i32 - 1))
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    // 枠も描く
    Rectangle::new(Point::new(0, 0), Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32))
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();

    // バッファをOLEDに送信
    display.flush().unwrap();
    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
