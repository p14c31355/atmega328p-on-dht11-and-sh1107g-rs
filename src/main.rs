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
    writeln!(serial, "[SH1107G Full Init Test]").ok();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // ---- コマンド配列 ----
    static EXPLORER_CMDS: [CmdNode; 14] = [
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
        CmdNode { bytes: &[0xA6], deps: &[11] },
        CmdNode { bytes: &[0xAF], deps: &[12] },
    ];

    writeln!(serial, "[Info] Starting exploration...").ok();

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    if let Err(e) = explore_commands(&EXPLORER_CMDS, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] Exploration failed: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Exploration complete").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// バックトラック探索関数
fn explore_commands<I2C, S>(
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
    let mut sent = [false; 14]; // 送信済みフラグ
    let mut progress = true;

    while progress {
        progress = false;

        for (idx, cmd) in cmds.iter().enumerate() {
            if sent[idx] { continue; }

            // 依存関係チェック
            if !cmd.deps.iter().all(|&d| sent[d]) { continue; }

            writeln!(serial, "[Try] Node {} bytes={:02X?} deps={:?}", idx, cmd.bytes, cmd.deps).ok();

            // バッファ作成
            let buf_len = 1 + cmd.bytes.len();
            if buf_len > BUF_CAP {
                writeln!(serial, "[Fail] Node {}: buffer overflow", idx).ok();
                return Err(ExplorerError::BufferOverflow);
            }
            let mut buf = [0u8; BUF_CAP];
            buf[0] = prefix;
            buf[1..buf_len].copy_from_slice(cmd.bytes);

            // I2C 書き込み
            match i2c.write(addr, &buf[..buf_len]) {
                Ok(_) => {
                    writeln!(serial, "[OK] Node {} sent", idx).ok();
                    sent[idx] = true;
                    progress = true;
                }
                Err(e) => {
                    writeln!(serial, "[Fail] Node {}: {:?}", idx, e).ok();
                    // 失敗しても探索継続
                    continue;
                }
            }
        }
    }

    // 全コマンド送信済みか確認
    if sent.iter().all(|&b| b) {
        Ok(())
    } else {
        Err(ExplorerError::ExecutionFailed)
    }
}
