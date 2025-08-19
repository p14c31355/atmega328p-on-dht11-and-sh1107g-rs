#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use arduino_hal::pins;
use panic_halt as _;
use dvcdbg::logger::Logger;
use dvcdbg::log;
use core::fmt::Write;
use embedded_hal::blocking::i2c;
use ufmt::uWrite; // ufmt::uWrite をインポート
use heapless::String; // heapless::String をインポート

// `ufmt::uWrite` を `core::fmt::Write` に適合させるアダプター
struct UfmtAdapter<W>(W);

impl<W: uWrite> core::fmt::Write for UfmtAdapter<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(s).map_err(|_| core::fmt::Error)
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = pins!(dp);

    // Serial
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut adapter = UfmtAdapter(serial); // UfmtAdapter を使用
    let mut logger = dvcdbg::logger::SerialLogger::new(&mut adapter); // dvcdbg::logger::SerialLogger を使用

    // I2C pins -> PullUp
    let scl = pins.a5.into_pull_up_input();
    let sda = pins.a4.into_pull_up_input();
    let mut i2c = arduino_hal::I2c::new(dp.TWI, sda, scl, 100_000);

    log!(logger, "[info] Starting I2C scan 0x03..0x77");

    // I2C スキャンロジックを直接実装
    for addr in 0x03..=0x77 {
        match i2c.write(addr, &[]) {
            Ok(_) => {
                log!(logger, "[ok] Found device at 0x{:02X}", addr);
            }
            Err(e) => {
                // HACK: For ehal 0.2, we rely on string matching the Debug output to detect
                // NACKs, as there is no standardized error kind. This may not be reliable
                // for all HAL implementations.
                let s = {
                    let mut buf: String<128> = String::new();
                    let _ = write!(&mut buf, "{:?}", e);
                    buf
                };
                let is_nack = s.contains("NACK") || s.contains("NoAcknowledge");

                if is_nack {
                    continue;
                } else {
                    log!(logger, "[error] write failed at 0x{:02X}: {:?}", addr, e);
                }
            }
        }
    }

    log!(logger, "[info] I2C scan complete.");

    loop {}
}
