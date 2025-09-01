#![no_std]
#![no_main]

use core::fmt::Write;
use dvcdbg::compat::util::calculate_cmd_buffer_size;
use dvcdbg::explore::explorer::{CmdNode, Explorer};
use dvcdbg::prelude::*;
use panic_abort as _;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 115200));
    arduino_hal::delay_ms(1000);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    match i2c.write(0x3C, &[0x00]) {
        Ok(_) => writeln!(serial, "I2C OK.").ok(),
        Err(_) => writeln!(serial, "I2C failed.").ok(),
    };
    arduino_hal::delay_ms(1000);

    // --- Explorer nodes ---
    const INIT_SEQUENCE_LEN: usize = 22;
    let prefix: u8 = 0x00;

let res = factorial_sort! {
    prefix,
    init = [
        0xAE, 0xD5, 0x51, 0xA8, 0x3F, 0xD3, 0x60, 0x40,
        0x00, 0xA1, 0x00, 0xA0, 0xC8, 0xAD, 0x8B, 0xD9,
        0x22, 0xDB, 0x35, 0x8D, 0x14, 0xB0
    ],
    nodes = [
        [0xAE],
        [0xD5, 0x51] @ [0],
        [0xA8, 0x3F] @ [1],
        [0xD3, 0x60] @ [2],
        [0x40] @ [3],
        [0xA1] @ [4],
        [0xA0] @ [5],
        [0xC8] @ [6],
        [0xAD, 0x8B] @ [7],
        [0xD9, 0x22] @ [8],
        [0xDB, 0x35] @ [9],
        [0x8D, 0x14] @ [10],
        [0x20, 0x00] @ [11],
        [0xA6] @ [12],
        [0xAF] @ [13]
    ],
    &mut i2c,
    &mut serial
};

match res {
    Ok(()) => writeln!(serial, "[I] Explorer OK.").ok(),
    Err(e) => writeln!(serial, "[E] Explorer failed: {:?}\r\n", e).ok(),
};

    writeln!(serial, "Enter main loop.").ok();
    loop {
        arduino_hal::delay_ms(1000);
    }
}
