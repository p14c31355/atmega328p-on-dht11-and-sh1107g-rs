#![no_std]
#![no_main]

use arduino_hal::{i2c::I2c, Peripherals};
use panic_halt as _;

use sh1107g_rs::Sh1107gBuilder;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::Text,
};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
 
    let i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        25_000,
    );

    let builder = Sh1107gBuilder::new().connect_i2c(i2c);
    let mut display = builder.build().unwrap();
    display.init().unwrap();

    // テキスト描画前に画面クリア
    display.clear(BinaryColor::Off).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello OLED!", Point::new(0, 0), style)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    loop {}
}
