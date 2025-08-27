#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 4;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Auto Backtrack Test]").ok();

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
        CmdNode { bytes: &[0xB0], deps: &[11] },
        CmdNode { bytes: &[0x00], deps: &[11] },
        CmdNode { bytes: &[0x10], deps: &[11] },
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    writeln!(serial, "[Info] Starting backtrack exploration...").ok();

    let mut sequence = heapless::Vec::<usize, 17>::new();
    let mut visited = [false; 17];

    if backtrack(
        &EXPLORER_CMDS,
        &mut i2c,
        &mut serial,
        0x3C,
        0x00,
        &mut visited,
        &mut sequence,
    ) {
        writeln!(serial, "[OK] Found working sequence: {:?}", sequence).ok();
    } else {
        writeln!(serial, "[Fail] No working sequence found").ok();
    }

    loop { arduino_hal::delay_ms(1000); }
}

fn backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    visited: &mut [bool; 17],
    sequence: &mut heapless::Vec<usize, 17>,
) -> bool
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    // 終端条件
    if sequence.len() == cmds.len() {
        // VRAM 判定
        if verify_vram(i2c, addr) {
            return true;
        } else {
            writeln!(serial, "[Info] Sequence invalid, backtracking...").ok();
            return false;
        }
    }

    for i in 0..cmds.len() {
        if visited[i] { continue; }

        // 依存関係が満たされているか
        if cmds[i].deps.iter().all(|&d| visited[d]) {
            writeln!(serial, "[Try] Node {} bytes={:02X?}", i, cmds[i].bytes).ok();

            let buf_len = 1 + cmds[i].bytes.len();
            if buf_len > BUF_CAP { continue; }
            let mut buf = [0u8; BUF_CAP];
            buf[0] = prefix;
            buf[1..buf_len].copy_from_slice(cmds[i].bytes);

            if i2c.write(addr, &buf[..buf_len]).is_err() {
                writeln!(serial, "[Fail] Node {} write failed", i).ok();
                continue;
            }

            visited[i] = true;
            sequence.push(i).ok();

            if backtrack(cmds, i2c, serial, addr, prefix, visited, sequence) {
                return true;
            }

            // バックトラック
            visited[i] = false;
            sequence.pop();
        }
    }

    false
}

/// VRAM 読み出し + パターン判定（簡易例）
fn verify_vram<I2C>(i2c: &mut I2C, addr: u8) -> bool
where
    I2C: embedded_hal::blocking::i2c::WriteRead,
{
    // 仮に1ページだけ読み出して確認する例
    let mut buf = [0u8; 16];
    let cmd = [0xB0, 0x00, 0x10];
    if i2c.write_read(addr, &cmd, &mut [0u8]).is_err() { return false; }
    // 実際の読み出しは HW に合わせて実装
    true
}
