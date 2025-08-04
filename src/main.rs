#![no_std]
#![no_main]

use arduino_hal::{i2c::I2c, Peripherals};
use panic_halt as _;

use sh1107g_rs::Sh1107gBuilder;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};


#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
 
    let i2c = I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA (Uno: A4)
        pins.a5.into_pull_up_input(), // SCL (Uno: A5)
        25_000,
    );

    // OLED 初期化
    let builder = Sh1107gBuilder::new().connect_i2c(i2c);
    let mut display = builder.build().unwrap();
    display.init().unwrap();

    // 画面全体を白く塗りつぶす
    display.clear(BinaryColor::On).unwrap();

    loop {
        // flush() を繰り返し呼び出すことで、描画内容を画面に維持する
        display.flush().unwrap();
    }
}