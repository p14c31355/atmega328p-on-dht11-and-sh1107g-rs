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
    static DEP_0: &[usize] = &[];
    static DEP_1: &[usize] = &[0];
    static DEP_2: &[usize] = &[1];
    static DEP_3: &[usize] = &[2];
    static DEP_4: &[usize] = &[3];
    static DEP_5: &[usize] = &[4];
    static DEP_6: &[usize] = &[5];
    static DEP_7: &[usize] = &[6];
    static DEP_8: &[usize] = &[7];
    static DEP_9: &[usize] = &[8];
    static DEP_10: &[usize] = &[9];
    static DEP_11: &[usize] = &[10];
    static DEP_12: &[usize] = &[11];
    static DEP_13: &[usize] = &[12];

    // ---- コマンド配列 ----
    static EXPLORER_CMDS: [CmdNode; 14] = [
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

    // ---- Explorer の初期化 ----
    // const fn で最大コマンド長を取得
    const fn calculate_max_cmd_len(cmds: &[CmdNode]) -> usize {
        let mut max_len = 0;
        let mut i = 0;
        while i < cmds.len() {
            let len = cmds[i].bytes.len();
            if len > max_len {
                max_len = len;
            }
            i += 1;
        }
        max_len + 1 // prefix add
    }

    const BUF_CAP: usize = calculate_max_cmd_len(&EXPLORER_CMDS);

    // Explorer 型に MAX_CMD_LEN を const パラメータとして渡す
    let explorer: Explorer<'static, 14, BUF_CAP> = Explorer {
        sequence: &EXPLORER_CMDS,
    };


    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    // for (i, node) in EXPLORER_CMDS.iter().enumerate() {
    //     writeln!(serial, "Node {} bytes={:02X?}, deps={:?}", i, node.bytes, node.deps).ok();
    // }


    // Explorer 実行
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
