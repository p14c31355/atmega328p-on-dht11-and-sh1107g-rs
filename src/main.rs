#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
    scanner::scan_i2c,
    explorer::{Explorer, CmdNode, CmdExecutor, ExplorerError}
};
use embedded_io::Write;
use panic_halt as _;
use heapless::Vec;

adapt_serial!(UnoWrapper);

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const PAGE_HEIGHT: usize = 8;
const COLUMN_OFFSET: usize = 2;
const I2C_MAX_WRITE: usize = 32;

// SH1107G_INIT_CMDS の内容を元に、新しい CmdNode の定義に合わせる
const SH1107G_NODES: &[CmdNode] = &[
    CmdNode { bytes: Vec::from_slice(&[0xAE]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xDC, 0x00]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0x81, 0x2F]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0x20, 0x02]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xA0]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xC0]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xA4]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xA6]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xA8, 0x7F]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xD3, 0x60]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xD5, 0x51]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xD9, 0x22]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xDB, 0x35]).unwrap(), deps: Vec::new() },
    CmdNode { bytes: Vec::from_slice(&[0xAD, 0x8A]).unwrap(), deps: Vec::new() },
    // Display ON コマンドは Display OFF に依存
    CmdNode { bytes: Vec::from_slice(&[0xAF]).unwrap(), deps: Vec::from_slice(&[0xAE]).unwrap() },
];

struct MyExecutor;
impl<I2C> CmdExecutor<I2C> for MyExecutor
where
    I2C: arduino_hal::i2c::I2c
{
    fn exec(&mut self, i2c: &mut I2C, addr: u8, cmd: &[u8]) -> bool {
        // プロトコル固有のロジック: コマンド送信時に 0x00 を前置する
        let mut buffer = Vec::<u8, 33>::new();
        buffer.push(0x00).ok();
        buffer.extend_from_slice(cmd).ok();

        i2c.write(addr, &buffer).is_ok()
    }
}


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
    writeln!(serial_wrapper, "[log] Start SH1107G auto-init test").unwrap();

    scan_i2c(&mut i2c, &mut serial_wrapper, dvcdbg::scanner::LogLevel::Quiet);

    let address = 0x3C;

    let explorer = Explorer { sequence: Vec::from_slice(SH1107G_NODES).unwrap() };
    let mut executor = MyExecutor;
    explorer.explore(&mut i2c, &mut serial_wrapper, &mut executor).ok();

    writeln!(serial_wrapper, "[oled] init sequence applied").unwrap();

    // --- 動作確認: 中央クロス表示 ---
    let mut page_buf: [u8; DISPLAY_WIDTH] = [0; DISPLAY_WIDTH];

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        for x in 0..DISPLAY_WIDTH {
            page_buf[x] = if page == DISPLAY_HEIGHT / 2 / PAGE_HEIGHT { 0xFF } else { 0x00 };
        }
        // 以下、i2c.writeへの呼び出しは全てexecutorを介するように変更
        executor.exec(&mut i2c, address, &[0xB0 + page as u8]).ok();
        executor.exec(&mut i2c, address, &[0x00 + COLUMN_OFFSET as u8]).ok();
        executor.exec(&mut i2c, address, &[0x10]).ok();

        // データの書き込みもexecutorを介する
        for chunk in page_buf.chunks(I2C_MAX_WRITE - 1) {
            let mut data = Vec::<u8, I2C_MAX_WRITE>::new();
            data.extend_from_slice(chunk).ok();
            executor.exec(&mut i2c, address, &data).ok();
        }
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}