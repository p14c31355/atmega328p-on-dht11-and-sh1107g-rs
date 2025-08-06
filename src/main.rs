#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::logger::SerialLogger;
use embedded_graphics::draw_target::DrawTarget;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // --- UART
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = SerialLogger::new(&mut serial); // ← 修正点1：&mut

    // --- I2C
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,
    );

    // --- Builder
    let builder = Sh1107gBuilder::new(i2c)
        .with_address(0x3C)
        .with_logger(&mut logger);

    let mut logger = SerialLogger::new(writer);
    // --- 注意：build に logger の writer を渡す！
    let mut display = builder
        .build(logger.writer)
        .expect("Failed to build SH1107G");

    // --- Display init + clear
    display.init().unwrap();
    display.clear().unwrap();
    display.flush().unwrap();

    loop {}
}
