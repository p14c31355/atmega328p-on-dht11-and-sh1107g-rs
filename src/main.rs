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

    // ---- 依存関係定義 ----
    static EXPLORER_CMDS: [CmdNode; 14] = [
    CmdNode { bytes: &[0xAE], deps: &[] },       // 0: Display OFF
    CmdNode { bytes: &[0xD5, 0x51], deps: &[0] }, // 1: Clock / Oscillator
    CmdNode { bytes: &[0xA8, 0x3F], deps: &[1] }, // 2: Multiplex Ratio
    CmdNode { bytes: &[0xD3, 0x60], deps: &[2] }, // 3: Display Offset
    CmdNode { bytes: &[0x40, 0x00], deps: &[3] }, // 4: Start Line
    CmdNode { bytes: &[0xA1, 0x00], deps: &[4] }, // 5: Segment Re-map
    CmdNode { bytes: &[0xA0], deps: &[5] },       // 6: Segment Re-map (alt)
    CmdNode { bytes: &[0xC8], deps: &[6] },       // 7: COM Output Scan
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[7] }, // 8: DC-DC Converter
    CmdNode { bytes: &[0xD9, 0x22], deps: &[8] }, // 9: Pre-charge Period
    CmdNode { bytes: &[0xDB, 0x35], deps: &[9] }, // 10: VCOMH Deselect
    CmdNode { bytes: &[0x8D, 0x14], deps: &[10] },// 11: Charge Pump
    CmdNode { bytes: &[0xA6], deps: &[11] },      // 12: Normal Display
    CmdNode { bytes: &[0xAF], deps: &[12] },      // 13: Display ON
];


    const BUF_CAP: usize = 3; // 最大2バイト + prefix

let explorer: Explorer<'static, 14, BUF_CAP> = Explorer {
    sequence: &EXPLORER_CMDS,
};


    // ---- デバッグ表示 ----
    for (i, node) in EXPLORER_CMDS.iter().enumerate() {
        writeln!(serial, "Node {} bytes={:02X?}, deps={:?}", i, node.bytes, node.deps).ok();
    }

    // ---- バッファ容量（最大コマンド長 + prefix） ----
    // const BUF_CAP: usize = 3; // 最大2バイト + prefix

    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();

    if let Err(e) = run_single_sequence_explorer::<_, _, 14, BUF_CAP>(
        &explorer,
        &mut i2c,
        &mut serial,
        0x3C, // SH1107 I2C address
        0x00, // prefix
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
