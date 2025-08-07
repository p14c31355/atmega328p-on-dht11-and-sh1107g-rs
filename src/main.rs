#![no_std]
#![no_main]

use arduino_hal::Peripherals;
use panic_halt as _;
use sh1107g_rs::Sh1107gBuilder;

// ロガーをdvcdbgクレートからインポート
use dvcdbg::logger;
use log::info;

// `log`フィーチャーが有効なときにロガーを初期化
#[cfg(feature = "log")]
extern crate log;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // ロガーの初期化とインスタンスの作成
    #[cfg(feature = "log")]
    let mut _logger = logger::init().unwrap();
    #[cfg(feature = "log")]
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
    // `log`フィーチャーが有効なときのみロガーを渡す
    let mut builder = {
        #[cfg(feature = "log")]
        { Sh1107gBuilder::new(i2c, &mut _logger) }

        #[cfg(not(feature = "log"))]
        { Sh1107gBuilder::new(i2c) }
    };
    
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