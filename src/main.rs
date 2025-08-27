#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 8;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    let _ = writeln!(serial, "[SH1107G Auto VRAM Kahn+Backtrack All Patterns]");

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    let _ = writeln!(serial, "[Info] I2C initialized");

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
        CmdNode { bytes: &[0xA6], deps: &[12,13,14] },
        CmdNode { bytes: &[0xAF], deps: &[15] },
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    let _ = writeln!(serial, "[Info] Starting Kahn+Backtrack exploration...");

    let mut all_sequences: heapless::Vec<heapless::Vec<usize,32>,16> = heapless::Vec::new();

    // 探索フェーズ: 書き込みは行わず成功パターンだけを保持
    if run_auto_vram_backtrack_all(&EXPLORER_CMDS, &mut all_sequences) {
        let _ = writeln!(serial, "[Info] Found {} valid sequence(s)", all_sequences.len());

        // 送信フェーズ: 順に I2C に書き込み + 読み出し検証
        for (idx, seq) in all_sequences.iter().enumerate() {
            let _ = writeln!(serial, "[Info] Writing sequence #{}: {:?}", idx+1, seq);
            for &node in seq.iter() {
                let cmd = &EXPLORER_CMDS[node];
                let mut buf = [0u8; BUF_CAP];
                buf[0] = prefix;
                buf[1..1+cmd.bytes.len()].copy_from_slice(cmd.bytes);
                if i2c.write(addr, &buf[..1+cmd.bytes.len()]).is_err() {
                    let _ = writeln!(serial, "[Fail] Node {} write failed in sequence #{}", node, idx+1);
                    break;
                }
                // 読み出し検証
                let mut read_buf = [0u8; BUF_CAP];
                if i2c.read(addr, &mut read_buf[..cmd.bytes.len()]).is_err() {
                    let _ = writeln!(serial, "[Fail] Node {} read failed in sequence #{}", node, idx+1);
                    break;
                }
                if &read_buf[..cmd.bytes.len()] != cmd.bytes {
                    let _ = writeln!(serial,
                        "[Fail] Node {} verification mismatch in sequence #{} expected={:02X?} read={:02X?}",
                        node, idx+1, cmd.bytes, &read_buf[..cmd.bytes.len()]);
                    break;
                }
            }
        }

    } else {
        let _ = writeln!(serial, "[Fail] No valid sequences found");
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

// 探索フェーズ: 書き込みは行わず、全成功シーケンスを all_sequences に保存
fn run_auto_vram_backtrack_all(
    cmds: &[CmdNode],
    all_sequences: &mut heapless::Vec<heapless::Vec<usize,32>,16>,
) -> bool {
    let n = cmds.len();
    let mut in_degree = [0usize;32];
    for i in 0..n { for &_dep in cmds[i].deps { in_degree[i] += 1; } }

    let mut sequence = heapless::Vec::<usize,32>::new();
    let mut visited = [false;32];

    fn backtrack(
        cmds: &[CmdNode],
        in_degree: &mut [usize;32],
        sequence: &mut heapless::Vec<usize,32>,
        visited: &mut [bool;32],
        all_sequences: &mut heapless::Vec<heapless::Vec<usize,32>,16>,
    ) -> bool {
        if sequence.len() == cmds.len() {
            all_sequences.push(sequence.clone()).ok();
            return true;
        }

        let mut candidates = heapless::Vec::<usize,32>::new();
        for i in 0..cmds.len() { if !visited[i] && in_degree[i]==0 { candidates.push(i).ok(); } }

        let mut success = false;
        for &node in candidates.iter() {
            visited[node] = true;
            sequence.push(node).ok();
            for i in 0..cmds.len() { if cmds[i].deps.contains(&node) { in_degree[i]-=1; } }

            if backtrack(cmds, in_degree, sequence, visited, all_sequences) { success = true; }

            visited[node] = false;
            sequence.pop();
            for i in 0..cmds.len() { if cmds[i].deps.contains(&node) { in_degree[i]+=1; } }
        }
        success
    }

    backtrack(cmds, &mut in_degree, &mut sequence, &mut visited, all_sequences)
}
