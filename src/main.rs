#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use panic_halt as _;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};

use sh1107g_rs::Sh1107gBuilder;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // ドライバ構築
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    // OLED 初期化
    display.init().unwrap();

    // e-grahpics 描画
    // 1. 黒背景に四角を描く
    Rectangle::new(Point::new(0, 0), Size::new(128, 128))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
        .draw(&mut display)
        .unwrap();

    // 2. 中心に白い四角
    Rectangle::new(Point::new(32, 32), Size::new(64, 64))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();

    // 3. テキスト描画
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello", Point::new(40, 60), text_style)
        .draw(&mut display)
        .unwrap();

    // バッファをフラッシュして OLED に表示
    display.flush().unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
