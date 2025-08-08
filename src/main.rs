#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::Peripherals;
use panic_halt as _;

use sh1107g_rs::{Sh1107gBuilder};
use dvcdbg::logger::SerialLogger;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    Drawable,
};

use core::fmt::Write;
use nb::block;

// UARTラッパー
use embedded_io::Write as EmbeddedWrite; // embedded-io の Write トレイトを利用
use core::fmt::Write as FmtWrite;

struct FmtWriteWrapper<W>(W);

impl<W> FmtWriteWrapper<W>
where
    W: EmbeddedWrite,
{
    pub fn new(w: W) -> Self {
        Self(w)
    }
}

impl<W> FmtWrite for FmtWriteWrapper<W>
where
    W: EmbeddedWrite,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // Writer は embedded_io::Write なので write_all によりバイト列を送信
        self.0.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)?;
        Ok(())
    }
}


#[arduino_hal::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // UART 57600bps 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    // I2C初期化 (SCL=A5, SDA=A4)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a5.into_pull_up_input(),
        pins.a4.into_pull_up_input(),
        100_000,
    );

    // SH1107g 初期化ビルダー
    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        //.with_address(0x3C) // 必要なら指定
        .build();

    // 初期化コマンド列（例、実際は適宜修正）
    let init_cmds: &[u8] = &[
        0xAE, 0xAD, 0x8B, 0xA8, 0x7F, 0xD3, 0x60, 0x40,
        0xD5, 0x51, 0xC0, 0xDA, 0x12, 0x81, 0x80, 0xD9,
        0x22, 0xDB, 0x35, 0xA0, 0xA4, 0xA6, 0xAF,
    ];

    for &cmd in init_cmds {
        let res = display.send_cmd(cmd);
        match res {
            Ok(_) => {
                logger.log_fmt(format_args!("✅ CMD 0x{:02X} sent OK\n", cmd));
            }
            Err(e) => {
                logger.log_fmt(format_args!("❌ CMD 0x{:02X} send FAILED: {:?}\n", cmd, e));
            }
        }
    }

    // バッファを真っ白（全ビット1）で埋める
    display.clear_buffer();
    for b in display.buffer.iter_mut() {
        *b = 0xFF;
    }

    // 画面に反映
    display.flush().unwrap();

    loop {
        // 無限ループ
    }
}
