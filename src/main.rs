#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_io::Write;
use panic_halt as _;

use sh1107g_rs::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G test").unwrap();

    // I2Cスキャン
    scan_i2c(&mut i2c, &mut serial_wrapper);

    // SH1107Gドライバ生成
    let mut disp = Sh1107g::new(i2c, 0x3C);

    // 初期化
    disp.init().unwrap();
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // 十字描画
    let width = 128;
    let height = 128;
    let page_height = 8;

    // 横線
    for page in 0..(height / page_height) {
        for x in 0..width {
            let idx = x + page * width;
            if page == height / 2 / page_height {
                disp.buffer[idx] = 0xFF;
            } else {
                disp.buffer[idx] = 0x00;
            }
        }
    }

    // 縦線
    for page in 0..(height / page_height) {
        for y_in_page in 0..page_height {
            let global_y = page * page_height + y_in_page;
            if global_y == height / 2 {
                let x = width / 2;
                let idx = x + page * width;
                disp.buffer[idx] = 0xFF;
            }
        }
    }

    // バッファをOLEDに送信
    disp.flush().unwrap();
    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
