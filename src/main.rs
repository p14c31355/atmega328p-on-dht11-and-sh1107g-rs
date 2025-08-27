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
    writeln!(serial, "[SH1107G Kahn Init Test]").ok();

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
    // ← ここまでは依存直列
    CmdNode { bytes: &[0xB0], deps: &[11] },  // Page=0
    CmdNode { bytes: &[0x00], deps: &[11] },  // Col low (同じ親を参照)
    CmdNode { bytes: &[0x10], deps: &[11] },  // Col high
    CmdNode { bytes: &[0xA6], deps: &[12,13,14] }, // Normal depends on all addr-set
    CmdNode { bytes: &[0xAF], deps: &[15] },  // Display ON
];



    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    writeln!(serial, "[Info] Starting Kahn exploration...").ok();

    if let Err(e) = run_kahn(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] Explorer failed: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Kahn exploration complete").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Kahn 法によるトポロジカルソート + I2C送信
fn run_kahn<I2C, S>(
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
    let mut in_deg = [0usize; 14];
    let mut sent = [false; 14];

    // 入次数を計算
    for (i, cmd) in cmds.iter().enumerate() {
        in_deg[i] = cmd.deps.len();
    }

    let mut queue = heapless::Vec::<usize, 14>::new();
    for i in 0..n {
        if in_deg[i] == 0 {
            queue.push(i).ok();
        }
    }

    while !queue.is_empty() {
        let idx = queue.remove(0); // FIFO
        writeln!(serial, "[Send] Node {} bytes={:02X?} deps={:?}", idx, cmds[idx].bytes, cmds[idx].deps).ok();

        // I2C 書き込み
        let buf_len = 1 + cmds[idx].bytes.len();
        if buf_len > BUF_CAP {
            writeln!(serial, "[Fail] Node {} buffer overflow", idx).ok();
            return Err(ExplorerError::BufferOverflow);
        }
        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix;
        buf[1..buf_len].copy_from_slice(cmds[idx].bytes);

        if i2c.write(addr, &buf[..buf_len]).is_err() {
            writeln!(serial, "[Fail] Node {} I2C write failed", idx).ok();
            return Err(ExplorerError::ExecutionFailed);
        }

        writeln!(serial, "[OK] Node {} sent", idx).ok();
        sent[idx] = true;

        // idx を依存するノードの in_deg を減らす
        for j in 0..n {
            if cmds[j].deps.contains(&idx) {
                in_deg[j] -= 1;
                if in_deg[j] == 0 {
                    queue.push(j).ok();
                }
            }
        }
    }

    if sent.iter().all(|&x| x) {
        Ok(())
    } else {
        Err(ExplorerError::DependencyCycle)
    }
}
