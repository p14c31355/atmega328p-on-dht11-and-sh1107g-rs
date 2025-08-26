#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use core::fmt::Write;

use dvcdbg::prelude::*;
use dvcdbg::explorer::{Explorer, CmdNode};
use dvcdbg::scanner::run_explorer;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    writeln!(serial, "\n[SH1107G Explorer Test]").ok();

    // I2C 初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,
    );

    // ---- SH1107G の候補初期化シーケンス ----
    // データシート準拠の代表的なコマンド群
    let init_seq: [u8; 5] = [
        0xAE, // Display OFF
        0xA1, // Segment remap
        0xA6, // Normal display
        0xA8, // Multiplex ratio
        0xAF, // Display ON
    ];

    // ---- Explorer 用コマンドノード定義 ----
    // CmdNode { bytes: コマンド配列, deps: 依存関係インデックス }
    let cmds: [CmdNode; 5] = [
        CmdNode { bytes: &[0xAE], deps: &[] }, // OFF
        CmdNode { bytes: &[0xA1], deps: &[] }, // Segment remap
        CmdNode { bytes: &[0xA6], deps: &[] }, // Normal display
        CmdNode { bytes: &[0xA8], deps: &[] }, // Multiplex
        CmdNode { bytes: &[0xAF], deps: &[0] }, // ON (OFFの後)
    ];
    let explorer = Explorer::<8> { sequence: &cmds };

    // ---- 探索実行 ----
    // prefix = 0x00 → コマンドモードで送信
    let _ = run_explorer::<_, _, 8, 32>(&explorer, &mut i2c, &mut serial, &init_seq, 0x3C, LogLevel::Verbose);

    loop {}
}