#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, Explorer};
use dvcdbg::logger::{SerialLogger, LogLevel};
use dvcdbg::scanner::run_explorer;
adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 32;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);

    let mut logger = SerialLogger::new(&mut serial, LogLevel::Normal);
    logger.log_info("[SH1107G Stable DFS Enumerate]");

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    logger.log_info("[Info] I2C initialized");

    static EXPLORER_CMDS: [CmdNode; 17] = [
        CmdNode { bytes: &[0xAE], deps: &[] },
        CmdNode { bytes: &[0xD5, 0x51], deps: &[0] },
        CmdNode { bytes: &[0xA8, 0x3F], deps: &[1] },
        CmdNode { bytes: &[0xD3, 0x60], deps: &[2] },
        CmdNode { bytes: &[0x40, 0x00], deps: &[3] },
        CmdNode { bytes: &[0xA1, 0x00], deps: &[4] },
        CmdNode { bytes: &[0xA0], deps: &[5] },
        CmdNode { bytes: &[0xC8], deps: &[6] },
        CmdNode { bytes: &[0xAD, 0x8A], deps: &[7] },
        CmdNode { bytes: &[0xD9, 0x22], deps: &[8] },
        CmdNode { bytes: &[0xDB, 0x35], deps: &[9] },
        CmdNode { bytes: &[0x8D, 0x14], deps: &[10] },
        CmdNode { bytes: &[0xB0], deps: &[11] },
        CmdNode { bytes: &[0x00], deps: &[11] },
        CmdNode { bytes: &[0x10], deps: &[11] },
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let explorer = Explorer { sequence: &EXPLORER_CMDS };
    let prefix: u8 = 0x00;

    logger.log_info("[Info] Starting explorer...");

    match run_explorer(
        &explorer,
        &mut i2c,
        &mut logger,
        &[],
        prefix,
        LogLevel::Normal,
    ) {
        Ok(_) => logger.log_info("[Info] Explorer completed successfully."),
        Err(e) => {
            let _ = write!(&mut serial, "[Error] Explorer failed: {:?}\r\n", e);
        }
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}