#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 8; 
const VRAM_CHECK_LEN: usize = 16; 
const VRAM_SAMPLE_ROWS: usize = 4;  
const VRAM_ROW_OFFSET: usize = 16;  
const VRAM_MIN_BITS_SET: usize = 4; // 最低限1行にセットされているビット数

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[SH1107G Auto VRAM BitCheck Backtrack Test]").ok();

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

    writeln!(serial, "[Info] Starting auto VRAM bit-check backtrack exploration...").ok();

    if let Err(e) = run_auto_vram_bitcheck_backtrack(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] No valid sequence found: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Working sequence found!").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

fn run_auto_vram_bitcheck_backtrack<I2C, S>(
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
    let mut sequence = [0usize; 17];

    fn backtrack<I2C, S>(
        cmds: &[CmdNode],
        i2c: &mut I2C,
        serial: &mut S,
        addr: u8,
        prefix: u8,
        sequence: &mut [usize],
        depth: usize,
    ) -> bool
    where
        I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::Read,
        <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
        <I2C as embedded_hal::blocking::i2c::Read>::Error: core::fmt::Debug,
        S: core::fmt::Write,
    {
        if depth == cmds.len() {
            // VRAM サンプル判定
            for row in 0..VRAM_SAMPLE_ROWS {
                let mut buf = [0u8; VRAM_CHECK_LEN];
                if i2c.read(addr, &mut buf).is_err() {
                    writeln!(serial, "[Fail] VRAM read failed").ok();
                    return false;
                }

                // ビットカウント
                let bits_set: usize = buf.iter().map(|b| b.count_ones() as usize).sum();
                if bits_set < VRAM_MIN_BITS_SET {
                    writeln!(serial, "[Info] Row {} too few bits set ({}), backtracking...", row, bits_set).ok();
                    return false;
                }
            }
            return true;
        }

        for idx in 0..cmds.len() {
            if cmds[idx].deps.iter().all(|&dep| sequence[..depth].contains(&dep)) &&
               !sequence[..depth].contains(&idx)
            {
                writeln!(serial, "[Try] Node {} bytes={:02X?}", idx, cmds[idx].bytes).ok();
                arduino_hal::delay_ms(1);

                let buf_len = 1 + cmds[idx].bytes.len();
                if buf_len > BUF_CAP {
                    writeln!(serial, "[Fail] Node {} buffer overflow", idx).ok();
                    continue;
                }
                let mut buf = [0u8; BUF_CAP];
                buf[0] = prefix;
                buf[1..buf_len].copy_from_slice(cmds[idx].bytes);

                if i2c.write(addr, &buf[..buf_len]).is_err() {
                    writeln!(serial, "[Fail] Node {} I2C write failed", idx).ok();
                    continue;
                }

                sequence[depth] = idx;
                if backtrack(cmds, i2c, serial, addr, prefix, sequence, depth + 1) {
                    writeln!(serial, "[OK] Node {} accepted", idx).ok();
                    return true;
                } else {
                    writeln!(serial, "[Info] Sequence invalid, backtracking...").ok();
                }
            }
        }
        false
    }

    if backtrack(cmds, i2c, serial, addr, prefix, &mut sequence, 0) {
        Ok(())
    } else {
        Err(ExplorerError::ExecutionFailed)
    }
}
