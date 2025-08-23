#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::draw_target::DrawTarget;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const PAGE_HEIGHT: usize = 8; // SH1107 ページ単位

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

    writeln!(serial_wrapper, "[log] Start Uno + SH1107G test").unwrap();

    // -------------------------
    // I2C デバイススキャン
    // -------------------------
    scan_i2c(&mut i2c, &mut serial_wrapper);

    // -------------------------
    // OLED 初期化
    // -------------------------
    let address = 0x3C;
    let init_sequence: &[u8] = &[
        0xAE, 0xDC, 0x00, 0x81, 0x2F, 0x20, 0xA0, 0xC0, 0xA4,
        0xA6, 0xA8, 0x7F, 0xD3, 0x60, 0xD5, 0x51, 0xD9, 0x22,
        0xDB, 0x35, 0xAD, 0x8A, 0xAF,
    ];

    // 初期化コマンドはまとめて送信（64バイト以内）
    i2c.write(address, init_sequence).ok();
    delay.delay_ms(10u16); // バス安定化
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // -------------------------
    // ページ単位クロス描画
    // -------------------------
    let mut page_buf = [0u8; DISPLAY_WIDTH]; // ページバッファ 128バイト

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        // ページ切替コマンド
        let page_cmds = [0xB0 + page as u8, 0x00, 0x10];
        i2c.write(address, &page_cmds).ok();

        // 横線
        for x in 0..DISPLAY_WIDTH {
            page_buf[x] = if page == (DISPLAY_HEIGHT / 2 / PAGE_HEIGHT) { 0xFF } else { 0x00 };
        }

        // データ開始
        i2c.write(address, &[0x40]).ok();

        // ページバッファ送信を 64バイトずつ
        for chunk in page_buf.chunks(64) {
            i2c.write(address, chunk).ok();
        }
    }

    // 縦線もページ単位で送信
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        let mut page_buf = [0u8; DISPLAY_WIDTH];

        for y_in_page in 0..PAGE_HEIGHT {
            let global_y = page * PAGE_HEIGHT + y_in_page;
            if global_y == DISPLAY_HEIGHT / 2 {
                page_buf[DISPLAY_WIDTH / 2] = 0xFF;
            }
        }

        let page_cmds = [0xB0 + page as u8, 0x00, 0x10];
        i2c.write(address, &page_cmds).ok();
        i2c.write(address, &[0x40]).ok();
        for chunk in page_buf.chunks(64) {
            i2c.write(address, chunk).ok();
        }
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
