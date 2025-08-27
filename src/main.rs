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
    let explorer_cmds: [CmdNode; 15] = [
        CmdNode {
            bytes: &[0xAE],
            deps: &[],
        }, // 0: Display OFF
        CmdNode {
            bytes: &[0xD5, 0x51],
            deps: &[0],
        }, // 1: Clock / Oscillator
        CmdNode {
            bytes: &[0xCA, 0x7F],
            deps: &[0],
        }, // 2: Mux Ratio
        CmdNode {
            bytes: &[0xA8, 0x3F],
            deps: &[0],
        }, // 3: Set Mux Ratio (deprecated, but good to have)
        CmdNode {
            bytes: &[0xD3, 0x60],
            deps: &[0],
        }, // 4: Display Offset
        CmdNode {
            bytes: &[0x40, 0x00],
            deps: &[0],
        }, // 5: Display Start Line
        CmdNode {
            bytes: &[0xA1, 0x00],
            deps: &[0],
        }, // 6: Segment Re-map
        CmdNode {
            bytes: &[0xA0],
            deps: &[0],
        }, // 7: Segment Re-map (alternate)
        CmdNode {
            bytes: &[0xC8],
            deps: &[0],
        }, // 8: COM Output Scan
        CmdNode {
            bytes: &[0xAD, 0x8A],
            deps: &[0],
        }, // 9: Vpp
        CmdNode {
            bytes: &[0xD9, 0x22],
            deps: &[0],
        }, // 10: Pre-charge Period
        CmdNode {
            bytes: &[0xDB, 0x35],
            deps: &[0],
        }, // 11: VCOMH Deselect
        CmdNode {
            bytes: &[0x8D, 0x14],
            deps: &[0],
        }, // 12: Charge Pump Setting
        // 注: Display ON (0xAF) と Normal Display (0xA6) は、
        // 全ての設定が完了した後に実行されるため、最も多くのコマンドに依存するようにします。
        // ここでは、必須の設定である Pre-charge (10), VCOMH (11), Charge Pump (12) に依存させます。
        CmdNode {
            bytes: &[0xA6],
            deps: &[10, 11, 12],
        }, // 13: Normal Display
        CmdNode {
            bytes: &[0xAF],
            deps: &[10, 11, 12, 13],
        }, // 14: Display ON
    ];

    let explorer = Explorer::<15> {
        sequence: &explorer_cmds,
    };

    // ---- Explore ----
    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    if let Err(e) = run_single_sequence_explorer::<_, _, 15, 128>(
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
