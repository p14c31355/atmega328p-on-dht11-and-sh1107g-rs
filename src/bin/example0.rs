#![no_std]
#![no_main]

use arduino_hal::Delay;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use panic_halt as _;
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::prelude::*;
use arduino_hal::default_serial;
use embedded_hal::blocking::serial::Write; // embedded_hal::blocking::serial::Writeトレイトをインポート
use nb; // nbクレートをインポート
use arduino_hal::pac::USART0;
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::hal::clock::MHz16;


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = Delay::new();

    let mut serial = default_serial!(dp, pins, 57600);
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    let mut display = Sh1107gBuilder::new(i2c).build();

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello, World!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    loop {}
}
