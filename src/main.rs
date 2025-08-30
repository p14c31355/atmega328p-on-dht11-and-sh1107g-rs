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

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 9600));
    arduino_hal::delay_ms(1000);

    let mut logger: SerialLogger<'_, _> = SerialLogger::new(&mut serial, LogLevel::Quiet);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    let prefix: u8 = 0x00;
    const MAX_CMD_LEN: usize = 26;
    const INIT_SEQUENCE: [u8; MAX_CMD_LEN] = [
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
            deps: &[0],
        },
        CmdNode {
            bytes: &[0xA8, 0x3F],
            deps: &[1],
        },
        CmdNode {
            bytes: &[0xD3, 0x60],
            deps: &[2],
        },
        CmdNode {
            bytes: &[0x40, 0x00],
            deps: &[3],
        },
        CmdNode {
            bytes: &[0xA1, 0x00],
            deps: &[4],
        },
        CmdNode {
            bytes: &[0xA0],
            deps: &[5],
        },
        CmdNode {
            bytes: &[0xC8],
            deps: &[6],
        },
        CmdNode {
            bytes: &[0xAD, 0x8A],
            deps: &[7],
        },
        CmdNode {
            bytes: &[0xD9, 0x22],
            deps: &[8],
        },
        CmdNode {
            bytes: &[0xDB, 0x35],
            deps: &[9],
        },
        CmdNode {
            bytes: &[0x8D, 0x14],
            deps: &[10],
        },
        CmdNode {
            bytes: &[0xB0],
            deps: &[11],
        },
        CmdNode {
            bytes: &[0x00],
            deps: &[11],
        },
        CmdNode {
            bytes: &[0x10],
            deps: &[11],
        },
        CmdNode {
            bytes: &[0xA6],
            deps: &[12, 13, 14],
        },
        CmdNode {
            bytes: &[0xAF],
            deps: &[15],
        },
    ];

    let explorer: Explorer<'_, 17> = Explorer {
        sequence: &EXPLORER_CMDS,
    };

    let successful_seq = match dvcdbg::scanner::scan_init_sequence(
        &mut i2c,
        &mut logger,
        prefix,
        &INIT_SEQUENCE,
        LogLevel::Verbose,
    ) {
        Ok(seq) => seq,
        Err(e) => {
            logger.log_error_fmt(|buf| {
                write!(
                    buf,
                    "[error] Initial sequence scan failed: {e:?}. Aborting explorer."
                )
            });
            panic!("Initial sequence scan failed.");
        }
    };
    logger.log_info_fmt(|buf| write!(buf, "[log] Start driver safe init"));
    
    
    let mut executor = dvcdbg::explore::explorer::PrefixExecutor::<{INIT_SEQUENCE.len()}, MAX_CMD_LEN>::new(prefix, successful_seq);

    match dvcdbg::explore::runner::run_pruned_explorer::<_, _, _, 17, MAX_CMD_LEN>(
        &explorer,
        &mut i2c,
        &mut executor,
        &mut logger,
        prefix,
        LogLevel::Normal,
    ) {
        Ok(_) => logger.log_info_fmt(|buf| write!(buf, "[I] Explorer OK.")),
        Err(e) => {
            logger.log_error_fmt(|buf| write!(buf, "[E] Explorer failed: {:?}\r\n", e));
        }
    }
    logger.log_info_fmt(|buf| write!(buf, "[D] Enter main loop."));
    loop {
        arduino_hal::delay_ms(1000);
    }
}