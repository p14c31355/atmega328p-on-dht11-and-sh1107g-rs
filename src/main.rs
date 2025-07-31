#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use embedded_dht_rs::{Dht11, DhtReading};
use oled_async::prelude::*;
use oled_async::interface::I2CDisplayInterface;
use oled_async::displays::sh1107::Sh1107_128_128;
use arduino_hal::i2c::I2c;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // I2C初期化
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        400_000,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Sh1107_128_128::new(interface).into_buffered_graphics_mode();
    display.init().unwrap();
    display.flush().unwrap();

    // DHT11 on D2
    let dht_pin = pins.d2.into_open_drain_output();
    let mut dht = Dht11::new(dht_pin);

    // UART for debugging (optional)
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    loop {
        match dht.read() {
            Ok(DhtReading { temperature, relative_humidity }) => {
                ufmt::uwriteln!(
                    &mut serial,
                    "Temp: {} C, Hum: {} %\r",
                    temperature,
                    relative_humidity
                )
                .ok();

                display.clear();
                use core::fmt::Write;
                use oled_async::graphics::GraphicsMode;
                let _ = write!(
                    display,
                    "Temp:{}C\nHum:{}%",
                    temperature,
                    relative_humidity
                );
                display.flush().unwrap();
            }
            Err(e) => {
                ufmt::uwriteln!(&mut serial, "Read error: {:?}\r", e).ok();
            }
        }

        arduino_hal::delay_ms(2000);
    }
}
