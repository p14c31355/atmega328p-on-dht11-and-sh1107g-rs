#![no_std]
#![no_main]

use arduino_hal::{i2c, Peripherals};
use panic_halt as _;

use sh1107g_rs::Sh1107gBuilder;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};


#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA (Uno: A4)
        pins.a5.into_pull_up_input(), // SCL (Uno: A5)
        100_000,
    );

    // OLED 初期化
    let builder = Sh1107gBuilder::new().connect_i2c(i2c);
    let mut display = builder.build().unwrap();
    display.init().unwrap();

    // "Hello!" を表示
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello!", Point::new(0, 0), text_style)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    arduino_hal::delay_ms(500); // 0.5秒だけ待つ

    loop {
        // 永久ループで終了させない
    }
}
