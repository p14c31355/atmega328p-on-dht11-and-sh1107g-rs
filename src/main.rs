#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
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
    CmdNode { bytes: &[0xAE], deps: &[] },          // 依存関係なし (最初)
    CmdNode { bytes: &[0xDC, 0x00], deps: &[0xAE] },  // 直前の0xAEに依存
    CmdNode { bytes: &[0x81, 0x2F], deps: &[0xDC] },  // 直前の0xDCに依存
    CmdNode { bytes: &[0x20, 0x02], deps: &[0x81] },  // 直前の0x81に依存
    CmdNode { bytes: &[0xA0], deps: &[0x20] },        // 直前の0x20に依存
    CmdNode { bytes: &[0xC0], deps: &[0xA0] },        // 直前の0xA0に依存
    CmdNode { bytes: &[0xA4], deps: &[0xC0] },        // 直前の0xC0に依存
    CmdNode { bytes: &[0xA6], deps: &[0xA4] },        // 直前の0xA4に依存
    CmdNode { bytes: &[0xA8, 0x7F], deps: &[0xA6] },  // 直前の0xA6に依存
    CmdNode { bytes: &[0xD3, 0x60], deps: &[0xA8] },  // 直前の0xA8に依存
    CmdNode { bytes: &[0xD5, 0x51], deps: &[0xD3] },  // 直前の0xD3に依存
    CmdNode { bytes: &[0xD9, 0x22], deps: &[0xD5] },  // 直前の0xD5に依存
    CmdNode { bytes: &[0xDB, 0x35], deps: &[0xD9] },  // 直前の0xD9に依存
    CmdNode { bytes: &[0xAD, 0x8A], deps: &[0xDB] },  // 直前の0xDBに依存
    CmdNode { bytes: &[0xAF], deps: &[0xAE] },        // 0xAFは0xAEにのみ依存
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
        writeln!(serial_wrapper, "[log] Start SH1107G auto-init test failed").unwrap();
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