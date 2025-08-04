#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::{i2c, Peripherals};
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use ufmt::uwriteln;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化（デバッグ用）
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    uwriteln!(&mut serial, "Start!").ok();

    // I2C 初期化
    let i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    uwriteln!(&mut serial, "I2C init OK").ok();

    // SH1107G ディスプレイ初期化
    let mut display = match Sh1107gBuilder::new()
        .connect_i2c(i2c)
        .with_address(0x3C)
        .build(&mut serial)
    {
        Ok(d) => d,
        Err(e) => {
            // uwriteln!が返すResultを明示的に処理
            match uwriteln!(&mut serial, "DRIVER INIT ERROR: {:?}", e) {
                Ok(_) => {}, // 成功時は何もしない
                Err(_) => {
                    // シリアル通信エラーが発生した場合、ここで停止
                    loop {}
                }
            }
            loop {}
        }
    };

    let init_cmds: &[u8] = &[
        0xAE,           // Display Off
        0x40,           // Display Start Line
        0x20, 0x02,     // Memory Addressing Mode
        0x81, 0x80,     // Contrast Control
        0xA0,           // Segment Remap (通常表示)
        0xA4,           // Entire Display On
        0xA6,           // Normal Display
        0xA8, 0x7F,     // Multiplex Ratio
        0xD3, 0x60,     // Display Offset
        0xD5, 0x51,     // Display Clock Divide Ratio
        0xC0,           // COM Output Scan Direction (通常表示)
        0xD9, 0x22,     // Pre-charge Period
        0xDA, 0x12,     // COM Pins Hardware Configuration
        0xDB, 0x35,     // VCOMH Deselect Level
        0xAD, 0x8B,     // Charge Pump
        0xAF,           // Display On
    ];

    if let Err(_) = display.write_command_list(init_cmds, &mut serial) {
        uwriteln!(&mut serial, "CMD LIST FAILED").ok();
        loop {}
    }
    uwriteln!(&mut serial, "CMD LIST OK").ok();


    if let Err(_) = display.init() {
        uwriteln!(&mut serial, "Display init FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Display init OK").ok();

    // 文字表示
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    if let Err(_) = Text::new("Hello!", Point::new(16, 16), text_style).draw(&mut display) {
        uwriteln!(&mut serial, "Draw FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Draw OK").ok();

    if let Err(_) = display.flush() {
        uwriteln!(&mut serial, "Flush FAILED!").ok();
        loop {}
    }
    uwriteln!(&mut serial, "Flush OK").ok();

    // 実行完了
    arduino_hal::delay_ms(500);
    uwriteln!(&mut serial, "All done.").ok();

    loop {} // 無限ループで終了させない
}
