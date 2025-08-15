#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::prelude::*;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Rectangle, PrimitiveStyle},
};
use sh1107g_rs::Sh1107gBuilder;
use dvcdbg::{log, logger::SerialLogger};
use dvcdbg::logger::Logger;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化（57600bps）
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    loop {
        // メインループ
    }
}
