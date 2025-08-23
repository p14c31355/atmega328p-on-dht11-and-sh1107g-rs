#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use arduino_hal::hal::port::mode::Input;
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::draw_target::DrawTarget;
use embedded_io::Write;
use panic_halt as _;

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const PAGE_HEIGHT: usize = 8;

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

    // OLED 初期化シーケンス
    let addr = 0x3C;
    let init_sequence: &[u8] = &[
        0xAE,       // Display OFF
        0xD5, 0x51, // Clock
        0xA8, 0x7F, // Multiplex
        0xD3, 0x60, // Offset
        0x40,       // Start line
        0x8D, 0x14, // Charge pump
        0x20, 0x02, // Memory mode: page
        0xA0,       // Segment remap
        0xC0,       // COM scan dir
        0xDA, 0x12, // COM pins
        0x81, 0x2F, // Contrast
        0xD9, 0x22, // Pre-charge
        0xDB, 0x35, // VCOMH
        0xA4,       // Entire display from RAM
        0xA6,       // Normal display
        0xAF,       // Display ON
    ];

    for cmd in init_sequence {
        i2c.write(addr, &[*cmd]).ok();
        delay.delay_ms(5u16);
    }

    // ページバッファ
    let mut page_buf: [u8; DISPLAY_WIDTH] = [0; DISPLAY_WIDTH];

    // 十字描画（ページ単位）
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        // 横線
        if page == (DISPLAY_HEIGHT / 2) / PAGE_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                page_buf[x] = 0xFF;
            }
        } else {
            page_buf.fill(0);
        }

        // ページアドレス設定
        i2c.write(addr, &[0xB0 + page as u8]).ok(); // ページ
        i2c.write(addr, &[0x00]).ok(); // 列下位
        i2c.write(addr, &[0x10]).ok(); // 列上位

        // データ送信（制御バイト 0x40 付き）
        let mut payload: [u8; DISPLAY_WIDTH + 1] = [0; DISPLAY_WIDTH + 1];
        payload[0] = 0x40;
        payload[1..].copy_from_slice(&page_buf);
        i2c.write(addr, &payload).ok();
        delay.delay_ms(1u16);
    }

    // 縦線もページ単位で送信
    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        page_buf.fill(0);
        for y_in_page in 0..PAGE_HEIGHT {
            let global_y = page * PAGE_HEIGHT + y_in_page;
            if global_y == DISPLAY_HEIGHT / 2 {
                page_buf[DISPLAY_WIDTH / 2] = 0xFF;
            }
        }

        i2c.write(addr, &[0xB0 + page as u8]).ok();
        i2c.write(addr, &[0x00]).ok();
        i2c.write(addr, &[0x10]).ok();

        let mut payload: [u8; DISPLAY_WIDTH + 1] = [0; DISPLAY_WIDTH + 1];
        payload[0] = 0x40;
        payload[1..].copy_from_slice(&page_buf);
        i2c.write(addr, &payload).ok();
        delay.delay_ms(1u16);
    }

    loop {
        delay.delay_ms(1000u16);
    }
}
