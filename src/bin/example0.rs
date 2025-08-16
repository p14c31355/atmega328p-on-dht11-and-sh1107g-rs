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

// SerialWriter構造体とimpl_fmt_write_for_serial!マクロを追加
pub struct SerialWriter<'a, T>
where
    T: Write<Word = u8>,
{
    serial: &'a mut T,
}

impl<'a, T> SerialWriter<'a, T>
where
    T: Write<Word = u8>,
{
    pub fn new(serial: &'a mut T) -> Self {
        SerialWriter { serial }
    }
}

impl<'a, T> core::fmt::Write for SerialWriter<'a, T>
where
    T: Write<Word = u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            // nb::block!はノンブロッキング操作をブロッキングに変換
            nb::block!(self.serial.write(byte)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = Delay::new();

    let mut serial = default_serial!(dp, pins, 57600);
    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);
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
