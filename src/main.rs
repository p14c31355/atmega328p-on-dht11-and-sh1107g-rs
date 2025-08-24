#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c, explorer::{Explorer, CmdNode}};
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const PAGE_HEIGHT: usize = 8;
const COLUMN_OFFSET: usize = 2; // SH1107G マージン
const I2C_MAX_WRITE: usize = 32; // UNO I2C バッファ制限

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G auto-init test").unwrap();

    // I2C デバイススキャン
    scan_i2c(&mut i2c, &mut serial_wrapper, dvcdbg::scanner::LogLevel::Quiet);

    let address = 0x3C;

    // コマンド依存関係を定義
    let nodes = &[
        CmdNode { cmd: 0xAE, deps: &[] },          // Display OFF
        CmdNode { cmd: 0x81, deps: &[0xAE] },      // Contrast
        CmdNode { cmd: 0x2F, deps: &[0x81] },      // Contrast value
        CmdNode { cmd: 0xA1, deps: &[0xAE] },      // Start line
        CmdNode { cmd: 0x00, deps: &[0xA1] },      // Start line value
        CmdNode { cmd: 0xD3, deps: &[0xAE] },      // Display offset
        CmdNode { cmd: 0x60, deps: &[0xD3] },      // Offset value
        CmdNode { cmd: 0xAF, deps: &[0xAE] },      // Display ON
    ];

    let explorer = Explorer { sequence: nodes };

    // 初期化コマンド順序探索と実行
    explorer.explore(&mut i2c, &mut serial_wrapper).ok();

    writeln!(serial_wrapper, "[oled] init sequence applied").unwrap();

    // 十字描画バッファ
    let mut page_buf: [u8; DISPLAY_WIDTH] = [0; DISPLAY_WIDTH];

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        // 横線
        for x in 0..DISPLAY_WIDTH {
            page_buf[x] = if page == DISPLAY_HEIGHT / 2 / PAGE_HEIGHT { 0xFF } else { 0x00 };
        }

        // ページ・列アドレス設定
        i2c.write(address, &[0x00, 0xB0 + page as u8]).ok();
        i2c.write(address, &[0x00, 0x00 + COLUMN_OFFSET as u8]).ok();
        i2c.write(address, &[0x00, 0x10]).ok();

        // データ送信（32バイトずつ分割）
        for chunk in page_buf.chunks(I2C_MAX_WRITE - 1) {
            let mut data: heapless::Vec<u8, I2C_MAX_WRITE> = heapless::Vec::new();
            data.push(0x40).ok(); // データ先頭バイト
            data.extend_from_slice(chunk).ok();
            i2c.write(address, &data).ok();
        }
    }

    // 縦線
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        page_buf = [0; DISPLAY_WIDTH];
        for y_in_page in 0..PAGE_HEIGHT {
            let global_y = page * PAGE_HEIGHT + y_in_page;
            if global_y == DISPLAY_HEIGHT / 2 {
                page_buf[DISPLAY_WIDTH / 2] = 0xFF;
            }
        }

        i2c.write(address, &[0x00, 0xB0 + page as u8]).ok();
        i2c.write(address, &[0x00, 0x00 + COLUMN_OFFSET as u8]).ok();
        i2c.write(address, &[0x00, 0x10]).ok();

        for chunk in page_buf.chunks(I2C_MAX_WRITE - 1) {
            let mut data: heapless::Vec<u8, I2C_MAX_WRITE> = heapless::Vec::new();
            data.push(0x40).ok();
            data.extend_from_slice(chunk).ok();
            i2c.write(address, &data).ok();
        }
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}
