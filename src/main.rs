#![no_std]
#![no_main]

use panic_halt as _;
use atmega328p_on_dht11_and_sh1107g_rs::*; // lib.rs の公開アイテムをインポート

adapt_serial!(UnoWrapper); // main.rs で UnoWrapper を定義

#[arduino_hal::entry]
fn main() -> ! {
    // -------------------------------------------------------------------------
    // Arduino 初期化
    // -------------------------------------------------------------------------
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 100kHz
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100000,
    );

    // シリアル (dvcdbg ロガー)
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial); // UfmtWrapper でラップ
    let mut logger = SerialLogger::new(&mut serial_wrapper); // ラップしたものを渡す

    log!(logger, "[scan] start");
    dvcdbg::scan_i2c!(&mut i2c, &mut logger); // 可変参照を渡す
    log!(logger, "[scan] done");

    // -------------------------------------------------------------------------
    // SH1107G ドライバ生成 + 初期化
    // -------------------------------------------------------------------------
    let mut oled = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    oled.init().unwrap();
    log!(logger, "[oled] init complete");

    // -------------------------------------------------------------------------
    // パターン描画テスト
    // -------------------------------------------------------------------------

    // 1. 真っ黒画面
    oled.clear(BinaryColor::Off).unwrap();
    oled.flush().unwrap();
    log!(logger, "[oled] cleared (black screen)");
    delay.delay_ms(2000u16);

    // 2. 横ライン (y=32 に白)
    oled.clear(BinaryColor::Off).unwrap();
    for x in 0..DISPLAY_WIDTH {
        let _ = oled.draw_iter([Pixel(Point::new(x as i32, 32), BinaryColor::On)]);
    }
    oled.flush().unwrap();
    log!(logger, "[oled] horizontal line y=32");
    delay.delay_ms(2000u16);

    // 3. 縦ライン (x=64 に白)
    oled.clear(BinaryColor::Off).unwrap();
    for y in 0..DISPLAY_HEIGHT {
        let _ = oled.draw_iter([Pixel(Point::new(64, y as i32), BinaryColor::On)]);
    }
    oled.flush().unwrap();
    log!(logger, "[oled] vertical line x=64");

    loop {}
}
