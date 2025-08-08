#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    style::PrimitiveStyleBuilder,
};

use sh1107g_rs::{Sh1107g, Sh1107gBuilder, display_size::DisplaySize};
use dvcdbg::{logger::SerialLogger, log_bytes};

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART 初期化（57600bps）
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = SerialLogger::new(serial);

    // I2C 初期化
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,
    );

    // Sh1107gBuilder を使用してドライバを初期化
    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        .with_size(DisplaySize::Display128x128)
        .build()
        .unwrap_or_else(|_| {
            log_bytes!(b"ERR: display init failed\n", &mut logger).ok();
            panic!()
        });

    // 真っ白に塗りつぶすスタイル
    let white_style = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::On)
        .build();

    // 全画面矩形
    let rect = Rectangle::new(Point::new(0, 0), Size::new(128, 128));

    // 描画
    if let Err(_) = rect.into_styled(white_style).draw(&mut display) {
        log_bytes!(b"ERR: draw failed\n", &mut logger).ok();
    }

    // バッファ送信
    if let Err(_) = display.flush() {
        log_bytes!(b"ERR: flush failed\n", &mut logger).ok();
    } else {
        log_bytes!(b"OK: white screen drawn\n", &mut logger).ok();
    }

    loop {}
}