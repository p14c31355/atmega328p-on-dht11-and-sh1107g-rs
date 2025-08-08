#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder, Sh1107g};

use dvcdbg::logger::SerialLogger;

use core::fmt::Write;
use heapless::String;
use nb::block;

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

    // UART 57600 baud
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    // I2C初期化 (SCL=A5, SDA=A4, 100kHz)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a5.into_pull_up_input(),
        pins.a4.into_pull_up_input(),
        100_000,
    );

    // SH1107g OLEDドライバ初期化
    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        .with_address(0x3C)
        .build();

    // 初期化処理 (コマンド送信時にログ出力される)
    if let Err(e) = display.init() {
        let mut buf: String<64> = String::new();
        let _ = write!(buf, "Display init failed: {:?}", e);
        logger.log_i2c(buf.as_str(), Err(()));
        loop {}
    }

    display.clear_buffer();

    // ここで簡単に何か描画する（例として全部点灯）
    for i in 0..display.buffer.len() {
        display.buffer[i] = 0xFF;
    }

    // バッファをOLEDへ送信（flush時にもログあり）
    if let Err(e) = display.flush() {
        let mut buf: String<64> = String::new();
        let _ = write!(buf, "Flush failed: {:?}", e);
        logger.log_i2c(buf.as_str(), Err(()));
    }

    loop {
        // ここは無限ループ
    }
}
