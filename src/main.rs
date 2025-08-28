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
    let _ = writeln!(serial, "[SH1107G Auto VRAM Kahn Test]");

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    let _ = writeln!(serial, "[Info] I2C initialized");

    static EXPLORER_CMDS: [CmdNode; 17] = [
        CmdNode { bytes: &[0xAE], deps: &[] },         // 0: Display OFF
        CmdNode { bytes: &[0xD5, 0x51], deps: &[0] },  // 1: Set display clock
        CmdNode { bytes: &[0xA8, 0x3F], deps: &[1] },  // 2: Set multiplex
        CmdNode { bytes: &[0xD3, 0x60], deps: &[2] },  // 3: Set display offset
        CmdNode { bytes: &[0x40, 0x00], deps: &[3] },  // 4: Set start line
        CmdNode { bytes: &[0xA1, 0x00], deps: &[4] },  // 5: Set segment remap
        CmdNode { bytes: &[0xA0], deps: &[5] },        // 6: Set scan direction
        CmdNode { bytes: &[0xC8], deps: &[6] },        // 7: COM scan
        CmdNode { bytes: &[0xAD, 0x8A], deps: &[7] },  // 8: Charge pump
        CmdNode { bytes: &[0xD9, 0x22], deps: &[8] },  // 9: Set precharge
        CmdNode { bytes: &[0xDB, 0x35], deps: &[9] },  //10: Set VCOMH
        CmdNode { bytes: &[0x8D, 0x14], deps: &[10] }, //11: Enable charge pump
        CmdNode { bytes: &[0xB0], deps: &[11] },       //12: Set page start
        CmdNode { bytes: &[0x00], deps: &[11] },       //13: Set lower column
        CmdNode { bytes: &[0x10], deps: &[11] },       //14: Set higher column
        CmdNode { bytes: &[0xA6], deps: &[12,13,14] }, //15: Normal display
        CmdNode { bytes: &[0xAF], deps: &[15] },       //16: Display ON
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    let _ = writeln!(serial, "[Info] Starting auto VRAM Kahn exploration...");

    match run_auto_vram_kahn(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        Ok(seq) => { let _ = writeln!(serial, "[OK] Sequence found: {:?}", seq); },
        Err(e) => { let _ = writeln!(serial, "[Fail] No valid sequence found: {:?}", e); },
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Kahn法でトポロジカルソートしながらI2C送信
/// 失敗ノードの詳細ログを追加
fn run_auto_vram_kahn<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
) -> Result<heapless::Vec<usize, 32>, ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    let n = cmds.len();
    let mut in_degree = [0usize; 32];
    for i in 0..n {
        for &_dep in cmds[i].deps {
            in_degree[i] += 1;
        }
    }

    let mut queue = heapless::Vec::<usize, 32>::new();
    for i in 0..n {
        if in_degree[i] == 0 {
            queue.push(i).ok();
        }
    }

    let mut sequence = heapless::Vec::<usize, 32>::new();

    while let Some(&node) = queue.first() {
        queue.remove(0);
        let cmd = &cmds[node];
        let _ = writeln!(serial, "[Try] Node {} bytes={:02X?}", node, cmd.bytes);

        if cmd.bytes.len() + 1 > BUF_CAP {
            let _ = writeln!(
                serial,
                "[Fail] Node {} buffer overflow bytes={:02X?}",
                node, cmd.bytes
            );
            return Err(ExplorerError::ExecutionFailed);
        }

        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix;
        buf[1..1 + cmd.bytes.len()].copy_from_slice(cmd.bytes);
        if i2c.write(addr, &buf[..1 + cmd.bytes.len()]).is_err() {
            let _ = writeln!(
                serial,
                "[Fail] Node {} I2C write failed bytes={:02X?}",
                node, cmd.bytes
            );
            return Err(ExplorerError::ExecutionFailed);
        }

        sequence.push(node).ok();

        for i in 0..n {
            if cmds[i].deps.contains(&node) {
                in_degree[i] -= 1;
                if in_degree[i] == 0 {
                    queue.push(i).ok();
                }
            }
        }
    }

    if sequence.len() == n {
        Ok(sequence)
    } else {
        let _ = writeln!(serial, "[Fail] Sequence incomplete, visited nodes: {:?}", sequence);
        Err(ExplorerError::ExecutionFailed)
    }
}
