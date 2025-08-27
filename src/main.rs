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
    CmdNode { bytes: &[0xAE], deps: &[] },                // Display OFF
    CmdNode { bytes: &[0xD5, 0x51], deps: &[0] },        // Clock / Oscillator
    CmdNode { bytes: &[0xCA, 0x7F], deps: &[0] },        // Multiplex Ratio
    CmdNode { bytes: &[0xA2, 0x00], deps: &[0] },        // Display Offset
    CmdNode { bytes: &[0xA1, 0x00], deps: &[0] },        // Start Line
    CmdNode { bytes: &[0xA0], deps: &[0] },              // Segment Re-map
    CmdNode { bytes: &[0xC8], deps: &[0] },              // COM Output Scan
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[0] },        // Vpp
    CmdNode { bytes: &[0xD9, 0x22], deps: &[7] },        // Pre-charge
    CmdNode { bytes: &[0xDB, 0x35], deps: &[8] },        // VCOMH
    CmdNode { bytes: &[0x8D, 0x14], deps: &[7] },        // Charge Pump
    CmdNode { bytes: &[0xA6], deps: &[0] },              // Normal Display
    CmdNode { bytes: &[0xAF], deps: &[0, 10] },          // Display ON
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
