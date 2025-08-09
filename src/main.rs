#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use dvcdbg::logger::{SerialLogger, Logger};
use dvcdbg::log;
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;
use core::fmt::Write;
use panic_halt as _;

use sh1107g_rs::Sh1107g;
use sh1107g_rs::Sh1107gBuilder;

struct SerialWriter<'a, W: EmbeddedHalSerialWrite<u8>> {
    writer: &'a mut W,
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> SerialWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> Write for SerialWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            nb::block!(self.writer.write(byte)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    // シリアルとロガー初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    // I2C初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000, // 100kHz
    );

    log!(&mut logger, "I2Cスキャン開始");

    let mut found_addr = None;
    for addr in 0x03..=0x77 {
        if i2c.write(addr, &[]).is_ok() {
            log!(&mut logger, "Found device at 0x{:02X}", addr);
            if addr == 0x3C || addr == 0x3D {
                found_addr = Some(addr);
                break;
            }
        }
    }

    if let Some(addr) = found_addr {
        log!(&mut logger, "SH1107G 初期化開始");
        // 所有権を渡して初期化はここで1回だけ
        // i2c所有権渡すのでmutは必要
          let builder = Sh1107gBuilder::new(i2c, &mut logger).with_address(addr);

          let mut builder = Sh1107gBuilder::new(i2c, &mut logger).with_address(addr);
          let mut display = builder.build();

          log!(&mut logger, "build() 成功");

          // 必要なら手動で初期化呼ぶ（非同期なら違うかもですが）
          // display.init().unwrap();

          if display.flush().is_ok() {
              log!(&mut logger, "flush() 成功");
          } else {
              log!(&mut logger, "flush() 失敗");
          }
          
        // ここから display を使った描画ループなどに入る
    }

    loop {}
  }