#![no_std]
#![no_main]

use core::fmt::Write as FmtWrite;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::CmdNode;

use embedded_io::{Read, Write as IoWrite}; // ← embedded-io traits

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 8;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    FmtWrite::write_fmt(&mut serial, format_args!("[SH1107G Auto VRAM Kahn+Read+Backtrack All Patterns]\n")).ok();
    IoWrite::flush(&mut serial).ok();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    FmtWrite::write_fmt(&mut serial, format_args!("[Info] I2C initialized\n")).ok();
    IoWrite::flush(&mut serial).ok();

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

    FmtWrite::write_fmt(&mut serial, format_args!("[Info] Starting auto VRAM Kahn+Read+Backtrack exploration...\n")).ok();
    IoWrite::flush(&mut serial).ok();

    let mut valid_sequences: heapless::Vec<heapless::Vec<usize,32>,16> = heapless::Vec::new();

    run_explorer(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix, &mut valid_sequences);

    if valid_sequences.is_empty() {
        FmtWrite::write_fmt(&mut serial, format_args!("[Fail] No valid sequences found\n")).ok();
        IoWrite::flush(&mut serial).ok();
    } else {
        FmtWrite::write_fmt(&mut serial, format_args!("[Info] Found {} valid sequence(s)\n", valid_sequences.len())).ok();
        IoWrite::flush(&mut serial).ok();
        for (idx, seq) in valid_sequences.iter().enumerate() {
            FmtWrite::write_fmt(&mut serial, format_args!("[Valid Sequence #{}] {:?}\n", idx+1, seq)).ok();
            IoWrite::flush(&mut serial).ok();
        }
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Kahn法 + バックトラック探索 + I2C検証
fn run_explorer<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    valid_sequences: &mut heapless::Vec<heapless::Vec<usize,32>,16>,
)
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
    S: FmtWrite + IoWrite,
{
    let n = cmds.len();
    let mut in_degree = [0usize;32];
    for i in 0..n {
        for &_dep in cmds[i].deps {
            in_degree[i] += 1;
        }
    }

    let mut sequence = heapless::Vec::<usize,32>::new();
    let mut visited = [false;32];

    fn backtrack<I2C, S>(
        cmds: &[CmdNode],
        i2c: &mut I2C,
        serial: &mut S,
        addr: u8,
        prefix: u8,
        in_degree: &mut [usize;32],
        sequence: &mut heapless::Vec<usize,32>,
        visited: &mut [bool;32],
        valid_sequences: &mut heapless::Vec<heapless::Vec<usize,32>,16>,
    ) -> bool
    where
        I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
        S: FmtWrite + IoWrite,
    {
        if sequence.len() == cmds.len() {
            // ---- シーケンス検証 ----
            let mut ok = true;
            for &node in sequence.iter() {
                let cmd = &cmds[node];
                let mut buf = [0u8; BUF_CAP];
                buf[0] = prefix;
                buf[1..1+cmd.bytes.len()].copy_from_slice(cmd.bytes);

                // write
                if i2c.write(addr, &buf[..1+cmd.bytes.len()]).is_err() {
                    FmtWrite::write_fmt(serial, format_args!("[Fail] Node {} write failed\n", node)).ok();
                    IoWrite::flush(serial).ok();
                    ok = false;
                    break;
                }

                // read-back (using write_read: send prefix, then read)
                let mut read_buf = [0u8; BUF_CAP];
                if i2c.write_read(addr, &[prefix], &mut read_buf[..cmd.bytes.len()]).is_err() {
                    FmtWrite::write_fmt(serial, format_args!("[Fail] Node {} read failed\n", node)).ok();
                    IoWrite::flush(serial).ok();
                    ok = false;
                    break;
                }

                if &read_buf[..cmd.bytes.len()] != cmd.bytes {
                    FmtWrite::write_fmt(serial,
                        format_args!("[Fail] Node {} mismatch expected={:02X?} read={:02X?}\n",
                        node, cmd.bytes, &read_buf[..cmd.bytes.len()])).ok();
                    IoWrite::flush(serial).ok();
                    ok = false;
                    break;
                }
            }
            if ok {
                FmtWrite::write_fmt(serial, format_args!("[Success] Sequence {:?} passed\n", sequence)).ok();
                IoWrite::flush(serial).ok();
                valid_sequences.push(sequence.clone()).ok();
            }
            return ok;
        }

        // ---- Kahn候補列挙 ----
        let mut candidates = heapless::Vec::<usize,32>::new();
        for i in 0..cmds.len() {
            if !visited[i] && in_degree[i]==0 {
                candidates.push(i).ok();
            }
        }

        let mut success = false;
        for &node in candidates.iter() {
            visited[node] = true;
            sequence.push(node).ok();
            for i in 0..cmds.len() {
                if cmds[i].deps.contains(&node) { in_degree[i] -= 1; }
            }

            if backtrack(cmds, i2c, serial, addr, prefix, in_degree, sequence, visited, valid_sequences) {
                success = true;
            }

            visited[node] = false;
            sequence.pop();
            for i in 0..cmds.len() {
                if cmds[i].deps.contains(&node) { in_degree[i] += 1; }
            }
        }
        success
    }

    backtrack(cmds, i2c, serial, addr, prefix, &mut in_degree, &mut sequence, &mut visited, valid_sequences);
}
