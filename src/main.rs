#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use embedded_io::Write;
use panic_abort as _; // panic-abort に変更済みの場合はこちらを使用

use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // シリアル初期化
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "\n[SH1107G Explorer Test]").ok();
    writeln!(serial, "").ok();

    // I2C 初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100000,
    );
    writeln!(serial, "[Info] I2Cの初期化が完了しました。").ok();

    // ---- 検証のため、init_seqとcmdsを空の配列で定義 ----
    // 元の配列はコメントアウトしてください
    let init_seq: [u8; 0] = [];
    let cmds: [CmdNode; 0] = [];
    let explorer = Explorer::<0> { sequence: &cmds };

    // ---- 探索実行 ----
    // Explorerのジェネリクスパラメータを0に修正
    let _ = run_explorer::<_, _, 0, 0>(
        &explorer,
        &mut i2c,
        &mut serial,
        &init_seq,
        0x3C,
        LogLevel::Verbose
    );

    loop {}
}