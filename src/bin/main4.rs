#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::prelude::*;
use arduino_hal::i2c;
use core::fmt::Write;

use dvcdbg::prelude::*;
adapt_serial!(UnoSerial);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // UART: 57600
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoSerial(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    // I2C
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // --------------------------
    // ログ用ヘルパー関数
    // --------------------------
    fn log_to_serial<L: Logger>(logger: &mut L, msg: &str) {
        logger.log_fmt(format_args!("{}", msg));
    }

    // テストログ
    log_to_serial(&mut logger, "Hello, world!");
    log_to_serial(&mut logger, "[scan] I2C scan start");

    // I2C スキャン
    dvcdbg::scanner::scan_i2c(&mut i2c, &mut logger);

    log_to_serial(&mut logger, "[scan] I2C scan done");

    // 初期化コマンド応答スキャン例
    let init_sequence: [u8; 3] = [0xAE, 0xA1, 0xAF];
    log_to_serial(&mut logger, "[scan] init sequence test start");
    dvcdbg::scanner::scan_init_sequence(&mut i2c, &mut logger, &init_sequence);
    log_to_serial(&mut logger, "[scan] init sequence test done");

    loop {
        delay.delay_ms(1000u16);
    }
}
