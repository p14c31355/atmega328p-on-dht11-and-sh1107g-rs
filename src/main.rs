// main.rs
#![no_std]
#![no_main]

use core::fmt::Write;
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

    // Init sequence nodes
    static EXPLORER_CMDS: [CmdNode; 17] = [
        CmdNode { bytes: &[0xAE], deps: &[] },                  // Display OFF
        CmdNode { bytes: &[0xD5, 0x51], deps: &[0] },           // Set display clock
        CmdNode { bytes: &[0xA8, 0x3F], deps: &[1] },           // Set multiplex
        CmdNode { bytes: &[0xD3, 0x60], deps: &[2] },           // Set display offset
        CmdNode { bytes: &[0x40, 0x00], deps: &[3] },           // Set start line
        CmdNode { bytes: &[0xA1, 0x00], deps: &[4] },           // Segment remap + col offset
        CmdNode { bytes: &[0xA0], deps: &[5] },                 // Set scan direction
        CmdNode { bytes: &[0xC8], deps: &[6] },                 // COM scan direction
        CmdNode { bytes: &[0xAD, 0x8A], deps: &[7] },           // Set charge pump
        CmdNode { bytes: &[0xD9, 0x22], deps: &[8] },           // Set pre-charge
        CmdNode { bytes: &[0xDB, 0x35], deps: &[9] },           // Set VCOM detect
        CmdNode { bytes: &[0x8D, 0x14], deps: &[10] },          // Enable charge pump
        CmdNode { bytes: &[0xB0], deps: &[11] },                // Set page start
        CmdNode { bytes: &[0x00], deps: &[12] },                // Set lower column
        CmdNode { bytes: &[0x10], deps: &[12] },                // Set higher column
        CmdNode { bytes: &[0xA6], deps: &[12] },                // Normal display
        CmdNode { bytes: &[0xAF], deps: &[15] },                // Display ON
    ];

    let explorer: Explorer<'_, 17> = Explorer { sequence: &EXPLORER_CMDS };
    let prefix: u8 = 0x00;

    let _ = match dvcdbg::explore::runner::run_single_sequence_explorer::<_, _, 17, 2, 64>(
        &explorer,
        &mut i2c,
        &mut serial,
        prefix,
        0x3C, // Add target_addr
    ) {
        Ok(_) => writeln!(serial, "[I] Explorer OK.").ok(),
        Err(e) => writeln!(serial, "[E] Explorer failed: {:?}\r\n", e).ok(),
    };

    writeln!(serial, "Enter main loop.").ok();
    loop {
        arduino_hal::delay_ms(1000);
    }
}
