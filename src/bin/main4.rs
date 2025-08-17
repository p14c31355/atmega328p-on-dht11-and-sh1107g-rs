#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::draw_target::DrawTarget;
use panic_halt as _;
use sh1107g_rs::Sh1107g;
use dvcdbg::prelude::*;
use dvcdbg::scanner::scan_i2c;
use embedded_io::Write;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let serial = default_serial!(dp, pins, 57600);

    adapt_serial!(UsartAdapter, serial, write_byte);
    let mut dbg_uart = UsartAdapter(serial);
    let mut logger = SerialLogger::new(&mut dbg_uart);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    scan_i2c(&mut i2c, &mut logger);

    let mut display = Sh1107g::new(&mut i2c, 0x3C);

    log!(logger, "Initializing...");
    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    log!(logger, "Display initialized and cleared.");
    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();

    log!(logger, "Display filled with white.");

    loop {}
}
