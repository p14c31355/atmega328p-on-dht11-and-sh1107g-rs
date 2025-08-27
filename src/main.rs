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

    // Serial initialization
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Full Init Test]").ok();

    // I2C initialization
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    let explorer_cmds: [CmdNode; 13] = [
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

    // ---- Send all commands to address 0x3C ----
    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    for cmd_node in explorer_cmds.iter() {
        let result = i2c.write(0x3C, cmd_node.bytes);
        match result {
            Ok(_) => writeln!(serial, "[OK] Command sent: {:X?}", cmd_node.bytes).ok(),
            Err(_) => writeln!(serial, "[ERROR] Command failed: {:X?}", cmd_node.bytes).ok(),
        };
        arduino_hal::delay_ms(50); // Optional: small delay between commands
    }

    writeln!(serial, "[Info] SH1107G full init test complete").ok();

    loop {
        arduino_hal::delay_ms(1000);
    }
}
