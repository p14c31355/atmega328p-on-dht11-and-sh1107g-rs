#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::{i2c, Peripherals};
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use ufmt::uwriteln;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化（デバッグ用）
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    uwriteln!(&mut serial, "Start!").ok();

    // I2C 初期化
    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    uwriteln!(&mut serial, "I2C init OK").ok();

    // SH1107G ディスプレイ初期化
    let mut display = match Sh1107gBuilder::new()
        .connect_i2c(i2c)
        .with_address(0x3C)
        .build()
    {
        Ok(d) => {
            uwriteln!(&mut serial, "Display build OK").ok();
            d
        }
        Err(e) => {
            uwriteln!(&mut serial, "Display build FAILED!").ok();
            // Optionally show the error (if it implements `core::fmt::Debug`)
            // uwriteln!(&mut serial, "{:?}", e).ok();
            loop {}
        }
    };

    if let Err(_) = display.init() {
        uwriteln!(&mut serial, "Display init FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Display init OK").ok();

    // 文字表示
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    if let Err(_) = Text::new("Hello!", Point::new(16, 16), text_style).draw(&mut display) {
        uwriteln!(&mut serial, "Draw FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Draw OK").ok();

    if let Err(_) = display.flush() {
        uwriteln!(&mut serial, "Flush FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Flush OK").ok();

    // 実行完了
    arduino_hal::delay_ms(500);
    uwriteln!(&mut serial, "All done.").ok();

    loop {} // 無限ループで終了させない
}
