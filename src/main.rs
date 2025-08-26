#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use dvcdbg::prelude::*;
use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::BinaryColor,
    Pixel,
};
use sh1107g_rs::{Sh1107gBuilder, error::Sh1107gError};

// UnoWrapper 生成と fmt::Write 対応
adapt_serial!(UnoWrapper);
dvcdbg::prelude::impl_fmt_write!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let i2c = arduino_hal::i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // シリアル初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut logger = UnoWrapper(serial);

    // SH1107G 初期化
    let mut display = Sh1107gBuilder::new(i2c)
        .clear_on_init(true)
        .build();

    let _ = writeln!(logger, "[log] Init SH1107G...");

    if let Err(e) = display.init() {
        log_error(&mut logger, "init failed", &e);
    }

    // 左上 8x8 ドット描画テスト
    for y in 0..8 {
        for x in 0..8 {
            let _ = display.draw_iter(core::iter::once(
                Pixel(Point::new(x, y), BinaryColor::On),
            ));
        }
    }

    if let Err(e) = display.flush() {
        log_error(&mut logger, "flush failed", &e);
    }

    loop {
        delay.delay_ms(1000u16);
    }
}

// SH1107G エラーを UnoWrapper に表示
fn log_error<E, W>(logger: &mut UnoWrapper<W>, msg: &str, err: &Sh1107gError<E>)
where
    E: core::fmt::Debug,
    W: embedded_io::Write + dvcdbg::compat::SerialCompat + core::fmt::Write,
{
    let _ = writeln!(logger, "[error] {}: {:?}", msg, err);
}
