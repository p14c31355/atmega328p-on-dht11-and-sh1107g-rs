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

    // 1. ドライバを初期化
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
    uwriteln!(&mut serial, "Driver built. Clearing screen...").ok();

    // 2. 画面全体をオフ（黒）でクリアする
    display.clear(BinaryColor::Off).unwrap();
    uwriteln!(&mut serial, "Buffer cleared.").ok();

    // 3. クリアしたバッファをディスプレイに書き込む
    if let Err(e) = display.flush() {
        uwriteln!(&mut serial, "Flush FAILED!: {:?}", e).ok();
        loop {}
    }
    uwriteln!(&mut serial, "Screen flushed. Check the display!").ok();
    
    // 実行完了
    uwriteln!(&mut serial, "All done.").ok();

    loop {} // ここで処理を止めて、ディスプレイが真っ黒になるか確認する
}