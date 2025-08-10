#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
};
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::{log, logger::SerialLogger};

use embedded_hal::serial::Write;

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

    log!(logger, "🚀 Start main");

    // I2C初期化 (SDA:A4, SCL:A5)
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        400_000,
    );

    // SH1107G 初期化 (build_logger()で初期化成功失敗も検知)
    let mut display = match Sh1107gBuilder::new(i2c, &mut logger).build_logger() {
        Ok(d) => d,
        Err(e) => {
            log!(logger, "❌ SH1107G initialization failed: {:?}", e);
            loop {}
        }
    };

    // 画面全体を白で塗りつぶし
    let white_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let rect = Rectangle::new(Point::new(0, 0), Size::new(128, 128));

    // display.logger を借用してログ出す
    // Sh1107g に with_logger メソッドがある前提

    
        log!(logger, "🎨 Drawing full white rectangle...");
    

    if let Err(e) = rect.into_styled(white_style).draw(&mut display) {
            log!(logger, "❌ Drawing failed: {:?}", e);
    } else {
            log!(logger, "✅ Drawing succeeded");
    }

        log!(logger, "📡 Flushing buffer to display...");

    if let Err(e) = display.flush() {
            log!(logger, "❌ Flush failed: {:?}", e);
    } else {
            log!(logger, "✅ Flush succeeded, display updated");
    }

        log!(logger, "🔄 Entering main loop");

    loop {
        // メインループ
    }
}

use panic_halt as _;
