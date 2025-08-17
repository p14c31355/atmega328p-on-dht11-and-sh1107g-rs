#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;
use dvcdbg::adapt_serial;

adapt_serial!(UsartAdapter, nb_write = write, flush);

#[arduino_hal::entry]
fn main() -> ! {
    // Peripherals とピンの取得
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // 標準シリアルを作成
    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    // マクロで生成されたアダプタにラップ
    let mut dbg_uart = UsartAdapter(serial);

    // 書き込みテスト
    writeln!(dbg_uart, "Hello from embedded-io bridge!").ok();
    dbg_uart.write_all(&[0x01, 0x02, 0x03]).ok();

    loop {}
}
