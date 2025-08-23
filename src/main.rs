#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_io::Write;
use panic_halt as _;
use sh1107g_rs::{Sh1107g, I2C_MAX_WRITE};

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let mut i2c = i2c::I2c::new(dp.TWI, pins.a4.into_pull_up_input(), pins.a5.into_pull_up_input(), 100_000);
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);

    writeln!(serial_wrapper, "[log] Start SH1107G test").unwrap();
    scan_i2c(&mut i2c, &mut serial_wrapper);

    let mut display = Sh1107g::new(i2c, 0x3C);
    display.init().unwrap();

    // 十字描画
    let width = 128;
    let height = 128;
    let mut buffer = display.buffer_mut();
    let mid_x = width / 2;
    let mid_y = height / 2;

    for y in 0..height {
        for x in 0..width {
            let byte_index = x + (y / 8) * width;
            let bit_mask = 1 << (y % 8);
            if x == mid_x || y == mid_y {
                buffer[byte_index] |= bit_mask;
            }
        }
    }

    display.flush().unwrap();
    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}
