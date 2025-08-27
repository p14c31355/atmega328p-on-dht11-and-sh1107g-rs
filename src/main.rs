#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 4; // 最大2バイト + prefix + 余裕
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

    // ---- コマンド配列 ----
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
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM exploration...").ok();

    if let Err(e) = explore_auto(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
    }

    loop { arduino_hal::delay_ms(1000); }
}

// -------------------------------------------------------------
// VRAM 判定
fn check_vram<I2C>(i2c: &mut I2C, addr: u8) -> bool
where
    I2C: embedded_hal::blocking::i2c::WriteRead,
{
    let mut byte = [0u8];
    // ページ0, col0 を読む簡易チェック
    if i2c.write_read(addr, &[0xB0, 0x00, 0x10, 0x40], &mut byte).is_ok() {
        return byte[0] != 0x00 && byte[0] != 0xFF;
    }
    false
}

// -------------------------------------------------------------
// バックトラック探索＋VRAM判定
fn explore_auto<I2C, S>(
    cmds: &[CmdNode; NODES],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> Result<(), ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::WriteRead>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    fn backtrack<I2C, S>(
        cmds: &[CmdNode; NODES],
        i2c: &mut I2C,
        serial: &mut S,
        prefix: u8,
        addr: u8,
        order: &mut heapless::Vec<usize, NODES>,
        sent: &mut [bool; NODES],
    ) -> bool
    where
        I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
        <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
        <I2C as embedded_hal::blocking::i2c::WriteRead>::Error: core::fmt::Debug,
        S: core::fmt::Write,
    {
        for i in 0..NODES {
            if sent[i] { continue; }
            if !cmds[i].deps.iter().all(|&d| sent[d]) { continue; }

            writeln!(serial, "[Try] Node {} bytes={:02X?}", i, cmds[i].bytes).ok();

            // I2C 書き込み
            let mut buf = [0u8; BUF_CAP];
            buf[0] = prefix;
            let len = cmds[i].bytes.len();
            buf[1..=len].copy_from_slice(cmds[i].bytes);
            if i2c.write(addr, &buf[..=len]).is_err() {
                writeln!(serial, "[Fail] Node {} write failed", i).ok();
                continue;
            }

            sent[i] = true;
            order.push(i).ok();

            // VRAM 判定
            // バックトラック探索
if order.len() == NODES {
    // 全部送ったあとに VRAM 判定
    if check_vram(i2c, addr) {
        return true;
    } else {
        // VRAM 不正ならこの順序は失敗
        sent[i] = false;
        order.pop();
        continue;
    }
}


            // バックトラック
            sent[i] = false;
            order.pop();
        }
        false
    }

    let mut order: heapless::Vec<usize, NODES> = heapless::Vec::new();
    let mut sent = [false; NODES];

    if backtrack(cmds, i2c, serial, prefix, addr, &mut order, &mut sent) {
        writeln!(serial, "[OK] Working sequence found: {:?}", &order[..]).ok();
        Ok(())
    } else {
        writeln!(serial, "[Fail] No working sequence found").ok();
        Err(ExplorerError::ExecutionFailed)
    }
}
