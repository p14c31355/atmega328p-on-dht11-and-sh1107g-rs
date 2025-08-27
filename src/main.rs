#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 4; // 最大2バイト + prefix +余裕
const NODES: usize = 17;

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

    static EXPLORER_CMDS: [CmdNode; NODES] = [
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
        CmdNode { bytes: &[0xA6], deps: &[12,13,14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM backtrack exploration...").ok();

    let mut sent = [false; NODES];
    let mut order = heapless::Vec::<usize, NODES>::new();

    if backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix, &mut sent, &mut order) {
        writeln!(serial, "[OK] Found working sequence! Order: {:?}", order).ok();
    } else {
        writeln!(serial, "[Fail] No valid sequence found").ok();
    }

    loop { arduino_hal::delay_ms(1000); }
}

/// 再帰バックトラック探索
fn backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    sent: &mut [bool; NODES],
    order: &mut heapless::Vec<usize, NODES>,
) -> bool
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    if order.len() == NODES {
        // 完全な順序ができたら VRAM 判定
        if check_vram(i2c, addr) {
            return true;
        }
        return false;
    }

    for i in 0..cmds.len() {
        if sent[i] { continue; }
        // 依存がすべて満たされているか確認
        if !cmds[i].deps.iter().all(|&d| sent[d]) { continue; }

        writeln!(serial, "[Try] Node {} bytes={:02X?}", i, cmds[i].bytes).ok();

        let buf_len = 1 + cmds[i].bytes.len();
        if buf_len > BUF_CAP { continue; }

        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix;
        buf[1..buf_len].copy_from_slice(cmds[i].bytes);

        if i2c.write(addr, &buf[..buf_len]).is_err() {
            writeln!(serial, "[Fail] Node {} I2C write failed", i).ok();
            continue;
        }

        sent[i] = true;
        order.push(i).ok();

        if backtrack(cmds, i2c, serial, addr, prefix, sent, order) {
            return true;
        }

        // バックトラック
        sent[i] = false;
        order.pop();
    }
    false
}

/// VRAM が初期化されているか判定する関数
fn check_vram<I2C>(i2c: &mut I2C, addr: u8) -> bool
where
    I2C: embedded_hal::blocking::i2c::Read + embedded_hal::blocking::i2c::Write,
{
    let mut buf = [0u8; 1];
    // 例: アドレス0 の VRAM を読む
    if i2c.write(addr, &[0x00]).is_err() { return false; } // コラムアドレスセット
    if i2c.read(addr, &mut buf).is_err() { return false; }

    // 仮判定: 0x00 以外が返れば初期化済み
    buf[0] != 0x00
}
