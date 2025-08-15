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
use arduino_hal::hal::usart::Usart;
use arduino_hal::port::{mode, Pin};
use embedded_hal::serial::nb::Write as EmbeddedHalSerialWrite; // embedded_hal::serial::nb::Write トレイトをインポート

// impl_fmt_write_for_serial! マクロを正確に記述
impl_fmt_write_for_serial!(Usart<arduino_hal::Atmega, arduino_hal::hal::usart::USART0, arduino_hal::port::Pin<arduino_hal::port::mode::Input, arduino_hal::port::PD0>, arduino_hal::port::Pin<arduino_hal::port::mode::Output, arduino_hal::port::PD1>, arduino_hal::clock::MHz16>, write);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = Delay::new();

    let mut serial = default_serial!(dp, pins, 57600);
    let mut logger = SerialLogger::new(&mut serial);
    log!(logger, "Program Start");

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );
    log!(logger, "I2C Initialized");

    let mut display = Sh1107gBuilder::new(i2c).build();
    log!(logger, "Display Builder created");

    display.init().unwrap();
    log!(logger, "Display Initialized");
    display.clear(BinaryColor::Off).unwrap();
    log!(logger, "Display Cleared");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello, World!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    log!(logger, "Text Drawn");

    display.flush().unwrap();
    log!(logger, "Display Flushed");

    loop {}
}
