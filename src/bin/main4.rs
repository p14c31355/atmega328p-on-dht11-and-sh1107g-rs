#![no_std]
#![no_main]

use panic_halt as _; // panic時は停止
use arduino_hal::prelude::*;
use core::fmt::Write;
use nb::block;
use dvcdbg::prelude::*;

/// # adapt_serial!
///
/// Arduino HAL の USART をラップして
/// `core::fmt::Write` を実装するマクロ
#[macro_export]
macro_rules! adapt_serial {
    ($wrapper:ident) => {
        pub struct $wrapper<T>(pub T);

        impl<T> $wrapper<T> {
            pub fn new(inner: T) -> Self {
                Self(inner)
            }
        }

        impl<T, E> core::fmt::Write for $wrapper<T>
        where
            T: embedded_hal::serial::Write<Word = u8, Error = E>,
        {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                for &b in s.as_bytes() {
                    block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
                }
                Ok(())
            }
        }
    };
}

// USART用のアダプタ型を作成
adapt_serial!(UsartAdapter);

#[arduino_hal::entry]
fn main() -> ! {
    // ボードのペリフェラルを取得
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化 (57600 baud)
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // アダプタに包んで core::fmt::Write に変換
    let mut uart = UsartAdapter::new(serial);

    // dvcdbg の Logger として使える
    writeln!(uart, "🔧 Hello from dvcdbg + Arduino!").ok();

    // I2Cスキャナを使ってみる例
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100000, // 100kHz
    );

    // I2Cスキャン結果をUARTに出力
    let devices = scan_i2c(&i2c);
    for dev in devices {
        writeln!(uart, "Found I2C device at 0x{:02X}", dev).ok();
    }

    loop {
        // メインループ
    }
}
