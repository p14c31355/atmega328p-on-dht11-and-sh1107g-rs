#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Rectangle,
    primitives::PrimitiveStyle,
};
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::logger::SerialLogger;
use dvcdbg::log;
use dvcdbg::logger;

use embedded_hal::serial::Write;
use core::fmt::Write as FmtWrite;

struct FmtWriteWrapper<W>(W);

impl<W> core::fmt::Write for FmtWriteWrapper<W>
where
    W: Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            nb::block!(self.0.write(b)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化（57600bps）
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = FmtWriteWrapper(serial);
    let mut logger = SerialLogger::new(&mut serial_wrapper);

    writeln!(logger, "Start main").ok();

    // I2C 初期化（SDA: A4, SCL: A5）
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    // SH1107G 初期化
    let mut display = Sh1107gBuilder::new(i2c, &mut logger).build();

    writeln!(logger, "Display initialized").ok();

    // 画面全体を白で塗りつぶす
    let white_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let rect = Rectangle::new(Point::new(0, 0), Size::new(128, 128));

    rect.into_styled(white_style).draw(&mut display).unwrap();

    if let Err(e) = display.flush() {
        log!(logger, "ERR: flush failed: {:?}", e).ok();
    } else {
        writeln!(logger, "OK: white screen drawn").ok();
    }

    loop {
        // メインループ何もしない
    }
}

use panic_halt as _;
