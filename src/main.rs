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
    let explorer_cmds: [CmdNode; 13] = [
    CmdNode { bytes: &[0x00, 0xAE], deps: &[] },        // Display off
    CmdNode { bytes: &[0x00, 0xD5, 0x51], deps: &[0] }, // Set Display Clock Divide Ratio/Oscillator Frequency
    CmdNode { bytes: &[0x00, 0xCA, 0x7F], deps: &[0] }, // Set Multiplex Ratio
    CmdNode { bytes: &[0x00, 0xA2, 0x00], deps: &[0] }, // Set Display Offset
    CmdNode { bytes: &[0x00, 0xA1, 0x00], deps: &[0] }, // Set Display Start Line
    CmdNode { bytes: &[0x00, 0xA0], deps: &[0] },       // Set Segment Re-map
    CmdNode { bytes: &[0x00, 0xC8], deps: &[0] },       // Set COM Output Scan Direction
    CmdNode { bytes: &[0x00, 0xAD, 0x8A], deps: &[0] }, // Set Vpp
    CmdNode { bytes: &[0x00, 0xD9, 0x22], deps: &[0] }, // Set Pre-charge Period
    CmdNode { bytes: &[0x00, 0xDB, 0x35], deps: &[0] }, // Set VCOMH Deselect Level
    CmdNode { bytes: &[0x00, 0x8D, 0x14], deps: &[0] }, // Set Charge Pump
    CmdNode { bytes: &[0x00, 0xA6], deps: &[0] },       // Normal Display
    CmdNode { bytes: &[0x00, 0xAF], deps: &[0] },       // Display on
];


    let explorer = Explorer::<13> { sequence: &explorer_cmds };

    // ---- Explorer 実行 ----
    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();
    if let Err(e) = run_explorer::<_, _, 13, 128>(
        &explorer,
        &mut i2c,
        &mut serial,
        &[],   // 初期化シーケンスは不要（CmdNode.bytes に全バイトを含めたため）
        0x00,  // プレフィックスは CmdNode.bytes に含め済み
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
