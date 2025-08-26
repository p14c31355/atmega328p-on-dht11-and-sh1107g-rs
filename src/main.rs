#![no_std]
#![no_main]

use arduino_hal::i2c;
use arduino_hal::prelude::*;
use dvcdbg::prelude::*;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

// SH1107G 安全初期化コマンド群（依存順に並べる）
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },            // Display OFF
    CmdNode { bytes: &[0xDC, 0x00], deps: &[] },      // Display start line
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },      // Contrast
    CmdNode { bytes: &[0x20, 0x02], deps: &[] },      // Memory addressing mode
    CmdNode { bytes: &[0xA0], deps: &[] },            // Segment remap
    CmdNode { bytes: &[0xC0], deps: &[] },            // COM output dir
    CmdNode { bytes: &[0xA4], deps: &[] },            // Entire display ON
    CmdNode { bytes: &[0xA6], deps: &[] },            // Normal display
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

    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UnoWrapper(serial);

    let _ = writeln!(logger, "[log] Start SH1107G safe init");

    // 順序通りに I2C コマンドを送信
    for node in SH1107G_NODES {
        if let Err(e) = i2c.write(0x3C, node.bytes) {
            let _ = writeln!(logger, "[error] write failed: {:?}", e);
        }
    }

    let _ = writeln!(logger, "[oled] init sequence applied");

    loop {
        delay.delay_ms(1000u16);
    }
}
