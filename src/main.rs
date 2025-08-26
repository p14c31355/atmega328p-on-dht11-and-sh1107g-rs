#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_io::Write;
use panic_abort as _;

use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // serial initialization
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Explorer Test]").ok();

    // I2C initialization
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- Explorer for SH1107G initialization sequence ----
    // This is the sequence for the Explorer's permutation algorithm.
    // The number of commands is now 13.
    const EXPLORER_CMDS: [CmdNode; 13] = [
        CmdNode { bytes: &[0xAE], deps: &[] },      // Display off
        CmdNode { bytes: &[0xD5, 0x51], deps: &[] }, // Set Display Clock Divide Ratio/Oscillator Frequency
        CmdNode { bytes: &[0xCA, 0x7F], deps: &[] }, // Set Multiplex Ratio
        CmdNode { bytes: &[0xA2, 0x00], deps: &[] }, // Set Display Offset
        CmdNode { bytes: &[0xA1, 0x00], deps: &[] }, // Set Display Start Line
        CmdNode { bytes: &[0xA0], deps: &[] },      // Set Segment Re-map
        CmdNode { bytes: &[0xC8], deps: &[] },      // Set COM Output Scan Direction
        CmdNode { bytes: &[0xAD, 0x8A], deps: &[] }, // Set Vpp
        CmdNode { bytes: &[0xD9, 0x22], deps: &[] }, // Set Pre-charge Period
        CmdNode { bytes: &[0xDB, 0x35], deps: &[] }, // Set VCOMH Deselect Level
        CmdNode { bytes: &[0x8D, 0x14], deps: &[] }, // Set Charge Pump
        CmdNode { bytes: &[0xA6], deps: &[] },      // Normal Display
        CmdNode { bytes: &[0xAF], deps: &[] },      // Display on
    ];

    // This is the flat byte array for the initial scan.
    // The number of bytes is now 21.
    const INIT_SEQ_BYTES: [u8; 21] = [
        0xAE, 0xD5, 0x51, 0xCA, 0x7F, 0xA2, 0x00, 0xA1, 0x00,
        0xA0, 0xC8, 0xAD, 0x8A, 0xD9, 0x22, 0xDB, 0x35, 0x8D,
        0x14, 0xA6, 0xAF,
    ];

    let explorer = Explorer::<13> { sequence: &EXPLORER_CMDS };

    // ---- Run exploring ----
    // The `run_explorer` function returns a `Result`.
    // We handle the `Err` case explicitly to print the error message.
    if let Err(e) = run_explorer::<_, _, 13, 128>(
        &explorer,
        &mut i2c,
        &mut serial,
        &INIT_SEQ_BYTES,
        0x00, // Prefix for commands (e.g., 0x00 for command mode)
        LogLevel::Verbose
    ) {
        // Print the error if the exploration fails.
        writeln!(serial, "[error] Exploration failed: {:?}", e).ok();
    }

    loop {}
}