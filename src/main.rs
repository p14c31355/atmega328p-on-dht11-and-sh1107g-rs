#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 12; // 分割送信に対応した余裕サイズ
const WIDTH: usize = 128;
const HEIGHT: usize = 128;
const PAGE_COUNT: usize = HEIGHT / 8;
const VRAM_SIZE: usize = WIDTH * PAGE_COUNT;

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
    let prefix_cmd: u8 = 0x00;
    let prefix_data: u8 = 0x40;

    let _ = writeln!(serial, "[Info] Starting auto VRAM Kahn exploration...");

    match run_auto_vram_kahn(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix_cmd, prefix_data) {
        Ok(seq) => { let _ = writeln!(serial, "[OK] Sequence found: {:?}", seq); },
        Err(e) => { let _ = writeln!(serial, "[Fail] No valid sequence found: {:?}", e); },
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Kahn法でトポロジカルソートしながらI2C送信 & VRAM完全一致検証
fn run_auto_vram_kahn<I2C, S>(
    cmds: &[CmdNode],
    i2c: &mut I2C,
    serial: &mut S,
    addr: u8,
    prefix_cmd: u8,
    prefix_data: u8,
) -> Result<heapless::Vec<usize, 32>, ExplorerError>
where
    I2C: embedded_hal::blocking::i2c::Write + embedded_hal::blocking::i2c::WriteRead,
    <I2C as embedded_hal::blocking::i2c::Write>::Error: core::fmt::Debug,
    <I2C as embedded_hal::blocking::i2c::WriteRead>::Error: core::fmt::Debug,
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
            let _ = writeln!(serial, "[Fail] Node {} buffer overflow", node);
            return Err(ExplorerError::ExecutionFailed);
        }

        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix_cmd;
        buf[1..1 + cmd.bytes.len()].copy_from_slice(cmd.bytes);
        if i2c.write(addr, &buf[..1 + cmd.bytes.len()]).is_err() {
            let _ = writeln!(serial, "[Fail] Node {} I2C write failed", node);
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

    if sequence.len() != n {
        let _ = writeln!(serial, "[Fail] Sequence incomplete, visited nodes: {:?}", sequence);
        return Err(ExplorerError::ExecutionFailed);
    }

    // === VRAM完全一致検証 ===
    let _ = writeln!(serial, "[Info] Starting VRAM full check...");
    let test_pattern: u8 = 0xAA;

    // 全ページにパターン書き込み
    for page in 0..PAGE_COUNT {
        let set_page = [prefix_cmd, 0xB0 | (page as u8)];
        let _ = i2c.write(addr, &set_page);

        let set_col_low = [prefix_cmd, 0x00];
        let _ = i2c.write(addr, &set_col_low);
        let set_col_high = [prefix_cmd, 0x10];
        let _ = i2c.write(addr, &set_col_high);

        let mut line = [0u8; 1 + WIDTH];
        line[0] = prefix_data;
        for b in line[1..].iter_mut() {
            *b = test_pattern;
        }
        let _ = i2c.write(addr, &line);
    }

    // 読み出し用バッファ
    let mut read_buf = [0u8; WIDTH];

    for page in 0..PAGE_COUNT {
        let set_page = [prefix_cmd, 0xB0 | (page as u8)];
        let _ = i2c.write(addr, &set_page);
        let set_col_low = [prefix_cmd, 0x00];
        let _ = i2c.write(addr, &set_col_low);
        let set_col_high = [prefix_cmd, 0x10];
        let _ = i2c.write(addr, &set_col_high);

        // GDDRAMリード
        // 最初の1バイトはダミーなので2回呼ぶ
        let mut dummy = [0u8; 1];
        let _ = i2c.write_read(addr, &[prefix_data], &mut dummy);
        if i2c.write_read(addr, &[prefix_data], &mut read_buf).is_err() {
            let _ = writeln!(serial, "[Fail] I2C read failed at page {}", page);
            return Err(ExplorerError::ExecutionFailed);
        }

        for (i, &val) in read_buf.iter().enumerate() {
            if val != test_pattern {
                let _ = writeln!(
                    serial,
                    "[Fail] VRAM mismatch page={} col={} got={:02X} expected={:02X}",
                    page, i, val, test_pattern
                );
                return Err(ExplorerError::ExecutionFailed);
            }
        }
    }

    let _ = writeln!(serial, "[OK] VRAM check passed, init confirmed!");

    Ok(sequence)
}
