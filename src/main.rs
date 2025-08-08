#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::hal::port;
use sh1107g_rs::{Sh1107g, Sh1107gBuilder, DefaultLogger};
use dvcdbg::log;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    style::PrimitiveStyleBuilder, // embedded-graphics 0.8.1
};

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    // Arduino HAL 初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART 初期化（57600bps）
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // ロガーを初期化
    let mut logger = DefaultLogger::new(&mut serial);

    // I2C 初期化
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,
    );

    // SH1107G ディスプレイドライバをビルダパターンで初期化
    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        .build_logger()
        .unwrap_or_else(|e| {
            log!(&mut logger, "ERR: display init failed: {:?}", e);
            panic!();
        });

    // 真っ白に塗りつぶすスタイル
    let white_style = PrimitiveStyleBuilder::new()
        .fill_color(BinaryColor::On)
        .build();

    // 全画面矩形
    let rect = Rectangle::new(Point::new(0, 0), Size::new(128, 128));

    // 描画
    if let Err(e) = rect.into_styled(white_style).draw(&mut display) {
        log!(&mut logger, "ERR: draw failed: {:?}", e);
    }

    // バッファ送信
    if let Err(e) = display.flush() {
        log!(&mut logger, "ERR: flush failed: {:?}", e);
    } else {
        log!(&mut logger, "OK: white screen drawn");
    }

    loop {}
}