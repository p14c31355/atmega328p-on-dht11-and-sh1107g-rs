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

    // UART
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoSerial(serial);
    writeln!(serial_wrapper, "Hello, world!").unwrap();

    // BufferedLogger に書き込む
    let mut buf_logger: dvcdbg::logger::BufferedLogger<512> = dvcdbg::logger::BufferedLogger::new();

    // I2C
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    log!(buf_logger, "テスト");
    log!(buf_logger, "0x48 0x65 0x6C 0x6C 0x6F"); // "Hello"
    log!(buf_logger, "[scan] I2C scan start (normal)");
    dvcdbg::scanner::scan_i2c(&mut i2c, &mut buf_logger);
    log!(buf_logger, "[scan] normal scan done");

    // 初期化コマンド応答スキャン
    let init_sequence: [u8; 3] = [0xAE, 0xA1, 0xAF];
    log!(buf_logger, "[scan] init sequence test start");
    dvcdbg::scanner::scan_init_sequence(&mut i2c, &mut buf_logger, &init_sequence);
    log!(buf_logger, "[scan] init sequence test done");

    // バッファから少しずつシリアルに出力
    let chunk_size = 64;
    let buf = buf_logger.buffer();
    for &b in buf_logger.buffer().as_bytes() {
        nb::block!(serial_wrapper.0.write(b)).ok();
    }


    loop {
        delay.delay_ms(1000u16);
    }
}
