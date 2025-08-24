#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{adapt_serial, scanner::{scan_i2c, scan_init_sequence}};
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    // I2C 初期化
    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    // UART 初期化
    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G order test").ok();

    // I2C デバイススキャン
    scan_i2c(&mut i2c, &mut serial_wrapper, dvcdbg::scanner::LogLevel::Quiet);

    // 並べ替え対象
    let cmds: [[u8; 2]; 3] = [
        [0x81, 0x2F], // Contrast
        [0xA1, 0x00], // Display start line（仮値）
        [0xD3, 0x60], // Display offset
    ];

    // 探索: 3! = 6 通り
    let perms: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for (i, order) in perms.iter().enumerate() {
        writeln!(serial_wrapper, "[test {}] start", i).ok();

        let mut seq: heapless::Vec<u8, 16> = heapless::Vec::new();
        seq.extend_from_slice(&[0xAE]).ok(); // Display OFF
        for &idx in order {
            seq.extend_from_slice(&cmds[idx]).ok();
        }
        seq.extend_from_slice(&[0xAF]).ok(); // Display ON

        scan_init_sequence(
            &mut i2c,
            &mut serial_wrapper,
            &seq,
            dvcdbg::scanner::LogLevel::Quiet,
        );

        delay.delay_ms(1000u16);
    }

    writeln!(serial_wrapper, "[log] all permutations done").ok();

    loop {
        delay.delay_ms(1000u16);
    }
}
