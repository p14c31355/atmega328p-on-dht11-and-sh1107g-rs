#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;

use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Serial 初期化
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Full Init Test]").ok();

    // I2C 初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- Explorer 用コマンド定義 ----
    // 0x00 制御バイトを各コマンドの先頭に含めています
    const EXPLORER_CMDS: [CmdNode; 14] = [
    CmdNode { bytes: &[0xAE], deps: &[] },                     // 0
    CmdNode { bytes: &[0xD5, 0x51], deps: &[0_usize] },    // 1
    CmdNode { bytes: &[0xA8, 0x3F], deps: &[1_usize] },    // 2
    CmdNode { bytes: &[0xD3, 0x60], deps: &[2_usize] },    // 3
    CmdNode { bytes: &[0x40, 0x00], deps: &[3_usize] },    // 4
    CmdNode { bytes: &[0xA1, 0x00], deps: &[4_usize] },    // 5
    CmdNode { bytes: &[0xA0], deps: &[5_usize] },          // 6
    CmdNode { bytes: &[0xC8], deps: &[6_usize] },          // 7
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[7_usize] },    // 8
    CmdNode { bytes: &[0xD9, 0x22], deps: &[8_usize] },    // 9
    CmdNode { bytes: &[0xDB, 0x35], deps: &[9_usize] },    // 10
    CmdNode { bytes: &[0x8D, 0x14], deps: &[10_usize] },   // 11
    CmdNode { bytes: &[0xA6], deps: &[11_usize] },         // 12
    CmdNode { bytes: &[0xAF], deps: &[12_usize] },         // 13
];



    let explorer = Explorer::<14> {
        sequence: &EXPLORER_CMDS,
    };

    // ---- Explore ----
    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    if let Err(e) = run_single_sequence_explorer::<_, _, 14, 128>(
        &explorer,
        &mut i2c,
        &mut serial,
        0x3C, // Specify the target address here
        0x00,
        LogLevel::Verbose,
    ) {
        writeln!(serial, "[error] Explorer failed: {:?}", e).ok();
    } else {
        writeln!(serial, "[Info] SH1107G full init test complete").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}
