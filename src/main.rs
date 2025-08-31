// main.rs

#![no_std]
#![no_main]

use core::fmt::Write;
use dvcdbg::explore::explorer::{CmdNode, Explorer};
use dvcdbg::explore::logger::{LogLevel, SerialLogger};
use dvcdbg::prelude::*;
use panic_abort as _;
adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 115200));
    arduino_hal::delay_ms(1000);

    let mut logger: SerialLogger<'_, _> = SerialLogger::new(&mut serial, LogLevel::Verbose);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    match i2c.write(0x3C, &[0x00]) {
        Ok(_) => logger.log_info_fmt(|buf| write!(buf, "I2C OK.")),
        Err(_) => logger.log_error_fmt(|buf| write!(buf, "I2C failed.")),
    };
    arduino_hal::delay_ms(1000);
    
    arduino_hal::delay_ms(1000);

    // This is the count of commands in INIT_SEQUENCE
    const INIT_SEQUENCE_LEN: usize = 26; 
    // This is the maximum length of any single command's byte array in EXPLORER_CMDS.
    // Looking at EXPLORER_CMDS, the longest is &[0xD5, 0x51] which has length 2.
    const MAX_BYTES_PER_CMD: usize = 2; 

    const INIT_SEQUENCE: [u8; INIT_SEQUENCE_LEN] = [
        0xAE, 0xD5, 0x51, 0xA8, 0x3F, 0xD3, 0x60, 0x40, 0x00, 0xA1, 0x00, 0xA0, 0xC8, 0xAD, 0x8A,
        0xD9, 0x22, 0xDB, 0x35, 0x8D, 0x14, 0xB0, 0x00, 0x10, 0xA6, 0xAF,
    ];

    static EXPLORER_CMDS: [CmdNode; 17] = [
        CmdNode {
            bytes: &[0xAE],
            deps: &[],
        },
        CmdNode {
            bytes: &[0xD5, 0x51],
            deps: &[0u8],
        },
        CmdNode {
            bytes: &[0xA8, 0x3F],
            deps: &[1u8],
        },
        CmdNode {
            bytes: &[0xD3, 0x60],
            deps: &[2u8],
        },
        CmdNode {
            bytes: &[0x40, 0x00],
            deps: &[3u8],
        },
        CmdNode {
            bytes: &[0xA1, 0x00],
            deps: &[4u8],
        },
        CmdNode {
            bytes: &[0xA0],
            deps: &[5u8],
        },
        CmdNode {
            bytes: &[0xC8],
            deps: &[6u8],
        },
        CmdNode {
            bytes: &[0xAD, 0x8A],
            deps: &[7u8],
        },
        CmdNode {
            bytes: &[0xD9, 0x22],
            deps: &[8u8],
        },
        CmdNode {
            bytes: &[0xDB, 0x35],
            deps: &[9u8],
        },
        CmdNode {
            bytes: &[0x8D, 0x14],
            deps: &[10u8],
        },
        CmdNode {
            bytes: &[0xB0],
            deps: &[11u8],
        },
        CmdNode {
            bytes: &[0x00],
            deps: &[11u8],
        },
        CmdNode {
            bytes: &[0x10],
            deps: &[11u8],
        },
        CmdNode {
            bytes: &[0xA6],
            deps: &[12u8, 13u8, 14u8],
        },
        CmdNode {
            bytes: &[0xAF],
            deps: &[15u8],
        },
    ];

    let explorer: Explorer<'_, 17> = Explorer {
        sequence: &EXPLORER_CMDS,
    };

    let prefix: u8 = 0x00;
    // let _ = dvcdbg::scanner::scan_init_sequence(&mut i2c, &mut logger, prefix, &INIT_SEQUENCE);
    match dvcdbg::explore::runner::run_pruned_explorer::<_, _, {EXPLORER_CMDS.len()}, INIT_SEQUENCE_LEN, MAX_BYTES_PER_CMD>(
    &explorer,
    &mut i2c,
    &mut logger,
    prefix,
    &INIT_SEQUENCE,
    LogLevel::Verbose,
) {
    Ok(_) => logger.log_info_fmt(|buf| write!(buf, "[I] Explorer OK.")),
    Err(e) => {
        logger.log_error_fmt(|buf| write!(buf, "[E] Explorer failed: {:?}\r\n", e));
    }
}
    logger.log_info_fmt(|buf| write!(buf, "Enter main loop."));
    loop {
        arduino_hal::delay_ms(1000);
    }
}