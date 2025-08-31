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
    const MAX_BYTES_PER_CMD: usize = 2;

    static EXPLORER_CMDS: [CmdNode; 15] = [
    CmdNode { bytes: &[0xAE], deps: &[] },           // 0
    CmdNode { bytes: &[0xD5, 0x51], deps: &[0] },    // 1
    CmdNode { bytes: &[0xA8, 0x3F], deps: &[1] },    // 2
    CmdNode { bytes: &[0xD3, 0x60], deps: &[2] },    // 3
    CmdNode { bytes: &[0x40], deps: &[3] },          // 4
    CmdNode { bytes: &[0xA1], deps: &[4] },          // 5
    CmdNode { bytes: &[0xA0], deps: &[5] },          // 6
    CmdNode { bytes: &[0xC8], deps: &[6] },          // 7
    CmdNode { bytes: &[0xAD, 0x8B], deps: &[7] },    // 8
    CmdNode { bytes: &[0xD9, 0x22], deps: &[8] },    // 9
    CmdNode { bytes: &[0xDB, 0x35], deps: &[9] },    // 10
    CmdNode { bytes: &[0x8D, 0x14], deps: &[10] },   // 11
    CmdNode { bytes: &[0x20, 0x00], deps: &[11] },   // 12
    CmdNode { bytes: &[0xA6], deps: &[12] },         // 13
    CmdNode { bytes: &[0xAF], deps: &[13] },         // 14
];

    let explorer: Explorer<'_, INIT_SEQUENCE_LEN> = Explorer {
        sequence: &EXPLORER_CMDS,
    };

    const CMD_BUFFER_SIZE: usize = calculate_cmd_buffer_size(1, MAX_BYTES_PER_CMD);

    let prefix: u8 = 0x00;
    let _ = match dvcdbg::explore::runner::run_single_sequence_explorer::<_, _, INIT_SEQUENCE_LEN, INIT_SEQUENCE_LEN, CMD_BUFFER_SIZE>(
        &explorer,
        &mut i2c,
        &mut serial,
        prefix,
        0x3C,
    ) {
        Ok(_) => writeln!(serial, "[I] Explorer OK.").ok(),
        Err(e) => writeln!(serial, "[E] Explorer failed: {:?}\r\n", e).ok(),
    };

    writeln!(serial, "Enter main loop.").ok();
    loop {
        arduino_hal::delay_ms(1000);
    }
}
