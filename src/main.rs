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
    // 依存関係を静的定義
const DEP_0: &[usize] = &[];
const DEP_1: &[usize] = &[0];
const DEP_2: &[usize] = &[1];
const DEP_3: &[usize] = &[2];
const DEP_4: &[usize] = &[3];
const DEP_5: &[usize] = &[4];
const DEP_6: &[usize] = &[5];
const DEP_7: &[usize] = &[6];
const DEP_8: &[usize] = &[7];
const DEP_9: &[usize] = &[8];
const DEP_10: &[usize] = &[9];
const DEP_11: &[usize] = &[10];
const DEP_12: &[usize] = &[11];
const DEP_13: &[usize] = &[12];

// コマンド配列
let explorer_cmds: [CmdNode; 14] = [
    CmdNode { bytes: &[0xAE], deps: DEP_0 },       // 0: Display OFF
    CmdNode { bytes: &[0xD5, 0x51], deps: DEP_1 }, // 1: Clock / Oscillator
    CmdNode { bytes: &[0xA8, 0x3F], deps: DEP_2 }, // 2: Multiplex Ratio
    CmdNode { bytes: &[0xD3, 0x60], deps: DEP_3 }, // 3: Display Offset
    CmdNode { bytes: &[0x40, 0x00], deps: DEP_4 }, // 4: Start Line
    CmdNode { bytes: &[0xA1, 0x00], deps: DEP_5 }, // 5: Segment Re-map
    CmdNode { bytes: &[0xA0], deps: DEP_6 },       // 6: Segment Re-map (alt)
    CmdNode { bytes: &[0xC8], deps: DEP_7 },       // 7: COM Output Scan
    CmdNode { bytes: &[0xAD, 0x8A], deps: DEP_8 }, // 8: DC-DC Converter
    CmdNode { bytes: &[0xD9, 0x22], deps: DEP_9 }, // 9: Pre-charge Period
    CmdNode { bytes: &[0xDB, 0x35], deps: DEP_10 },// 10: VCOMH Deselect
    CmdNode { bytes: &[0x8D, 0x14], deps: DEP_11 },// 11: Charge Pump
    CmdNode { bytes: &[0xA6], deps: DEP_12 },      // 12: Normal Display
    CmdNode { bytes: &[0xAF], deps: DEP_13 },      // 13: Display ON
];




    let explorer = Explorer::<14> {
        sequence: &explorer_cmds,
    };

    // ---- Explore ----
    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    if let Err(e) = run_single_sequence_explorer::<_, _, 14, 2048>(
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
