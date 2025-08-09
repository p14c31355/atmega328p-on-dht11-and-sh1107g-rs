#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use dvcdbg::logger::SerialLogger;
use dvcdbg::log;
use embedded_hal::i2c::I2c;
use dvcdbg::logger::Logger;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    // シリアルとロガー初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = SerialLogger::new(serial);

    // I2C初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000, // 100kHz
    );

    log!(&mut logger, "I2Cスキャン開始");

    // スキャン
    for addr in 0x03..=0x77 {
        if i2c.write(addr, &[]).is_ok() {
            log!(&mut logger, "Found device at 0x{:02X}", addr);

            // 見つけたデバイスに SH1107G 初期化コマンド列を送信
            if addr == 0x3C || addr == 0x3D {
                log!(&mut logger, "SH1107G 初期化開始");

                let init_cmds: [u8; 24] = [
                    0x00, // Control byte for command
                    0xAE, // Display OFF
                    0xDC, 0x00, // Display start line = 0
                    0x81, 0x2F, // Contrast
                    0x20, // Memory addressing mode: page
                    0xA0, // Segment remap normal
                    0xC0, // Common output scan direction normal
                    0xA4, // Entire display ON from RAM
                    0xA6, // Normal display
                    0xA8, 0x7F, // Multiplex ratio 128
                    0xD3, 0x60, // Display offset
                    0xD5, 0x51, // Oscillator frequency
                    0xD9, 0x22, // Pre-charge period
                    0xDB, 0x35, // VCOM deselect level
                    0xAD, 0x8A, // DC-DC control
                    0xAF,       // Display ON
                ];

                if let Err(e) = i2c.write(addr, &init_cmds) {
                    log!(&mut logger, "Init failed: {:?}", e);
                } else {
                    log!(&mut logger, "Init OK");
                }
            }
        }
    }

    loop {}
}
