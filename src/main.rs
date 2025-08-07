#![no_std]
#![no_main]

use arduino_hal::Peripherals;
use panic_halt as _;
use sh1107g_rs::Sh1107gBuilder;

#[cfg(feature = "log")]
use dvcdbg::logger;
#[cfg(feature = "log")]
use log::info;

// `log`フィーチャーが有効なときにロガーを初期化
#[cfg(feature = "log")]
extern crate log;

// ロギングが無効な場合にSh1107gBuilderに渡すダミーロガー
#[cfg(not(feature = "log"))]
struct NullLogger;

#[cfg(not(feature = "log"))]
impl log::Log for NullLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        false
    }
    fn log(&self, _record: &log::Record) {}
    fn flush(&self) {}
}

#[cfg(not(feature = "log"))]
static NULL_LOGGER: NullLogger = NullLogger;

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

    #[cfg(feature = "log")]
    info!("I2C initialized with 25_000 Hz.");

    // OLEDドライバの初期化
    let mut builder = {
        #[cfg(feature = "log")]
        { Sh1107gBuilder::new(i2c, &mut _logger) }

        #[cfg(not(feature = "log"))]
        { Sh1107gBuilder::new(i2c, &mut NULL_LOGGER) }
    };

    #[cfg(feature = "log")]
    info!("OLED builder created. Starting driver build.");

    let mut display = builder.build().unwrap();

    #[cfg(feature = "log")]
    info!("OLED driver built successfully. Starting init sequence.");

    // デバッガで init() シーケンスの成否をログに出力
    match display.init() {
        Ok(_) => {
            #[cfg(feature = "log")]
            info!("Display initialization successful!");
        },
        Err(e) => {
            // エラーをログに出力（デバッグ情報として）
            #[cfg(feature = "log")]
            info!("Display initialization failed: {:?}", e);
        }
    };

    #[cfg(feature = "log")]
    info!("Entering main loop...");

    loop {
        // flush() の実行もログで確認
        match display.flush() {
            Ok(_) => {
                #[cfg(feature = "log")]
                info!("Flush successful.");
            },
            Err(e) => {
                #[cfg(feature = "log")]
                info!("Flush failed: {:?}", e);
            }
        };

        // 描画や他のロジックをここに追加

        arduino_hal::delay_ms(500); // 500ミリ秒待つ
    }
}