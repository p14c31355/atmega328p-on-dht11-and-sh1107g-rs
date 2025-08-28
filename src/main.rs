#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::CmdNode;
use core::hash::Hasher;
use hash32::FnvHasher;

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 16;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    let _ = writeln!(serial, "[SH1107G Safe Auto VRAM Enumerate]");

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    let _ = writeln!(serial, "[Info] I2C initialized");

    static EXPLORER_CMDS: [CmdNode; 17] = [
        CmdNode { bytes: &[0xAE], deps: &[] },         // DISPLAY OFF
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
        CmdNode { bytes: &[0xAF], deps: &[15] },       // DISPLAY ON
    ];

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;
    let mut visited_hashes = heapless::Vec::<u64, 128>::new();

    loop {
        let mut in_degree = [0usize; 32];
        for i in 0..EXPLORER_CMDS.len() {
            for &dep in EXPLORER_CMDS[i].deps {
                in_degree[i] += 1;
            }
        }

        let mut sequence = heapless::Vec::<usize, 32>::new();
        let mut found_new = false;

        enumerate_and_hash(
            &EXPLORER_CMDS,
            &mut i2c,
            &mut serial,
            addr,
            prefix,
            &mut in_degree,
            &mut sequence,
            &mut visited_hashes,
            &mut found_new,
        );

        if !found_new {
            break;
        }
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

fn enumerate_and_hash<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix: u8,
    in_degree: &mut [usize; 32],
    sequence: &mut heapless::Vec<usize, 32>,
    visited_hashes: &mut heapless::Vec<u64, 128>,
    found_new: &mut bool,
) where
    I2C: embedded_hal::blocking::i2c::Write,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    S: core::fmt::Write,
{
    if sequence.len() == cmds.len() {
        let mut hasher = FnvHasher::default();
        for &node in sequence.iter() {
            hasher.write_usize(node);
        }
        let hash = hasher.finish();

        if !visited_hashes.contains(&hash) {
            visited_hashes.push(hash).ok();
            *found_new = true;

            for &node in sequence.iter() {
                let cmd = &cmds[node];
                let mut buf = [0u8; BUF_CAP];
                buf[0] = prefix;
                buf[1..1 + cmd.bytes.len()].copy_from_slice(cmd.bytes);
                let _ = i2c.write(addr, &buf[..1 + cmd.bytes.len()]);
            }

            let _ = writeln!(serial, "[Seq] {:?}", sequence.as_slice());
        }
        return;
    }

    for node in 0..cmds.len() {
        if in_degree[node] == 0 && !sequence.contains(&node) {
            // DISPLAY OFFは常に最初
            if sequence.is_empty() && node != 0 {
                continue;
            }
            // DISPLAY ONは常に最後
            if sequence.len() == cmds.len() - 1 && node != cmds.len() - 1 {
                continue;
            }

            sequence.push(node).ok();
            for dep_target in 0..cmds.len() {
                if cmds[dep_target].deps.contains(&node) {
                    in_degree[dep_target] -= 1;
                }
            }

            enumerate_and_hash(
                cmds,
                i2c,
                serial,
                addr,
                prefix,
                in_degree,
                sequence,
                visited_hashes,
                found_new,
            );

            for dep_target in 0..cmds.len() {
                if cmds[dep_target].deps.contains(&node) {
                    in_degree[dep_target] += 1;
                }
            }
            sequence.pop();
        }
    }
}
