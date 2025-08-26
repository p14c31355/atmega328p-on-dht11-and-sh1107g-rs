#![no_std]
#![no_main]

use core::fmt::Write;

use arduino_hal::prelude::*;
use panic_halt as _;
use dvcdbg::prelude::*;
use embedded_graphics_core::draw_target::DrawTarget;

adapt_serial!(UnoWrapper);

use sh1107g_rs::{Sh1107gBuilder, error::*};

// arduino_hal::hal::usart::Usart を embedded_io::Write に適合させるアダプター


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UnoWrapper(serial);

    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    let _ = writeln!(logger, "[log] Init SH1107G...");

    if let Err(e) = display.init() {
        log_error(&mut logger, "init failed", &e);
    }

    // テスト描画
    for y in 0..8 {
        for x in 0..8 {
            let _ = display.draw_iter(core::iter::once(
                embedded_graphics_core::Pixel(
                    embedded_graphics_core::geometry::Point::new(x, y),
                    embedded_graphics_core::pixelcolor::BinaryColor::On
                )
            ));
        }
    }

    if let Err(e) = display.flush() {
        log_error(&mut logger, "flush failed", &e);
    }

    loop {
        delay.delay_ms(1000u16);
    }
}

// dvcdbg UnoWrapper で Sh1107gError を表示
fn log_error<E>(logger: &mut UnoWrapper<impl dvcdbg::compat::SerialCompat>, msg: &str, err: &Sh1107gError<E>)
where
    E: core::fmt::Debug,
{
    let _ = writeln!(logger, "[error] {}: {:?}\n", msg, err);
}
