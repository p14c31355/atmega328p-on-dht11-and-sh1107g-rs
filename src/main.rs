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
    // ... （シリアルとI2Cの初期化） ...

    // 1. ドライバを初期化
    let mut display = match Sh1107gBuilder::new()
        .connect_i2c(i2c)
        .with_address(0x3C)
        .build(&mut serial)
    {
        Ok(d) => d,
        Err(e) => {
            let _ = uwriteln!(&mut serial, "DRIVER INIT ERROR: {:?}", e);
            loop {}
        }
    };
    uwriteln!(&mut serial, "Driver built. Clearing screen...").ok();

    // 2. 画面全体をオフ（黒）でクリアする
    if display.clear(BinaryColor::Off).is_err() {
        uwriteln!(&mut serial, "Clear FAILED!").ok();
        loop {}
    }
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