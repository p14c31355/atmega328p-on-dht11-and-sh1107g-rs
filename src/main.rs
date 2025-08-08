#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    primitives::PrimitiveStyle,
};
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::logger::SerialLogger;

use embedded_hal::serial::Write;
use core::fmt::Write as FmtWrite;

struct FmtWriteWrapper<W>(W);

impl<W> core::fmt::Write for FmtWriteWrapper<W>
where
    W: Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            nb::block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        // .with_size(DisplaySize::Display128x128) ← 削除
        .build()
        .unwrap_or_else(|_| {
            writeln!(logger, "ERR: display init failed").ok();
            panic!()
        });

    let white_style = PrimitiveStyle::with_fill(BinaryColor::On);

    let rect = Rectangle::new(Point::new(0, 0), Size::new(128, 128));

    if let Err(_) = rect.into_styled(white_style).draw(&mut display) {
        writeln!(logger, "ERR: draw failed").ok();
    }

    if let Err(_) = display.flush() {
        writeln!(logger, "ERR: flush failed").ok();
    } else {
        writeln!(logger, "OK: white screen drawn").ok();
    }

    loop {}
}

// panic-halt の利用で panic_handler は不要
use panic_halt as _;
