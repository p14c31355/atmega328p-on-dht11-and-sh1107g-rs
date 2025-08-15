#![no_std]
#![no_main]

use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::{mode, Pin};
use arduino_hal::Delay;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use panic_halt as _;
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::{log, logger::SerialLogger};
use core::fmt::Write; // log!マクロで必要になる可能性があるため追加
// use sh1107g_rs::sync::Display; // Displayトレイトは不要

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = Delay::new();

    let mut serial = SerialLogger::new(dp.USART0, pins.d0, pins.d1.into_output());
    writeln!(serial, "dvcdbg: Program Start").unwrap();

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );
    writeln!(serial, "dvcdbg: I2C Initialized").unwrap();

    let mut display = Sh1107gBuilder::new(i2c).build();
    writeln!(serial, "dvcdbg: Display Builder created").unwrap();

    display.init().unwrap();
    writeln!(serial, "dvcdbg: Display Initialized").unwrap();
    display.clear(BinaryColor::Off).unwrap();
    writeln!(serial, "dvcdbg: Display Cleared").unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello, World!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    writeln!(serial, "dvcdbg: Text Drawn").unwrap();

    display.flush().unwrap();
    writeln!(serial, "dvcdbg: Display Flushed").unwrap();

    loop {}
}
