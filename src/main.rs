#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
    scanner::scan_i2c,
    explorer::{Explorer, CmdNode, CmdExecutor}
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

// SH1107G_INIT_CMDSを、本来の多バイトコマンドとして定義
const SH1107G_NODES: &[CmdNode] = &[
    CmdNode { bytes: &[0xAE], deps: &[] },          // Display OFF
    CmdNode { bytes: &[0xDC, 0x00], deps: &[] },    // Display start line
    CmdNode { bytes: &[0x81, 0x2F], deps: &[] },    // Contrast
    CmdNode { bytes: &[0x20, 0x02], deps: &[] },    // Memory addressing mode
    CmdNode { bytes: &[0xA0], deps: &[] },          // Segment remap
    CmdNode { bytes: &[0xC0], deps: &[] },          // Common output scan direction
    CmdNode { bytes: &[0xA4], deps: &[] },          // Entire display ON from RAM
    CmdNode { bytes: &[0xA6], deps: &[] },          // Normal display
    CmdNode { bytes: &[0xA8, 0x7F], deps: &[] },    // Multiplex ratio
    CmdNode { bytes: &[0xD3, 0x60], deps: &[] },    // Display offset
    CmdNode { bytes: &[0xD5, 0x51], deps: &[] },    // Oscillator frequency
    CmdNode { bytes: &[0xD9, 0x22], deps: &[] },    // Pre-charge period
    CmdNode { bytes: &[0xDB, 0x35], deps: &[] },    // VCOM deselect level
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[] },    // DC-DC
    // Display ON コマンドは Display OFF に依存
    CmdNode { bytes: &[0xAF], deps: &[0xAE] },      // Display ON
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
    
    if writeln!(serial_wrapper, "[log] Start SH1107G auto-init test").is_err() {
        // 必要に応じてエラーハンドリング
    }
    delay.delay_ms(1u16); 

    delay.delay_ms(1u16);

    let explorer = Explorer { sequence: SH1107G_NODES };
    let mut executor = MyExecutor;
    
    match explorer.explore(&mut i2c, &mut serial_wrapper, &mut executor) {
        Ok(()) => {
            if writeln!(serial_wrapper, "[oled] init sequence applied").is_err() {
                 // 必要に応じてエラーハンドリング
            }
        },
        Err(e) => {
            if writeln!(serial_wrapper, "[error] explorer failed: {:?}", e).is_err() {
                 // 必要に応じてエラーハンドリング
            }
        },
    }
    delay.delay_ms(1u16); 

    loop { delay.delay_ms(1000u16); }
}