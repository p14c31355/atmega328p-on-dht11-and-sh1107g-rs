#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::{Explorer, CmdNode, ExplorerError};

adapt_serial!(UnoWrapper);

// 最大コマンド長 + prefix
const BUF_CAP: usize = 3;

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

    // ---- Explorer 初期化 ----
    let explorer: Explorer<'static, 14, BUF_CAP> = Explorer {
        sequence: &EXPLORER_CMDS,
    };

    writeln!(serial, "[Info] Sending all commands to 0x3C...").ok();

    let addr: u8 = 0x3C;
    let prefix: u8 = 0x00;

    if let Err(e) = run_explorer_with_log(&explorer, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] Explorer execution failed: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Explorer execution complete").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Explorer の依存関係順送信 + リアルタイムログ
fn run_explorer_with_log<I2C, S, const N: usize, const BUF_CAP: usize>(
    explorer: &Explorer<'_, N, BUF_CAP>,
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
    // トポロジカルソートで依存順のインデックスを取得
    let sorted_indices = explorer.topo_sort()?; // ExplorerError::DependencyCycle 返し

    for &idx in sorted_indices.iter() {
        let node = &explorer.sequence[idx];

        writeln!(serial, "[Send] Node {} bytes={:02X?} deps={:?}", idx, node.bytes, node.deps).ok();

        // prefix + コマンドバイト
        let mut buf = [0u8; BUF_CAP];
        buf[0] = prefix;
        let len = 1 + node.bytes.len().min(BUF_CAP - 1);
        buf[1..len].copy_from_slice(&node.bytes[..len - 1]);

        if let Err(e) = i2c.write(addr, &buf[..len]) {
            writeln!(serial, "[Fail] Node {}: {:?}", idx, e).ok();
            // 途中失敗でも残りコマンドを続行
            continue;
        } else {
            writeln!(serial, "[OK] Node {} sent", idx).ok();
        }
    }

    Ok(())
}
