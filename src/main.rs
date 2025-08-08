#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use arduino_hal::hal::port::Dynamic;
use arduino_hal::port::{mode, Pin};
use arduino_hal::Delay;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::{Alignment, Text},
};
use panic_halt as _;
use sh1107g_rs::{prelude::*, Sh1107g};
use dvcdbg::logger::{Logger, SerialLogger};
use dvcdbg::log;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = default_serial!(dp, pins, 57600);

    let mut logger = SerialLogger::new(&mut serial);

    log!(&mut logger, "Initializing...");

    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_mode(&mut pins.ddr),
        pins.a5.into_mode(&mut pins.ddr),
        50000,
    );

    let mut display: Sh1107g<
        I2cInterface<arduino_hal::I2c>,
        SerialLogger<arduino_hal::DefaultSerial>,
    > = Sh1107g::new(I2cInterface::new(i2c), 0x3C, Some(&mut logger));

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    log!(&mut logger, "Display initialized and cleared.");

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    log!(&mut logger, "Display filled with white.");

    loop {
        // 無限ループ
    }
}
