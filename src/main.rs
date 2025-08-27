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
    writeln!(serial, "[SH1107G Auto VRAM Test]").ok();

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
        CmdNode { bytes: &[0xB0], deps: &[11] },  // Page=0
        CmdNode { bytes: &[0x00], deps: &[11] },  // Col low
        CmdNode { bytes: &[0x10], deps: &[11] },  // Col high
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM exploration...").ok();

    if let Err(e) = run_auto_backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Valid sequence found!").ok();
    }

    loop { arduino_hal::delay_ms(1000); }
}

/// VRAM 書き込み→読み出しで判定
fn verify_vram<I2C>(i2c: &mut I2C, addr: u8) -> bool
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
{
    let test_pattern: [u8; 1] = [0xAA];

    // ページ0, カラム0 に書き込み（簡易例）
    if i2c.write(addr, &test_pattern).is_err() {
        return false;
    }

    let mut buf: [u8; 1] = [0];
    if i2c.read(addr, &mut buf).is_err() {
        return false;
    }

    buf[0] == test_pattern[0]
}

/// バックトラック探索 + VRAM 判定
fn run_auto_backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> Result<(), ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    let mut sequence = heapless::Vec::<usize, 17>::new();
    try_sequence(cmds, i2c, serial, addr, prefix, &mut sequence, 0)?;
    Ok(())
}

fn try_sequence<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    sequence: &mut heapless::Vec<usize, 17>,
    idx: usize,
) -> Result<(), ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    if idx >= cmds.len() {
        // 最終判定
        if verify_vram(i2c, addr) {
            writeln!(serial, "[Info] Sequence valid: {:?}", sequence).ok();
            return Ok(());
        } else {
            return Err(ExplorerError::ExecutionFailed);
        }
    }

    // 依存条件を確認
    for &dep in cmds[idx].deps {
        if !sequence.contains(&dep) {
            return Err(ExplorerError::DependencyCycle);
        }
    }

    writeln!(serial, "[Try] Node {} bytes={:02X?}", idx, cmds[idx].bytes).ok();

    // I2C 書き込み
    let buf_len = 1 + cmds[idx].bytes.len();
    if buf_len > BUF_CAP { return Err(ExplorerError::BufferOverflow); }
    let mut buf = [0u8; BUF_CAP];
    buf[0] = prefix;
    buf[1..buf_len].copy_from_slice(cmds[idx].bytes);

    i2c.write(addr, &buf[..buf_len]).map_err(|_| ExplorerError::ExecutionFailed)?;
    sequence.push(idx).map_err(|_| ExplorerError::BufferOverflow)?;

    // 再帰で次のノードへ
    match try_sequence(cmds, i2c, serial, addr, prefix, sequence, idx + 1) {
        Ok(_) => Ok(()),
        Err(_) => {
            // バックトラック
            sequence.pop();
            Err(ExplorerError::ExecutionFailed)
        }
    }
}
