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
    writeln!(serial, "\n[SH1107G Explorer Test]").ok();
    writeln!(serial, "").ok();

    // I2C initialization
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- Explorer for SH1107G initialization sequence ----
    const INIT_COMMANDS: [CmdNode; 21] = [
        CmdNode { bytes: &[0xAE], deps: &[] }, // Display off
        CmdNode { bytes: &[0xD5], deps: &[] }, // Set Display Clock Divide Ratio/Oscillator Frequency
        CmdNode { bytes: &[0x51], deps: &[] }, 
        CmdNode { bytes: &[0xCA], deps: &[] }, // Set Multiplex Ratio
        CmdNode { bytes: &[0x7F], deps: &[] },
        CmdNode { bytes: &[0xA2], deps: &[] }, // Set Display Offset
        CmdNode { bytes: &[0x00], deps: &[] },
        CmdNode { bytes: &[0xA1], deps: &[] }, // Set Display Start Line
        CmdNode { bytes: &[0x00], deps: &[] },
        CmdNode { bytes: &[0xA0], deps: &[] }, // Set Segment Re-map
        CmdNode { bytes: &[0xC8], deps: &[] }, // Set COM Output Scan Direction
        CmdNode { bytes: &[0xAD], deps: &[] }, // Set Vpp
        CmdNode { bytes: &[0x8A], deps: &[] },
        CmdNode { bytes: &[0xD9], deps: &[] }, // Set Pre-charge Period
        CmdNode { bytes: &[0x22], deps: &[] },
        CmdNode { bytes: &[0xDB], deps: &[] }, // Set VCOMH Deselect Level
        CmdNode { bytes: &[0x35], deps: &[] },
        CmdNode { bytes: &[0x8D], deps: &[] }, // Set Charge Pump
        CmdNode { bytes: &[0x14], deps: &[] },
        CmdNode { bytes: &[0xA6], deps: &[] }, // Normal Display
        CmdNode { bytes: &[0xAF], deps: &[] }, // Display on
    ];

    let init_seq: [u8; 0] = [];
    let cmds = &INIT_COMMANDS;
    let explorer = Explorer::<14> { sequence: cmds };

    // ---- Run exploring ----
    let _ = run_explorer::<_, _, 14, 0>(
        &explorer,
        &mut i2c,
        &mut serial,
        &init_seq,
        0x3C,
        LogLevel::Verbose
    );

    loop {}
}