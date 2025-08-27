#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 32; // 最大2バイト + prefix +余裕

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Auto VRAM Backtrack Test]").ok();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

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
        CmdNode { bytes: &[0xB0], deps: &[11] }, // Page=0
        CmdNode { bytes: &[0x00], deps: &[11] }, // Col low
        CmdNode { bytes: &[0x10], deps: &[11] }, // Col high
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] }, // Normal display
        CmdNode { bytes: &[0xAF], deps: &[15] }, // Display ON
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM backtrack exploration...").ok();

    match auto_vram_backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        Ok(seq) => {
            writeln!(serial, "[OK] Found working sequence! Order: {:?}", seq).ok();
        }
        Err(e) => {
            writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
        }
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// VRAM を読み取って動作確認する自動バックトラック探索
fn auto_vram_backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> Result<heapless::Vec<usize, 17>, ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    use heapless::Vec;
    let mut order: Vec<usize, 17> = Vec::new();

    fn backtrack<I2C, S>(
        cmds: &[CmdNode],
        i2c: &mut I2C,
        serial: &mut S,
        prefix: u8,
        addr: u8,
        idx: usize,
        used: &mut [bool; 17],
        order: &mut Vec<usize, 17>,
    ) -> bool
    where
        I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
        <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
        <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
        S: core::fmt::Write,
    {
        if order.len() == cmds.len() {
            // VRAM チェック
            let mut buf = [0u8; 1];
            if i2c.read(addr, &mut buf).is_ok() {
                // 適当な判定: 0x00 / 0xFF 以外ならOKと仮定
                if buf[0] != 0x00 && buf[0] != 0xFF {
                    return true;
                }
            }
            return false;
        }

        for i in 0..cmds.len() {
            if used[i] {
                continue;
            }
            // 依存条件チェック
            if cmds[i].deps.iter().any(|&d| !used[d]) {
                continue;
            }

            writeln!(serial, "[Try] Node {} bytes={:02X?}", i, cmds[i].bytes).ok();

            let buf_len = 1 + cmds[i].bytes.len();
            if buf_len > BUF_CAP {
                writeln!(serial, "[Fail] Node {} buffer overflow", i).ok();
                continue;
            }
            let mut buf = [0u8; BUF_CAP];
            buf[0] = prefix;
            buf[1..buf_len].copy_from_slice(cmds[i].bytes);

            if i2c.write(addr, &buf[..buf_len]).is_err() {
                writeln!(serial, "[Fail] Node {} I2C write failed", i).ok();
                continue;
            }

            used[i] = true;
            order.push(i).ok();

            if backtrack(cmds, i2c, serial, prefix, addr, idx + 1, used, order) {
                return true;
            }

            // バックトラック
            used[i] = false;
            order.pop();
        }
        false
    }

    let mut used = [false; 17];
    if backtrack(cmds, i2c, serial, prefix, addr, 0, &mut used, &mut order) {
        Ok(order)
    } else {
        Err(ExplorerError::ExecutionFailed)
    }
}
