#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_io::Write;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, Rectangle},
};
use panic_halt as _;
use sh1107g_rs::Sh1107gBuilder;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] SH1107G demo start").ok();

    // I2C デバイススキャン
    scan_i2c(&mut i2c, &mut serial_wrapper);

    // SH1107G 初期化
    let mut display = Sh1107gBuilder::new(i2c).build();
    display.init().unwrap();
    display.clear_buffer();

    // embedded-graphics で描画
    let style_on = PrimitiveStyle::with_fill(BinaryColor::On);

    // 十字
    Line::new(Point::new(0, 64), Point::new(127, 64))
        .into_styled(style_on)
        .draw(&mut display)
        .unwrap();

    Line::new(Point::new(64, 0), Point::new(64, 127))
        .into_styled(style_on)
        .draw(&mut display)
        .unwrap();

    // 四角
    Rectangle::new(Point::new(20, 20), Size::new(40, 40))
        .into_styled(style_on)
        .draw(&mut display)
        .unwrap();

    // OLED に反映
    display.flush().unwrap();

    writeln!(serial_wrapper, "[oled] drawing done").ok();

    loop {
        delay.delay_ms(1000u16);
    }
}
