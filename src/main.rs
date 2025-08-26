#![no_std]
#![no_main]

use arduino_hal::i2c;
use arduino_hal::prelude::*;
use dvcdbg::prelude::*;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

// SH1107G 安全初期化コマンド群（テスト用に最初の 3 コマンドだけ）
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },   // Display OFF
    CmdNode { bytes: &[0xDC, 0x00], deps: &[0xAE] }, // Display start line
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },     // Contrast
];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UnoWrapper(serial);

    let _ = writeln!(logger, "[log] Start SH1107G minimal init");

    // 1️⃣ I2C バススキャン（Quiet）
    scan_i2c(&mut i2c, &mut logger, LogLevel::Quiet);

    // 2️⃣ 初期化コマンド候補
    let init_candidates: &[u8] = &[0xAE, 0xDC, 0x81];

    // 3️⃣ 実機応答があるコマンドだけ抽出
    let successful_init = scan_init_sequence(&mut i2c, &mut logger, init_candidates, LogLevel::Quiet);

    let _ = writeln!(logger, "[scan] init sequence filtered: {} cmds", successful_init.len());

    // 4️⃣ Explorer 実行（Quiet）
    let explorer = Explorer { sequence: SH1107G_NODES };
    let _ = run_explorer::<_, _, 32>(
        &explorer,
        &mut i2c,
        &mut logger,
        &successful_init,
        0x3C,
        LogLevel::Quiet,
    );

    let _ = writeln!(logger, "[oled] minimal init applied");

    loop {
        delay.delay_ms(1000u16);
    }
}
