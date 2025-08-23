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
const PAGE_HEIGHT: usize = 8;

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
    writeln!(serial_wrapper, "[log] Start UNO + SH1107G").unwrap();

    let address = 0x3C;

    // -----------------------------
    // 安定した I2C 初期化シーケンス
    // -----------------------------
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

    // -----------------------------
    // ページ単位でクロス描画
    // -----------------------------
    let mut page_buf: [u8; DISPLAY_WIDTH] = [0; DISPLAY_WIDTH];

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        // 横線
        if page == (DISPLAY_HEIGHT / 2 / PAGE_HEIGHT) {
            for x in 0..DISPLAY_WIDTH {
                page_buf[x] = 0xFF;
            }
        } else {
            for x in 0..DISPLAY_WIDTH {
                page_buf[x] = 0x00;
            }
        }
        i2c.write(address, &[0xB0 + page as u8]).ok(); // ページアドレス
        i2c.write(address, &[0x00, 0x10]).ok();       // 列アドレス
        i2c.write(address, &[0x40]).ok();             // データバイト開始
        i2c.write(address, &page_buf).ok();
    }

    // 縦線
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        page_buf = [0; DISPLAY_WIDTH];
        let y_in_page = DISPLAY_HEIGHT / 2 % PAGE_HEIGHT;
        page_buf[DISPLAY_WIDTH / 2] = 1 << y_in_page;
        i2c.write(address, &[0xB0 + page as u8]).ok();
        i2c.write(address, &[0x00, 0x10]).ok();
        i2c.write(address, &[0x40]).ok();
        i2c.write(address, &page_buf).ok();
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
