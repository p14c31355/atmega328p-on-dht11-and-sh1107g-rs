#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::adapt_serial;
use embedded_io::Write;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G demo").unwrap();
    
    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    
    

    // SH1107G 初期化
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    display.init().unwrap();
    writeln!(serial_wrapper, "[oled] init done").ok();

    // バッファに十字を描画
    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    // 縦線
    Line::new(
        Point::new((DISPLAY_WIDTH / 2) as i32, 0),
        Point::new((DISPLAY_WIDTH / 2) as i32, (DISPLAY_HEIGHT - 1) as i32),
    )
    .into_styled(style)
    .draw(&mut display)
    .unwrap();

    // 横線
    Line::new(
        Point::new(0, (DISPLAY_HEIGHT / 2) as i32),
        Point::new((DISPLAY_WIDTH - 1) as i32, (DISPLAY_HEIGHT / 2) as i32),
    )
    .into_styled(style)
    .draw(&mut display)
    .unwrap();

    // OLED に描画反映
    display.flush().unwrap();
    writeln!(serial_wrapper, "[oled] cross drawn").ok();

    loop {
        delay.delay_ms(1000u16);
    }
}
