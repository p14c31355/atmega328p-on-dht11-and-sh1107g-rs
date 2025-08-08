#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use panic_halt as _;

use sh1107::{Builder, DisplayRotation, Interface};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // I2Cセットアップ (SCL = A5, SDA = A4)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a5.into_pull_up_input(),
        pins.a4.into_pull_up_input(),
        400_000,
    );

    // SH1107 初期化
    let mut disp = Builder::new()
        .with_rotation(DisplayRotation::Rotate0)
        .connect_i2c(i2c)
        .init()
        .unwrap();

    // 画面クリア
    disp.clear();

    // 文字スタイル準備
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    // 文字列描画 (左上から)
    Text::new("Hello, world!", Point::zero(), text_style)
        .draw(&mut disp)
        .unwrap();

    // 描画反映
    disp.flush().unwrap();

    loop {
        // 無限ループ
    }
}
