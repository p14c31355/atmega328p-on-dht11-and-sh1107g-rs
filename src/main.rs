#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::scan_i2c};
use embedded_io::Write;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use panic_halt as _;
use sh1107g_rs::{Sh1107g,Sh1107gBuilder};
use sh1107g_rs::DISPLAY_WIDTH;
use sh1107g_rs::DISPLAY_HEIGHT;

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
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    writeln!(serial, "[log] Start SH1107G test").ok();


    // SH1107G ドライバ初期化
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    display.init().unwrap();
    display.clear_buffer();

    // embedded_graphics で十字描画
    let center_x = (DISPLAY_WIDTH / 2) as i32;
    let center_y = (DISPLAY_HEIGHT / 2) as i32;

    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    Line::new(Point::new(0, center_y), Point::new(DISPLAY_WIDTH as i32 - 1, center_y))
        .into_styled(line_style)
        .draw(&mut display)
        .ok();

    Line::new(Point::new(center_x, 0), Point::new(center_x, DISPLAY_HEIGHT as i32 - 1))
        .into_styled(line_style)
        .draw(&mut display)
        .ok();

    display.flush().unwrap();

    loop {
        delay.delay_ms(1000u16);
    }
}
