#![no_std]
#![no_main]

use dvcdbg::prelude::*;
use panic_abort as _;

use embedded_graphics::{
    mono_font::{
        ascii::FONT_8X13_BOLD,
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle, PrimitiveStyle},
    text::{Baseline, Text},
};

use sh1107g_rs::Sh1107gBuilder;

// A wrapper for Serial communication
adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Initialize serial for debugging
    // let serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 115200));
    arduino_hal::delay_ms(1000);

    // Initialize I2C bus
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );

    // Create a new display driver instance
    // You can set the I2C address here (e.g., 0x3D) if it's different
    let mut display = Sh1107gBuilder::new(&mut i2c).build();

    // Initialize the display with a set of commands
    // The init() method is provided in lib.rs
    // based on the SH1107G_INIT_CMDS array in cmds.rs
    display.init().unwrap();
    arduino_hal::delay_ms(1000);
    display.clear_buffer();

    // Draw a rectangle
    Rectangle::new(Point::new(10, 10), Size::new(100, 50))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut display)
        .unwrap();
    display.clear_buffer();
    // Draw a circle
    Circle::new(Point::new(64, 90), 20)
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
    display.clear_buffer();
    // Draw some text
    let character_style = MonoTextStyle::new(&FONT_8X13_BOLD, BinaryColor::On);
    Text::with_baseline(
        "Hello, Rust!",
        Point::new(15, 30),
        character_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    // Flush the buffer to the display to show the changes
    display.flush().unwrap();
    
    // The loop is necessary to prevent the program from ending
    // on embedded systems.
    loop {}
}