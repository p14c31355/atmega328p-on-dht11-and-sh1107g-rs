#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::prelude::*;
use arduino_hal::i2c;
use core::fmt::Write;

use dvcdbg::prelude::*;

adapt_serial!(UnoSerial);

/// ヘルパー関数：BufferedLogger の内容をシリアルに出力
fn flush_logger<W: dvcdbg::prelude::SerialCompat>(logger: &dvcdbg::logger::BufferedLogger<256>, serial: &mut W) {
    for &b in logger.buffer().as_bytes() {
        // nb::block! は Result<T, nb::Error<E>> を返すので、match で処理する
        match nb::block!(serial.write(&[b])) {
            Ok(_) => {}, // 成功時は何もしない
            Err(nb::Error::WouldBlock) => {
                // WouldBlock エラーは、ノンブロッキング操作がまだ完了していないことを示す。
                // このコンテキストでは、ブロックするまで待つ nb::block! を使っているので、
                // ここに到達することはないはずだが、念のため含める。
            },
            Err(nb::Error::Other(_e)) => {
                // その他のエラーは、シリアル通信で発生した実際のエラー。
                // ここではエラーを無視するが、実際にはエラーハンドリングを検討すべき。
            }
        }
    }
    // serial.flush() も Result を返すので、同様に処理する
    match serial.flush() {
        Ok(_) => {},
        Err(_e) => {
            // フラッシュエラーも無視する
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoSerial(serial);

    // BufferedLogger 作成
    let mut logger = dvcdbg::logger::BufferedLogger::<256>::new();

    // 通常ログ
    log!(logger, "Hello, world!");
    log!(logger, "テスト: 0x48 0x65 0x6C 0x6C 0x6F");
    flush_logger(&logger, &mut serial_wrapper.0);

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // I2C スキャン（リアルタイムログ）
    log!(logger, "[scan] I2C scan start");
    flush_logger(&logger, &mut serial_wrapper.0);

    dvcdbg::scanner::scan_i2c(&mut i2c, &mut logger);

    log!(logger, "[scan] normal scan done");
    flush_logger(&logger, &mut serial_wrapper.0);

    // 初期化コマンド応答スキャン
    let init_sequence: [u8; 3] = [0xAE, 0xA1, 0xAF];
    log!(logger, "[scan] init sequence test start");
    flush_logger(&logger, &mut serial_wrapper.0);

    dvcdbg::scanner::scan_init_sequence(&mut i2c, &mut logger, &init_sequence);

    log!(logger, "[scan] init sequence test done");
    flush_logger(&logger, &mut serial_wrapper.0);

    loop {
        delay.delay_ms(1000u16);
    }
}
