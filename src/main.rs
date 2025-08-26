#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_io::Write;
use panic_abort as _; // panic-abortに変更した場合はこちらをコメントアウト

use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "\n[SH1107G Explorer Test]").ok();
    writeln!(serial, "").ok();

    // I2C 初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100000,
    );
    writeln!(serial, "[Info] I2Cの初期化が完了しました。").ok();

    // ---- SH1107G の候補初期化シーケンス ----
    let init_seq: [u8; 24] = [
        0xAE, // Display OFF
        0xDC, 0x00, // Display start line = 0
        0x81, 0x2F, // Contrast
        0x20, 0x02, // Memory addressing mode: page
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
    // 各コマンドの前に制御バイト 0x00 を追加
    // `CmdNode` { `bytes`: コマンド配列, `deps`: 依存関係インデックス }
    const NUM_CMDS: usize = 15;
    let cmds: [CmdNode; NUM_CMDS] = [
        CmdNode { bytes: &[0x00, 0xAE], deps: &[] },
        CmdNode { bytes: &[0x00, 0xDC, 0x00], deps: &[] },
        CmdNode { bytes: &[0x00, 0x81, 0x2F], deps: &[] },
        CmdNode { bytes: &[0x00, 0x20, 0x02], deps: &[] },
        CmdNode { bytes: &[0x00, 0xA0], deps: &[] },
        CmdNode { bytes: &[0x00, 0xC0], deps: &[] },
        CmdNode { bytes: &[0x00, 0xA4], deps: &[] },
        CmdNode { bytes: &[0x00, 0xA6], deps: &[] },
        CmdNode { bytes: &[0x00, 0xA8, 0x7F], deps: &[] },
        CmdNode { bytes: &[0x00, 0xD3, 0x60], deps: &[] },
        CmdNode { bytes: &[0x00, 0xD5, 0x51], deps: &[] },
        CmdNode { bytes: &[0x00, 0xD9, 0x22], deps: &[] },
        CmdNode { bytes: &[0x00, 0xDB, 0x35], deps: &[] },
        CmdNode { bytes: &[0x00, 0xAD, 0x8A], deps: &[] },
        CmdNode { bytes: &[0x00, 0xAF], deps: &[0] }, // ONコマンド(0xAF)はOFFコマンド(0xAE)に依存
    ];
    let explorer = Explorer::<NUM_CMDS> { sequence: &cmds };

    // ---- 探索実行 ----
    let _ = run_explorer::<_, _, NUM_CMDS, 24>(
        &explorer,
        &mut i2c,
        &mut serial,
        &init_seq,
        0x3C,
        LogLevel::Verbose
    );

    loop {}
}