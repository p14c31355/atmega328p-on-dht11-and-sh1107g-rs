#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};
use panic_halt as _;
use sh1107g_rs::Sh1107g;
use dvcdbg::prelude::*;
use core::fmt::Write;

use dvcdbg::scanner::scan_i2c;

impl_fmt_write_for_serial!(arduino_hal::hal::usart::Usart, write_byte);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = default_serial!(dp, pins, 57600);

    let mut logger = SerialLogger::new(&mut serial);

    let mut i2c = arduino_hal::I2c::new( // i2c を可変にする
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    // I2Cバスのスキャンを実行
    scan_i2c(&mut i2c, &mut logger);

    let mut display = Sh1107g::new(&mut i2c, 0x3C);

    // log! マクロの呼び出しを display.with_logger でラップ
    // `Sh1107g` に `with_logger` メソッドを追加
    log!(logger, "Initializing...");

    display.init().unwrap();
    display.clear(BinaryColor::Off).unwrap();

    log!(logger, "Display initialized and cleared.");
    

    display.clear(BinaryColor::On).unwrap();
    display.flush().unwrap();
    
        log!(logger, "Display filled with white.");

    loop {
        // 無限ループ
    }
}
