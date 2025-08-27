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

    // ---- Serial init ----
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Explorer Test]").ok();

    // ---- I2C init ----
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- I2C scan ----
    writeln!(serial, "[Info] Scanning I2C bus...").ok();
    let addr = scan_i2c(&mut i2c, &mut serial, &[0x00], LogLevel::Verbose);

    // ---- Explorer sequence ----
    #[link_section = ".progmem.data"]
    static EXPLORER_CMDS: [CmdNode; 13] = [
        CmdNode { bytes: &[0xAE], deps: &[] },        // Display off
        CmdNode { bytes: &[0xD5, 0x51], deps: &[] },  // Clock div
        CmdNode { bytes: &[0xCA, 0x7F], deps: &[] },  // Multiplex
        CmdNode { bytes: &[0xA2, 0x00], deps: &[] },  // Offset
        CmdNode { bytes: &[0xA1, 0x00], deps: &[] },  // Start line
        CmdNode { bytes: &[0xA0], deps: &[] },        // Segment remap
        CmdNode { bytes: &[0xC8], deps: &[] },        // COM scan dir
        CmdNode { bytes: &[0xAD, 0x8A], deps: &[] },  // Vpp
        CmdNode { bytes: &[0xD9, 0x22], deps: &[] },  // Precharge
        CmdNode { bytes: &[0xDB, 0x35], deps: &[] },  // VCOMH
        CmdNode { bytes: &[0x8D, 0x14], deps: &[] },  // Charge pump
        CmdNode { bytes: &[0xA6], deps: &[] },        // Normal display
        CmdNode { bytes: &[0xAF], deps: &[] },        // Display on
    ];

    #[link_section = ".progmem.data"]
    static INIT_SEQ_BYTES: [u8; 21] = [
        0xAE, 0xD5, 0x51, 0xCA, 0x7F, 0xA2, 0x00, 0xA1, 0x00,
        0xA0, 0xC8, 0xAD, 0x8A, 0xD9, 0x22, 0xDB, 0x35,
        0x8D, 0x14, 0xA6, 0xAF,
    ];

    let explorer = Explorer::<13> { sequence: &EXPLORER_CMDS };

    // ---- Run explorer ----
    if let Err(e) = run_explorer::<_, _, 13, 128>(
        &explorer,
        &mut i2c,
        &mut serial,
        &INIT_SEQ_BYTES, // Initial sequence to test
        addr.as_ref().map_or(0x00, |v| v[0]), // SH1107G I2C address (0x3C/0x3D)
        LogLevel::Verbose,
    ) {
        writeln!(serial, "[error] Exploration failed: {:?}", e).ok();
    }

    // ---- Loop ----
    loop {
        arduino_hal::delay_ms(1000);
    }
}
