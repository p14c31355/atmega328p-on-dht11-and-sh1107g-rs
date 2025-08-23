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

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // -------------------------
    // I2C 初期化
    // -------------------------
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // -------------------------
    // UART 初期化
    // -------------------------
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start Uno + SH1107G safe test").unwrap();

    // -------------------------
    // I2C デバイススキャン
    // -------------------------
    scan_i2c(&mut i2c, &mut serial_wrapper);

    // -------------------------
    // OLED 初期化（簡易版）
    // -------------------------
    let address = 0x3C;
    let init_sequence: &[u8] = &[
        0xAE, 0xDC, 0x00, 0x81, 0x2F, 0x20, 0xA0, 0xC0,
        0xA4, 0xA6, 0xA8, 0x7F, 0xD3, 0x60, 0xD5, 0x51,
        0xD9, 0x22, 0xDB, 0x35, 0xAD, 0x8A, 0xAF,
    ];

    for cmd in init_sequence {
        i2c.write(address, &[*cmd]).ok();
        delay.delay_ms(1u16); // ちょっと待つ
    }

    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // -------------------------
    // クロス＋矩形描画
    // -------------------------
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        let mut page_buf = [0u8; DISPLAY_WIDTH];

        for x in 0..DISPLAY_WIDTH {
            // 横線
            if page == (DISPLAY_HEIGHT / 2 / PAGE_HEIGHT) {
                page_buf[x] = 0xFF;
            }

            // 縦線
            let bit_in_page = (DISPLAY_HEIGHT / 2) % 8;
            if x == DISPLAY_WIDTH / 2 {
                page_buf[x] |= 1 << bit_in_page;
            }

            // 矩形（20x20）を中央に描く
            let rect_left = DISPLAY_WIDTH / 2 - 10;
            let rect_top = DISPLAY_HEIGHT / 2 - 10;
            let rect_right = rect_left + 20;
            let rect_bottom = rect_top + 20;

            if x >= rect_left && x < rect_right {
                let y_start_in_page = page * PAGE_HEIGHT;
                for bit in 0..8 {
                    let y = y_start_in_page + bit;
                    if y >= rect_top && y < rect_bottom {
                        page_buf[x] |= 1 << bit;
                    }
                }
            }
        }

        // ページ切替
        i2c.write(address, &[0xB0 + page as u8, 0x00, 0x10]).ok();
        i2c.write(address, &[0x40]).ok(); // データ開始
        i2c.write(address, &page_buf).ok();
    }

    writeln!(serial_wrapper, "[oled] cross + rect drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
