#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
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
    writeln!(serial_wrapper, "[log] Start SH1107G test").unwrap();

    // I2C デバイススキャン
    scan_i2c(&mut i2c, &mut serial_wrapper);

    let address = 0x3C;

    // 初期化コマンド（1コマンドずつ 0x00 を付与）
    let init_sequence: &[&[u8]] = &[
        &[0xAE],             // Display OFF
        &[0xDC, 0x00],       // Display start line
        &[0x81, 0x2F],       // Contrast
        &[0x20, 0x02],       // Memory addressing mode: page
        &[0xA0],             // Segment remap
        &[0xC0],             // Com scan direction
        &[0xA4],             // Entire display ON from RAM
        &[0xA6],             // Normal display
        &[0xA8, 0x7F],       // Multiplex ratio
        &[0xD3, 0x60],       // Display offset
        &[0xD5, 0x51],       // Oscillator frequency
        &[0xD9, 0x22],       // Pre-charge period
        &[0xDB, 0x35],       // VCOM deselect level
        &[0xAD, 0x8A],       // DC-DC control
        &[0xAF],             // Display ON
    ];

    for cmd in init_sequence {
        let mut payload: heapless::Vec<u8, 8> = heapless::Vec::new();
        payload.push(0x00).ok(); // コマンドバイト
        payload.extend_from_slice(cmd).ok();
        i2c.write(address, &payload).ok();
        delay.delay_ms(5u16);
    }
    writeln!(serial_wrapper, "[oled] init done").unwrap();

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
                page_buf[DISPLAY_WIDTH / 2] = 0x00;
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

    loop {} 
}
