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

use ufmt::{uwrite, uwriteln};
use core::fmt::Write; // これ必須

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

    let mut display = Sh1107gBuilder::new(i2c, &mut logger)
        //.with_address(0x3C) // 必要なら指定
        .build();

    // 初期化コマンド列
    let init_cmds: &[u8] = &[
        0xAE,       // DISPLAY_OFF
        0xAD, 0x8B, // CHARGE_PUMP_ON_CMD + CHARGE_PUMP_ON_DATA
        0xA8, 0x7F, // SET_MULTIPLEX_RATIO + MULTIPLEX_RATIO_DATA
        0xD3, 0x60, // DISPLAY_OFFSET_CMD + DISPLAY_OFFSET_DATA
        0x40,       // DISPLAY_START_LINE_CMD
        0xD5, 0x51, // CLOCK_DIVIDE_CMD + CLOCK_DIVIDE_DATA
        0xC0,       // COM_OUTPUT_SCAN_DIR
        0xDA, 0x12, // SET_COM_PINS_CMD + SET_COM_PINS_DATA
        0x81, 0x80, // CONTRAST_CONTROL_CMD + CONTRAST_CONTROL_DATA
        0xD9, 0x22, // PRECHARGE_CMD + PRECHARGE_DATA
        0xDB, 0x35, // VCOM_DESELECT_CMD + VCOM_DESELECT_DATA
        0xA0,       // SEGMENT_REMAP
        0xA4,       // SET_ENTIRE_DISPLAY_ON_OFF_CMD
        0xA6,       // SET_NORMAL_INVERSE_DISPLAY_CMD
        0xAF,       // DISPLAY_ON
    ];

    use heapless::String;
    use core::fmt::Write;
    use dvcdbg::logger::Logger;

    use core::convert::Infallible;

    for &cmd in init_cmds {
    let res = display.send_cmd(cmd);
    let mut buf: heapless::String<64> = heapless::String::new();
    use core::fmt::Write;

    match &res {
        Ok(_) => {
            let _ = write!(buf, "I2C CMD 0x{:02X} sent OK", cmd);
            logger.log_i2c(buf.as_str(), Ok::<(), ()>(()));
        }
        Err(e) => {
            let _ = write!(buf, "I2C CMD 0x{:02X} failed: {:?}", cmd, e);
            logger.log_i2c(buf.as_str(), Err::<(), ()>(()));
        }
    }
}



    display.clear_buffer();
    display.flush().unwrap();

    loop {
        // ここで停止
    }
}
