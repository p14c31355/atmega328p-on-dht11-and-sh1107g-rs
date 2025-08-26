#![no_std]
#![no_main]

use arduino_hal::i2c;
use arduino_hal::prelude::*;
use dvcdbg::prelude::*;

use dvcdbg::scanner::run_explorer;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

// SH1107G 安全初期化コマンド（順番に送る）
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode {
        bytes: &[0xAE],
        deps: &[],
    }, // Display OFF
    CmdNode {
        bytes: &[0xDC, 0x00],
        deps: &[0xAE],
    }, // Display start line = 0
    CmdNode {
        bytes: &[0x81, 0x2F],
        deps: &[],
    }, // Contrast
    CmdNode {
        bytes: &[0x20, 0x02],
        deps: &[],
    }, // Memory addressing mode: page
    CmdNode {
        bytes: &[0xA0],
        deps: &[],
    }, // Segment remap normal
    CmdNode {
        bytes: &[0xC0],
        deps: &[0xA0],
    }, // Common output scan direction normal
    CmdNode {
        bytes: &[0xA4],
        deps: &[0xC0],
    }, // Entire display ON from RAM
    CmdNode {
        bytes: &[0xA6],
        deps: &[0xA4],
    }, // Normal display
    CmdNode {
        bytes: &[0xA8, 0x7F],
        deps: &[],
    }, // Multiplex ratio 128
    CmdNode {
        bytes: &[0xD3, 0x60],
        deps: &[],
    }, // Display offset
    CmdNode {
        bytes: &[0xD5, 0x51],
        deps: &[],
    }, // Oscillator frequency
    CmdNode {
        bytes: &[0xD9, 0x22],
        deps: &[],
    }, // Pre-charge period
    CmdNode {
        bytes: &[0xDB, 0x35],
        deps: &[],
    }, // VCOM deselect level
    CmdNode {
        bytes: &[0xAD, 0x8A],
        deps: &[],
    }, // DC-DC control
    CmdNode {
        bytes: &[0xAF],
        deps: &[],
    }, // Display ON
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
    let mut serial_wrapper = UnoWrapper(serial);

    let _ = writeln!(serial_wrapper, "[log] Start SH1107G safe init");

    // Explorer 構築
    let explorer = Explorer {
        sequence: SH1107G_NODES,
    };

    // Explorer 実行
    // The `run_explorer` function internally creates a `PrefixExecutor`
    // scan_init_sequence を省略して固定配列を直接渡す
    let init_sequence: &[u8] = &[
        0xAE, 0xDC, 0x81, 0x20, 0xA0, 0xC0, 0xA4, 0xA6, 0xA8, 0xD3, 0xD5, 0xD9, 0xDB, 0xAD, 0xAF,
    ];

    // run_explorer に渡す際に clone をやめて Vec に変換も最小容量に
    let init_vec: heapless::Vec<u8, 16> = init_sequence.iter().copied().collect();

    // Uno の RAM に優しい軽量版
    let _ = run_explorer(
        &explorer,
        &mut i2c,
        &mut serial_wrapper,
        &init_vec,
        0x00,
        LogLevel::Verbose,
    );

    let _ = writeln!(serial_wrapper, "[oled] init sequence applied");

    loop {
        delay.delay_ms(1000u16);
    }
}
