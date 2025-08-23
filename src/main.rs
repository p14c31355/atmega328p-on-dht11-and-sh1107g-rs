#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_io::Write;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, Rectangle},
    Drawable,
};
use panic_halt as _;

use sh1107g_rs::{Sh1107g, Sh1107gBuilder};

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] SH1107G demo start").unwrap();

    // I2C スキャン
    // scan_i2c(&mut i2c, &mut serial_wrapper);

    // SH1107G ドライバ初期化
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    display.init().unwrap();
    writeln!(serial_wrapper, "[oled] init done").unwrap();

    // 画面中央に十字描画
    let center_x = sh1107g_rs::DISPLAY_WIDTH as i32 / 2;
    let center_y = sh1107g_rs::DISPLAY_HEIGHT as i32 / 2;

    // 横線
    Line::new(
        Point::new(0, center_y),
        Point::new(sh1107g_rs::DISPLAY_WIDTH as i32 - 1, center_y),
    )
    .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(BinaryColor::On))
    .draw(&mut display)
    .unwrap();

    // 縦線
    Line::new(
        Point::new(center_x, 0),
        Point::new(center_x, sh1107g_rs::DISPLAY_HEIGHT as i32 - 1),
    )
    .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(BinaryColor::On))
    .draw(&mut display)
    .unwrap();

    // 画面更新
    display.flush().unwrap();
    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
