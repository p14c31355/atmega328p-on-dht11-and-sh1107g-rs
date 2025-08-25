#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
    explorer::{Explorer, CmdNode, CmdExecutor},
    scanner::LogLevel,
};
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

// PrefixExecutor をそのまま使えるようにコピー
struct PrefixExecutor {
    prefix: u8,
    init_sequence: heapless::Vec<u8, 64>,
    buffer: heapless::Vec<u8, 64>,
}

impl PrefixExecutor {
    fn new(prefix: u8, init_sequence: heapless::Vec<u8, 64>) -> Self {
        Self {
            prefix,
            init_sequence,
            buffer: heapless::Vec::new(),
        }
    }
}

impl<I2C> CmdExecutor<I2C> for PrefixExecutor
where
    I2C: dvcdbg::compat::I2cCompat,
    <I2C as dvcdbg::compat::I2cCompat>::Error: dvcdbg::compat::HalErrorExt,
{
    fn exec(&mut self, i2c: &mut I2C, addr: u8, cmd: &[u8]) -> bool {
        // init sequence before explorer commands
        for &c in self.init_sequence.iter() {
            let command = [self.prefix, c];
            if i2c.write(addr, &command).is_err() {
                return false;
            }
        }

        self.buffer.clear();
        if self.buffer.push(self.prefix).is_err() || self.buffer.extend_from_slice(cmd).is_err() {
            return false;
        }

        i2c.write(addr, &self.buffer).is_ok()
    }
}

// SH1107G 初期化コマンド
const SH1107G_NODES: &[CmdNode<'_>] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },         // Display OFF
    CmdNode { bytes: &[0xDC, 0x00], deps: &[] },   // Display start line = 0
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },   // Contrast
    CmdNode { bytes: &[0x20, 0x02], deps: &[] },   // Memory addressing mode: page
    CmdNode { bytes: &[0xA0], deps: &[] },         // Segment remap normal
    CmdNode { bytes: &[0xC0], deps: &[] },         // Common output scan direction normal
    CmdNode { bytes: &[0xA4], deps: &[] },         // Entire display ON from RAM
    CmdNode { bytes: &[0xA6], deps: &[] },         // Normal display
    CmdNode { bytes: &[0xA8, 0x7F], deps: &[] },   // Multiplex ratio 128
    CmdNode { bytes: &[0xD3, 0x60], deps: &[] },   // Display offset
    CmdNode { bytes: &[0xD5, 0x51], deps: &[] },   // Oscillator frequency
    CmdNode { bytes: &[0xD9, 0x22], deps: &[] },   // Pre-charge period
    CmdNode { bytes: &[0xDB, 0x35], deps: &[] },   // VCOM deselect level
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[] },   // DC-DC control
    CmdNode { bytes: &[0xAF], deps: &[] },         // Display ON
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
    let explorer = Explorer { sequence: SH1107G_NODES };

    // 初期化シーケンスを scan_init_sequence で取得
    let init_sequence = dvcdbg::scanner::scan_init_sequence(
        &mut i2c,
        &mut serial_wrapper,
        &[0xAE, 0xDC, 0x81, 0x20, 0xA0, 0xC0, 0xA4, 0xA6, 0xA8, 0xD3, 0xD5, 0xD9, 0xDB, 0xAD, 0xAF],
        LogLevel::Verbose,
    );

    let mut executor = PrefixExecutor::new(0x00, init_sequence);

    // Explorer 実行
    let _ = explorer.explore(&mut i2c, &mut serial_wrapper, &mut executor);

    let _ = writeln!(serial_wrapper, "[oled] Init sequence applied");

    loop {
        delay.delay_ms(1000u16);
    }
}
