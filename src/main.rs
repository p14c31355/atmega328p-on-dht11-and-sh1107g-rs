#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
    explorer::{CmdNode, CmdExecutor}
};
use embedded_io::Write;
use panic_halt as _;
use heapless::Vec;

adapt_serial!(UnoWrapper);

struct MyExecutor;
impl<I2C> CmdExecutor<I2C> for MyExecutor
where
    I2C: dvcdbg::compat::I2cCompat,
{
    fn exec(&mut self, i2c: &mut I2C, addr: u8, cmd: &[u8]) -> bool {
        let mut buffer = Vec::<u8, 33>::new();
        if buffer.push(0x00).is_err() || buffer.extend_from_slice(cmd).is_err() {
            return false;
        }
        i2c.write(addr, &buffer).is_ok()
    }
}

// SH1107G 安全初期化コマンド（順番に送る）
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

    let mut executor = MyExecutor;

    for (i, cmd) in SH1107G_NODES.iter().enumerate() {
    // ここで Explorer か Validator に渡して順序チェック
    if cmd.bytes.first() == Some(&0xAF) && i < SH1107G_NODES.len() - 1 {
        writeln!(serial_wrapper, "[oled] Display ON issued too early at index {}", i);
    }
}


    let _ = writeln!(serial_wrapper, "[oled] init sequence applied");

    loop {
        delay.delay_ms(1000u16);
    }
}