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
const COLUMN_OFFSET: usize = 2; // SH1107G の左右マージン

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

    // SH1107G 初期化（簡略版）
    let address = 0x3C;
    let init_sequence: &[u8] = &[
        0xAE,             // Display OFF
        0xDC, 0x00,       // Display start line
        0x81, 0x2F,       // Contrast
        0x20,             // Memory addressing mode: page
        0xA0,             // Segment remap normal
        0xC0,             // Common output scan direction normal
        0xA4,             // Entire display ON from RAM
        0xA6,             // Normal display
        0xA8, 0x7F,       // Multiplex ratio 128
        0xD3, 0x60,       // Display offset
        0xD5, 0x51,       // Oscillator frequency
        0xD9, 0x22,       // Pre-charge period
        0xDB, 0x35,       // VCOM deselect level
        0xAD, 0x8A,       // DC-DC control
        0xDA, 0x12,       // COM pins
        0xAF,             // Display ON
    ];

    for cmd in init_sequence {
        i2c.write(address, &[*cmd]).ok();
        delay.delay_ms(10u16); // バス安定化
    }
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // 十字描画バッファ（ページ単位）
    let mut page_buf: [u8; DISPLAY_WIDTH + COLUMN_OFFSET] = [0; DISPLAY_WIDTH + COLUMN_OFFSET];

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        // 横線
        for x in 0..DISPLAY_WIDTH {
            page_buf[COLUMN_OFFSET + x] = if page == (DISPLAY_HEIGHT / 2 / PAGE_HEIGHT) { 0xFF } else { 0x00 };
        }
        i2c.write(address, &[0xB0 + page as u8]).ok(); // ページアドレス
        i2c.write(address, &[0x00 + COLUMN_OFFSET as u8]).ok(); // 列下位
        i2c.write(address, &[0x10]).ok(); // 列上位
        i2c.write(address, &[0x40]).ok(); // データ開始
        i2c.write(address, &page_buf).ok();
    }

    // 縦線（ページ単位）
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        page_buf = [0; DISPLAY_WIDTH + COLUMN_OFFSET];
        for y_in_page in 0..PAGE_HEIGHT {
            let global_y = page * PAGE_HEIGHT + y_in_page;
            if global_y == DISPLAY_HEIGHT / 2 {
                page_buf[COLUMN_OFFSET + DISPLAY_WIDTH / 2] = 0xFF;
            }
        }
        i2c.write(address, &[0xB0 + page as u8]).ok();
        i2c.write(address, &[0x00 + COLUMN_OFFSET as u8]).ok();
        i2c.write(address, &[0x10]).ok();
        i2c.write(address, &[0x40]).ok();
        i2c.write(address, &page_buf).ok();
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}
