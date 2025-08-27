#![no_std]
#![no_main]

use core::fmt::Write;
use panic_abort as _;
use dvcdbg::prelude::*;
use dvcdbg::explorer::ExplorerError; // ExplorerError をインポート

adapt_serial!(UnoWrapper);

const BUF_CAP: usize = 10; // 最大2バイト + prefix

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

    let addr: u8 = 0x3C; // `addr` をスコープ内で定義
    let prefix: u8 = 0x00; // `prefix` をスコープ内で定義 (コマンドプレフィックスとして0x00を仮定)

    // `sorted_cmd_bytes_buf` と `sorted_cmd_lengths` が `main` 関数内で定義されていないため、
    // `run_explorer_with_log` 関数を呼び出すように変更します。
    // `main` 関数内のこのブロックは、`run_explorer_with_log` の呼び出しに置き換えられます。
    if let Err(e) = run_explorer_with_log(&explorer, &mut i2c, &mut serial, addr, prefix) {
        writeln!(serial, "[Fail] Explorer execution failed: {:?}", e).ok();
    } else {
        writeln!(serial, "[OK] Explorer execution complete").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}

/// Explorer の順序探索 + リアルタイムログ出力
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
    let (sorted_cmd_bytes_buf, sorted_cmd_lengths) = explorer
        .get_one_topological_sort_buf(&mut dvcdbg::logger::SerialLogger::new(serial, dvcdbg::logger::LogLevel::Verbose))
        .map_err(|_| ExplorerError::DependencyCycle)?; // ExplorerError::TopoSortFailure は存在しないため、DependencyCycle に変更

    // get_one_topological_sort_buf はソートされたコマンドバイトの配列と、各コマンドの長さの配列を返す
    // 元のコードの q[i] は、このメソッドの内部で使われているキューであり、外部には公開されていない
    // そのため、ここではソートされたコマンドバイトと長さを直接利用する
    for i in 0..explorer.sequence.len() {
        let cmd_bytes = &sorted_cmd_bytes_buf[i][..sorted_cmd_lengths[i]];
        // ここで元の CmdNode の情報（depsなど）が必要な場合、get_one_topological_sort_buf の戻り値に
        // 元の CmdNode のインデックスを含めるように変更する必要がある。
        // しかし、現在のタスクはコンパイルエラーの修正なので、ここでは直接コマンドバイトを使用する。
        // 依存関係の表示は、元の CmdNode のインデックスが不明なため、一時的に省略する。

        writeln!(
            serial,
            "[Send] Command {} bytes={:02X?}",
            i, cmd_bytes
        )
        .ok();

        // prefix とコマンドバイトを結合してI2C書き込みを行う
        let mut buf: [u8; BUF_CAP] = [0; BUF_CAP];
        buf[0] = prefix;
        let mut current_len = 1;
        for &byte in cmd_bytes.iter() {
            if current_len < BUF_CAP {
                buf[current_len] = byte;
                current_len += 1;
            } else {
                writeln!(serial, "[Fail] Command {}: Buffer overflow", i).ok();
                return Err(ExplorerError::BufferOverflow);
            }
        }

        if let Err(e) = i2c.write(addr, &buf[..current_len]) {
            writeln!(serial, "[Fail] Command {}: {:?}", i, e).ok();
            return Err(ExplorerError::ExecutionFailed);
        } else {
            writeln!(serial, "[OK] Command {} sent", i).ok();
        }
    }
    Ok(())
}
