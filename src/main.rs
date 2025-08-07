#![no_std]
#![no_main]

use core::fmt::Write;

use arduino_hal::Peripherals;
use panic_halt as _;

// `sh1107g_rs` ドライバクレートをインポート
use sh1107g_rs::{Sh1107gBuilder, BUFFER_SIZE, DISPLAY_HEIGHT, DISPLAY_WIDTH};

// `embedded-graphics`クレート
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

// `dvcdbg`ロガークレートをインポート
use dvcdbg::logger::SerialLogger;
use log::info;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // ロガーのシリアルポートをUART0 (D0, D1ピン)で初期化
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    // `serial`変数をSerialLoggerに渡すように修正
    let mut logger = SerialLogger::new(&mut serial);
    
    // シリアルポートをロガーに渡すことで、`info!`マクロの出力先になる
    info!("Starting Arduino application...");

    // I2CバスをA4 (SDA) と A5 (SCL) ピンで初期化
    let i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000, // 高速モード
    );

    // `Sh1107gBuilder`にI2Cバスとロガーを渡してドライバをビルド
    let mut display = Sh1107gBuilder::new(i2c, &mut logger).build().unwrap();

    info!("Display driver built successfully.");

    // ディスプレイを初期化
    // `init()`は画面をクリアし、コマンドを送信します。
    display.init().unwrap();

    // `embedded-graphics`の描画開始
    // 描画前にバッファをクリアするのが良いプラクティスです
    display.clear_buffer();

    // テキストスタイルを定義
    let character_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    // "Hello, World!" を描画
    Text::new("Hello, World!", Point::new(16, 64), character_style)
        .draw(&mut display)
        .unwrap();

    info!("Text 'Hello, World!' drawn to buffer.");

    // バッファの内容を物理ディスプレイに送信
    display.flush().unwrap();

    info!("Buffer flushed to display.");

    // プログラムが終了しないように無限ループ
    loop {
        // 必要に応じてここに描画やセンサー読み取りのロジックを追加
    }
}