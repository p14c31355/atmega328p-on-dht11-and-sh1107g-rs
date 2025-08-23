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
const COLUMN_OFFSET: usize = 2;
const PAGE_BYTES: usize = (DISPLAY_WIDTH + COLUMN_OFFSET) / 64 * 64; // 64バイト境界に切り上げ

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(dp.TWI, pins.a4.into_pull_up_input(), pins.a5.into_pull_up_input(), 100_000);

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G test").unwrap();

    let address = 0x3C;

    // SH1107G 初期化
    let init_sequence: &[(u8, &[u8])] = &[
        (0x00, &[0xAE]),             // Display OFF
        (0x00, &[0xDC, 0x00]),       // Display start line
        (0x00, &[0x81, 0x2F]),       // Contrast
        (0x00, &[0x20, 0x02]),       // Memory addressing mode: page
        (0x00, &[0xA0]),             // Segment remap normal
        (0x00, &[0xC0]),             // Common output scan direction normal
        (0x00, &[0xA4]),             // Entire display ON from RAM
        (0x00, &[0xA6]),             // Normal display
        (0x00, &[0xA8, 0x7F]),       // Multiplex ratio
        (0x00, &[0xD3, 0x60]),       // Display offset
        (0x00, &[0xD5, 0x51]),       // Oscillator frequency
        (0x00, &[0xD9, 0x22]),       // Pre-charge period
        (0x00, &[0xDB, 0x35]),       // VCOM deselect level
        (0x00, &[0xAD, 0x8A]),       // DC-DC control
        (0x00, &[0xAF]),             // Display ON
    ];

    for (ctrl, cmds) in init_sequence {
        let mut payload: heapless::Vec<u8, 16> = heapless::Vec::new();
        payload.push(*ctrl).ok();
        payload.extend_from_slice(cmds).ok();
        i2c.write(address, &payload).ok();
        delay.delay_ms(5u16);
    }
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // 全ページバッファクリア
    let mut buffer: [u8; DISPLAY_HEIGHT * (DISPLAY_WIDTH + COLUMN_OFFSET) / 8] = [0; DISPLAY_HEIGHT * (DISPLAY_WIDTH + COLUMN_OFFSET) / 8];

    // 十字描画
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let byte_index = (x + COLUMN_OFFSET) + (y / 8) * (DISPLAY_WIDTH + COLUMN_OFFSET);
            let bit_mask = 1 << (y % 8);
            if y == DISPLAY_HEIGHT / 2 || x == DISPLAY_WIDTH / 2 {
                buffer[byte_index] |= bit_mask;
            }
        }
    }

    // ページごとに 64 バイト単位で送信
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        i2c.write(address, &[0xB0 + page as u8]).ok(); // ページアドレス
        i2c.write(address, &[0x00 + COLUMN_OFFSET as u8]).ok(); // 列下位
        i2c.write(address, &[0x10]).ok(); // 列上位

        let start = page * (DISPLAY_WIDTH + COLUMN_OFFSET);
        let end = start + DISPLAY_WIDTH + COLUMN_OFFSET;
        let page_slice = &buffer[start..end];

        for chunk in page_slice.chunks(64) {
            let mut payload: heapless::Vec<u8, 64> = heapless::Vec::new();
            payload.extend_from_slice(chunk).ok();
            i2c.write(address, &[0x40]).ok(); // データ開始
            i2c.write(address, &payload).ok();
        }
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}
