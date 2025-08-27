#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 3; // 最大2バイト + prefix

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Full Init Test]").ok();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    static EXPLORER_CMDS: [CmdNode; 14] = [
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
        CmdNode { bytes: &[0xA6], deps: &[11] },
        CmdNode { bytes: &[0xAF], deps: &[12] },
    ];

    let explorer: Explorer<'static, 14, BUF_CAP> = Explorer {
        sequence: &EXPLORER_CMDS,
    };

    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();

    for (i, node) in EXPLORER_CMDS.iter().enumerate() {
        writeln!(
            serial,
            "[Send] Node {} bytes={:02X?} deps={:?}",
            i, node.bytes, node.deps
        )
        .ok();

        if let Err(e) = i2c_write_with_prefix(&mut i2c, 0x3C, 0x00, node.bytes) {
            writeln!(serial, "[Fail] Node {}: {:?}", i, e).ok();
        } else {
            writeln!(serial, "[OK] Node {} sent", i).ok();
        }
    }

    writeln!(serial, "[Info] SH1107G full init test complete").ok();

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// prefix付きI2C書き込み
fn i2c_write_with_prefix<I2C>(
    i2c: &mut I2C,
    addr: u8,
    prefix: u8,
    data: &[u8],
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal::blocking::i2c::Write,
{
    let mut buf = [0u8; BUF_CAP];
    buf[0] = prefix;
    buf[1..=data.len()].copy_from_slice(data);
    i2c.write(addr, &buf[..=data.len()])
}
