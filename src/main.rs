#![no_std]
#![no_main]

use arduino_hal::i2c;
use arduino_hal::prelude::*;
use arduino_hal::default_serial;
use dvcdbg::prelude::*;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

// SH1107G 安全初期化コマンド群
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },
    CmdNode { bytes: &[0xDC, 0x00], deps: &[0xAE] },
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },
    CmdNode { bytes: &[0x20, 0x02], deps: &[] },
    CmdNode { bytes: &[0xA0], deps: &[] },
    CmdNode { bytes: &[0xC0], deps: &[0xA0] },
    CmdNode { bytes: &[0xA4], deps: &[0xC0] },
    CmdNode { bytes: &[0xA6], deps: &[0xA4] },
    CmdNode { bytes: &[0xA8, 0x7F], deps: &[] },
    CmdNode { bytes: &[0xD3, 0x60], deps: &[] },
    CmdNode { bytes: &[0xD5, 0x51], deps: &[] },
    CmdNode { bytes: &[0xD9, 0x22], deps: &[] },
    CmdNode { bytes: &[0xDB, 0x35], deps: &[] },
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[] },
    CmdNode { bytes: &[0xAF], deps: &[] },
];

pub const SH1107G_INIT_CMDS: &[u8] = &[
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

    let serial = default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);

    let _ = writeln!(serial_wrapper, "[log] Start SH1107G safe init");

    // Explorer 構築
    //let explorer = Explorer { sequence: SH1107G_NODES };

    // I2C バススキャン
    scan_init_sequence(&mut i2c, &mut serial_wrapper, SH1107G_INIT_CMDS, LogLevel::Quiet);
/*
    // 初期化コマンド候補
    let init_candidates: &[u8] = &[
        0xAE, 0xDC, 0x81, 0x20, 0xA0, 0xC0, 0xA4, 0xA6,
        0xA8, 0xD3, 0xD5, 0xD9, 0xDB, 0xAD, 0xAF,
    ];

    // 実際に応答があったコマンドだけを抽出
    let successful_init = scan_init_sequence(&mut i2c, &mut serial_wrapper, init_candidates, LogLevel::Quiet);

    let _ = writeln!(serial_wrapper, "[scan] init sequence filtered:");
    let _ = write_bytes_hex_prefixed(&mut serial_wrapper, &successful_init);
    let _ = writeln!(serial_wrapper, "");

    // Explorer 実行
    let _ = run_explorer::<_, _, 128>(
        &explorer,
        &mut i2c,
        &mut serial_wrapper,
        &successful_init,
        0x3C,
        LogLevel::Quiet,
    );

    let _ = writeln!(serial_wrapper, "[oled] init sequence applied");
 */
    loop {
        delay.delay_ms(1000u16);
    }
}
