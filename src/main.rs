#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use embedded_io::Write;

use dvcdbg::prelude::*;

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
        100000,
    );

    //let _found = scan_i2c(&mut i2c, &mut serial, LogLevel::Verbose);
    // ---- SH1107G の候補初期化シーケンス ----
    // データシート準拠の代表的なコマンド群
    let init_seq: [u8; 24] = [
    0xAE, // Display OFF
    0xDC, 0x00, // Display start line = 0
    0x81, 0x2F, // Contrast
    0x20,  0x02, // Memory addressing mode: page
    0xA0, // Segment remap normal
    0xC0, // Common output scan direction normal
    0xA4, // Entire display ON from RAM
    0xA6, // Normal display
    0xA8, 0x7F, // Multiplex ratio 128
    0xD3, 0x60, // Display offset
    0xD5, 0x51, // Oscillator frequency
    0xD9, 0x22, // Pre-charge period
    0xDB, 0x35, // VCOM deselect level
    0xAD, 0x8A, // DC-DC control
    0xAF,       // Display ON
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
    let _ = run_explorer::<_, _, 8, 32>(&explorer, &mut i2c, &mut serial, &init_seq, 0x00, LogLevel::Quiet);

    loop {}
}

/*0xAE
 0xDC
 0x00
 0x81
 0x2F
 0x20
 0x02
 0xA0
 0xC0
 0xA4
 0xA6
 0xA8
 0x7F
 0xD3
 0x60
 0xD5
 0x51
 0xD9
 0x22
 0xDB
 0x35
 0xAD
 0x8A
 0xAF */