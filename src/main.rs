#![no_std]
#![no_main]

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

    // ---- I2C Bus Scan ----
    writeln!(serial, "[Info] Scanning I2C bus for devices...").ok();
    let found_addrs = scan_i2c(&mut i2c, &mut serial, LogLevel::Verbose);

    if !found_addrs.unwrap().contains(&0x3C) {
        writeln!(serial, "[Error] SH1107G device (0x3C) not found. Check wiring.").ok();
    } else {
        writeln!(serial, "[Info] SH1107G device found at 0x3C.").ok();

        // ---- Explorer for SH1107G initialization sequence ----
        // Use `static` instead of `const` to place data into Flash memory.
        #[link_section = ".progmem.data"]
        static EXPLORER_CMDS: [CmdNode; 13] = [
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

        // Use `static` instead of `const`.
        #[link_section = ".progmem.data"]
        static INIT_SEQ_BYTES: [u8; 21] = [
            0xAE, 0xD5, 0x51, 0xCA, 0x7F, 0xA2, 0x00, 0xA1, 0x00,
            0xA0, 0xC8, 0xAD, 0x8A, 0xD9, 0x22, 0xDB, 0x35, 0x8D,
            0x14, 0xA6, 0xAF,
        ];

        let explorer = Explorer::<13> { sequence: &EXPLORER_CMDS };

        // ---- Run exploring ----
        if let Err(e) = run_explorer::<_, _, 13, 128>(
            &explorer,
            &mut i2c,
            &mut serial,
            &INIT_SEQ_BYTES,
            0x00, // Prefix for commands (e.g., 0x00 for command mode)
            LogLevel::Verbose
        ) {
            writeln!(serial, "[error] Exploration failed: {:?}", e).ok();
        }
    }

    loop {}
}