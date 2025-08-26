#![no_std]
#![no_main]

use arduino_hal::i2c;
use arduino_hal::prelude::*;
use dvcdbg::prelude::*;
use dvcdbg::scanner::{scan_i2c, scan_init_sequence, run_explorer, LogLevel};
use embedded_io::Write;
use panic_halt as _;

// シリアルラッパー生成マクロ
adapt_serial!(UnoWrapper);

// SH1107G 安全初期化コマンド群
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },            // Display OFF
    CmdNode { bytes: &[0xDC, 0x00], deps: &[0xAE] },  // Display start line
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },      // Contrast
    CmdNode { bytes: &[0x20, 0x02], deps: &[] },      // Memory addressing mode
    CmdNode { bytes: &[0xA0], deps: &[] },            // Segment remap
    CmdNode { bytes: &[0xC0], deps: &[0xA0] },        // COM output dir
    CmdNode { bytes: &[0xA4], deps: &[0xC0] },        // Entire display ON
    CmdNode { bytes: &[0xA6], deps: &[0xA4] },        // Normal display
    CmdNode { bytes: &[0xA8, 0x7F], deps: &[] },      // Multiplex ratio
    CmdNode { bytes: &[0xD3, 0x60], deps: &[] },      // Display offset
    CmdNode { bytes: &[0xD5, 0x51], deps: &[] },      // Oscillator
    CmdNode { bytes: &[0xD9, 0x22], deps: &[] },      // Pre-charge
    CmdNode { bytes: &[0xDB, 0x35], deps: &[] },      // VCOM level
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[] },      // DC-DC control
    CmdNode { bytes: &[0xAF], deps: &[] },            // Display ON
];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // シリアル初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    let _ = writeln!(serial_wrapper, "[log] Start SH1107G safe init");

    // Explorer 構築
    let explorer = Explorer { sequence: SH1107G_NODES };

    // I2C バススキャン
    scan_i2c(&mut i2c, &mut serial_wrapper, LogLevel::Verbose);

    // 初期化候補コマンド
    let init_candidates: &[u8] = &[
        0xAE, 0xDC, 0x81, 0x20, 0xA0, 0xC0, 0xA4, 0xA6,
        0xA8, 0xD3, 0xD5, 0xD9, 0xDB, 0xAD, 0xAF,
    ];

    // 実機応答のあったコマンドのみ抽出
    let successful_init = scan_init_sequence(
        &mut i2c,
        &mut serial_wrapper,
        init_candidates,
        LogLevel::Verbose,
    );
    let _ = writeln!(serial_wrapper, "[scan] init sequence filtered: {} cmds", successful_init.len());

    // Explorer 実行
    let _ = run_explorer::<_, _, 15>(
        &explorer,
        &mut i2c,
        &mut serial_wrapper,
        &successful_init,
        0x3C, // デバイスアドレス仮
        LogLevel::Quiet,
    );
    let _ = writeln!(serial_wrapper, "[oled] init sequence applied");

    loop {
        delay.delay_ms(1000u16);
    }
}
