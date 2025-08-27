#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 4; // 最大2バイト + prefix +余裕

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Backtrack Init Test]").ok();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- コマンド配列 ----
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
        CmdNode { bytes: &[0xD9, 0x22], deps: &[] },
        CmdNode { bytes: &[0xDB, 0x35], deps: &[9] },
        CmdNode { bytes: &[0x8D, 0x14], deps: &[10] },
        CmdNode { bytes: &[0xA6], deps: &[11] },
        CmdNode { bytes: &[0xAF], deps: &[12] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    let mut sent = [false; EXPLORER_CMDS.len()];
    let mut sequence = [0usize; EXPLORER_CMDS.len()];

    writeln!(serial, "[Info] Starting backtrack exploration...").ok();

    if explore_recursive(&EXPLORER_CMDS, &mut sent, &mut sequence, 0, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[OK] Exploration complete! Order: {:?}", sequence).ok();
    } else {
        writeln!(serial, "[Fail] No valid sequence found").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// 再帰バックトラック探索
fn explore_recursive<I2C, S>(
    cmds: &[CmdNode],
    sent: &mut [bool],
    sequence: &mut [usize],
    depth: usize,
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> bool
where
    I2C: embedded_hal::blocking::i2c::Write,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    if depth == cmds.len() {
        // 全コマンド送信成功
        return true;
    }

    for idx in 0..cmds.len() {
        if sent[idx] { continue; }
        if !cmds[idx].deps.iter().all(|&d| sent[d]) { continue; }

        writeln!(serial, "[Try] Node {} bytes={:02X?} deps={:?}", idx, cmds[idx].bytes, cmds[idx].deps).ok();

        // バッファ作成
        let buf_len = 1 + cmds[idx].bytes.len();
        if buf_len > BUF_CAP {
            writeln!(serial, "[Fail] Node {}: buffer overflow", idx).ok();
            continue;
        }
        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix;
        buf[1..buf_len].copy_from_slice(cmds[idx].bytes);

        // I2C 書き込み
        if i2c.write(addr, &buf[..buf_len]).is_ok() {
            writeln!(serial, "[OK] Node {} sent", idx).ok();
            sent[idx] = true;
            sequence[depth] = idx;

            // 再帰呼び出し
            if explore_recursive(cmds, sent, sequence, depth + 1, i2c, serial, addr, prefix) {
                return true;
            }

            // バックトラック
            writeln!(serial, "[Backtrack] Node {}", idx).ok();
            sent[idx] = false;
        } else {
            writeln!(serial, "[Fail] Node {} I2C write failed", idx).ok();
        }
    }

    false
}
