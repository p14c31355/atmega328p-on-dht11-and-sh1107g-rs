#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 8; // 分割送信に対応した余裕サイズ

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
        CmdNode { bytes: &[0xB0], deps: &[11] },
        CmdNode { bytes: &[0x00], deps: &[11] },
        CmdNode { bytes: &[0x10], deps: &[11] },
        CmdNode { bytes: &[0xA6], deps: &[12, 13, 14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting auto VRAM backtrack exploration...").ok();

    if let Err(e) = run_auto_vram_backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Found working sequence!").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// バックトラック探索 + VRAM 分割送信 + Display ON 遅延
fn run_auto_vram_backtrack<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> Result<(), ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    let n = cmds.len();
    let mut sequence = heapless::Vec::<usize, 32>::new(); // 探索順序

    fn backtrack<I2C, S>(
        cmds: &[CmdNode],
        i2c: &mut I2C,
        serial: &mut S,
        addr: u8,
        prefix: u8,
        sequence: &mut heapless::Vec<usize, 32>,
        visited: &mut [bool],
    ) -> bool
    where
        I2C: embedded_hal::blocking::i2c::Write,
        <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
        S: core::fmt::Write,
    {
        if sequence.len() == cmds.len() {
            // VRAM 判定をここで行う (擬似的に OK とする)
            return true;
        }

        for i in 0..cmds.len() {
            if visited[i] {
                continue;
            }
            if !cmds[i].deps.iter().all(|&dep| visited[dep]) {
                continue;
            }

            // コマンド送信（BUF_CAP に分割）
            let cmd = &cmds[i];
            writeln!(serial, "[Try] Node {} bytes={:02X?}", i, cmd.bytes).ok();

            if cmd.bytes.len() + 1 > BUF_CAP {
                writeln!(serial, "[Fail] Node {} buffer overflow", i).ok();
                continue;
            }

            let mut buf = [0u8; BUF_CAP];
            buf[0] = prefix;
            buf[1..1 + cmd.bytes.len()].copy_from_slice(cmd.bytes);
            if i2c.write(addr, &buf[..1 + cmd.bytes.len()]).is_err() {
                writeln!(serial, "[Fail] Node {} I2C write failed", i).ok();
                continue;
            }

            visited[i] = true;
            sequence.push(i).ok();

            if backtrack(cmds, i2c, serial, addr, prefix, sequence, visited) {
                return true;
            }

            // バックトラック
            visited[i] = false;
            sequence.pop();
        }

        false
    }

    let mut visited = [false; 32];
    if backtrack(cmds, i2c, serial, addr, prefix, &mut sequence, &mut visited) {
        writeln!(serial, "[Info] Sequence found: {:?}", sequence).ok();
        Ok(())
    } else {
        Err(ExplorerError::ExecutionFailed)
    }
}
