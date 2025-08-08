#![no_std]
#![no_main]

use arduino_hal::Peripherals;
use panic_halt as _;

use sh1107g_rs::Sh1107gBuilder;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
};

use dvcdbg::logger::SerialLogger;
use nb::block;

// UART用の書き込みラッパー
struct FmtWriteWrapper<W>(W);
impl<W> core::fmt::Write for FmtWriteWrapper<W>
where
    W: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // ドライバのビルド（logger付き）
    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        //.with_address(0x3C) // 必要ならアドレス指定
        .build();

    // 初期化コマンド送信
    display.init().unwrap();

    // バッファを0x00でクリア
    display.clear_buffer();

    // クリアしたバッファをディスプレイへ送信
    display.flush().unwrap();

    // ここまで来たら砂嵐ではなく真っ黒になるはず

    loop {
        // 無限ループで停止
    }
}
