#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 2; // prefix + 1バイト送信用

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

    // ---- コマンド配列 ----
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

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM backtrack exploration...").ok();

    if let Err(e) = run_backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix, 0, &mut [false;17]) {
        writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// バッファを小さくして分割送信するバックトラック探索
fn run_backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    idx: usize,
    sent: &mut [bool; 17],
) -> Result<(), ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    if idx >= cmds.len() {
        // 全ノード送信完了
        writeln!(serial, "[OK] Found working sequence! Order: {:?}", sent.iter().enumerate().filter(|(_, &b)| b).map(|(i, _)| i).collect::<heapless::Vec<usize,17>>()).ok();
        return Ok(());
    }

    // 依存が未送信のノードはスキップ
    if cmds[idx].deps.iter().any(|&d| !sent[d]) {
        return run_backtrack(cmds, i2c, serial, addr, prefix, idx + 1, sent);
    }

    writeln!(serial, "[Try] Node {} bytes={:02X?}", idx, cmds[idx].bytes).ok();

    // 分割送信
    for &byte in cmds[idx].bytes.iter() {
        let buf = [prefix, byte];
        i2c.write(addr, &buf).map_err(|_| ExplorerError::ExecutionFailed)?;
    }

    sent[idx] = true;

    let res = run_backtrack(cmds, i2c, serial, addr, prefix, idx + 1, sent);

    if res.is_err() {
        // バックトラック
        sent[idx] = false;
    }

    res
}
