#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::prelude::*;
use sh1107g_rs::{Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::BinaryColor;
use panic_halt as _;

// UART 用ラッパー（adapt_serial! マクロ前提）
adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    // -------------------------------------------------------------------------
    // Arduino 初期化
    // -------------------------------------------------------------------------
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // シリアル初期化（ログ用）
    let serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut serial_wrapper = UnoWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    log!(logger, "[log] Start minimal test");

    // I2C 初期化 (SDA=A4, SCL=A5, 100kHz)
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    log!(logger, "[scan] I2C scan start");

    
    
        dvcdbg::scanner::scan_i2c(&mut i2c, &mut logger);
    

    log!(logger, "[scan] I2C scan done");

    // -------------------------------------------------------------------------
    // SH1107G 初期化
    // -------------------------------------------------------------------------
    let mut oled = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    if oled.init().is_ok() {
        log!(logger, "[oled] init complete");
    } else {
        log!(logger, "[oled] init failed!");
    }

    // -------------------------------------------------------------------------
    // 簡単な描画テスト
    // -------------------------------------------------------------------------
    oled.clear(BinaryColor::Off).ok();
    oled.flush().ok();
    log!(logger, "[oled] cleared (black)");

    // 画面中央に十字ライン描画
    for x in 0..DISPLAY_WIDTH {
        let _ = oled.draw_iter([Pixel(Point::new(x as i32, (DISPLAY_HEIGHT/2) as i32), BinaryColor::On)]);
    }
    for y in 0..DISPLAY_HEIGHT {
        let _ = oled.draw_iter([Pixel(Point::new((DISPLAY_WIDTH/2) as i32, y as i32), BinaryColor::On)]);
    }
    oled.flush().ok();
    log!(logger, "[oled] cross line drawn");

    loop {
        // 繰り返しはせず、ここで止める
        delay.delay_ms(1000u16);
    }
}
