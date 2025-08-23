#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use sh1107g_rs::{DISPLAY_WIDTH, DISPLAY_HEIGHT};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::draw_target::DrawTarget;
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

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
    // ページ単位 OLED 初期化
    // -------------------------
    let address = 0x3C;
    let init_sequence: &[u8] = &[
        0xAE, // Display OFF
        0xDC, 0x00, // Display start line = 0
        0x81, 0x2F, // Contrast
        0x20, // Memory addressing mode: page
        0xA0, // Segment remap normal
        0xC0, // Common output scan direction normal
        0xA4, // Entire display ON from RAM
        0xA6, // Normal display
        0xA8, 0x7F, // Multiplex ratio 128
        0xD3, 0x60, // Display offset
        0xD5, 0x51, // Oscillator frequency
        0xD9, 0x22, // Pre-charge period
        0xDB, 0x35, // VCOM deselect level
        0xAD, 0x8A, // DC-DC control
        0xAF,       // Display ON
    ];

    for cmd in init_sequence {
        i2c.write(address, &[*cmd]).ok();
        delay.delay_ms(1000u16); // バス安定化
    }
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // -------------------------
    // ページ単位でバッファ描画
    // -------------------------
    let mut page_buf: [u8; DISPLAY_WIDTH as usize] = [0; DISPLAY_WIDTH as usize]; // 128バイトだけ確保

    // 十字を描画
    for page in 0..(DISPLAY_HEIGHT as usize / PAGE_HEIGHT) {
        for x in 0..DISPLAY_WIDTH as usize {
            page_buf[x] = if page == (DISPLAY_HEIGHT/2 / PAGE_HEIGHT as u32) as usize {
                0xFF // 横線
            } else {
                0x00
            };
        }
        i2c.write(address, &[0x40]).ok(); // データバイト開始
        i2c.write(address, &page_buf).ok();
    }

    // 縦線もページ単位で送信
    for page in 0..(DISPLAY_HEIGHT as usize / PAGE_HEIGHT) {
        page_buf = [0; DISPLAY_WIDTH as usize];
        for y_in_page in 0..PAGE_HEIGHT {
            let global_y = page * PAGE_HEIGHT + y_in_page;
            if global_y == DISPLAY_HEIGHT as usize / 2 {
                page_buf[DISPLAY_WIDTH as usize/2] = 0xFF;
            }
        }
        i2c.write(address, &[0x40]).ok();
        i2c.write(address, &page_buf).ok();
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
