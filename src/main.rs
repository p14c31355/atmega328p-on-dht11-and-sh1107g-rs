#![no_std]
#![no_main]

use arduino_hal::Peripherals;
use panic_halt as _;

use sh1107g_rs::Sh1107gBuilder;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use dvcdbg::logger::SerialLogger;
use log::info;

use embedded_hal::serial::Write as SerialWrite;

// arduino_hal の Usart は nb::Write<u8> を実装している
struct FmtWriteWrapper<W>(W);

// ここがポイント！
// core::fmt::Write はジェネリック無し。W は nb::Write<u8> を実装している必要あり。
impl<W: SerialWrite<u8>> core::fmt::Write for FmtWriteWrapper<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            nb::block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);

    // SerialLogger は core::fmt::Write を要求するので OK
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    info!("Starting Arduino application...");

    let i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    let mut display = Sh1107gBuilder::new(i2c, &mut logger).build().unwrap();

    info!("Display driver built successfully.");

    display.init().unwrap();
    display.clear_buffer();

    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    Text::new("Hello, World!", Point::new(16, 64), character_style)
        .draw(&mut display)
        .unwrap();

    info!("Text 'Hello, World!' drawn to buffer.");

    display.flush().unwrap();

    info!("Buffer flushed to display.");

    loop {}
}
