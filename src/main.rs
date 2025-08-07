#![no_std]
#![no_main]

use arduino_hal::Peripherals;
use panic_halt as _;
use sh1107g_rs::{Sh1107gBuilder, DisplaySize};

// ロガーをインポート
use dvcdbg::logger;
use log::info;

// `log`フィーチャーが有効なときにロガーを初期化
#[cfg(feature = "log")]
extern crate log;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // ロガーの初期化
    // シリアル通信の初期化もここで行われます
    #[cfg(feature = "log")]
    logger::init().unwrap();

    info!("Arduino HAL is initialized.");

    // I2Cの初期化
    let i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        25_000,
    );

    info!("I2C initialized with 25_000 Hz.");

    // OLEDドライバの初期化
    let builder = Sh1107gBuilder::new()
        .connect_i2c(i2c)
        .with_size(DisplaySize::Display128x128); // 適切なサイズを指定

    info!("OLED builder created. Starting driver build.");

    let mut display = builder.build().unwrap();

    info!("OLED driver built successfully. Starting init sequence.");

    // デバッガで init() シーケンスの成否をログに出力
    match display.init() {
        Ok(_) => info!("Display initialization successful!"),
        Err(e) => {
            // エラーをログに出力（デバッグ情報として）
            info!("Display initialization failed: {:?}", e);
        }
    };

    info!("Entering main loop...");

    loop {
        // flush() の実行もログで確認
        match display.flush() {
            Ok(_) => info!("Flush successful."),
            Err(e) => {
                info!("Flush failed: {:?}", e);
            }
        };

        // 描画や他のロジックをここに追加

        arduino_hal::delay_ms(500); // 500ミリ秒待つ
    }
}