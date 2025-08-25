#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::i2c;
use dvcdbg::{
    adapt_serial,
    scanner::scan_i2c,
    explorer::{Explorer, CmdNode}
};
use embedded_io::Write;
use panic_halt as _;

adapt_serial!(UnoWrapper);

const DISPLAY_WIDTH: usize = 128;
const DISPLAY_HEIGHT: usize = 128;
const PAGE_HEIGHT: usize = 8;
const COLUMN_OFFSET: usize = 2;
const I2C_MAX_WRITE: usize = 32;

pub const SH1107G_INIT_CMDS: &[u8] = &[
    0xAE,       // Display OFF
    0xDC, 0x00, // Display start line = 0
    0x81, 0x2F, // Contrast
    0x20, 0x02, // Memory addressing mode: page
    0xA0,       // Segment remap normal
    0xC0,       // Common output scan direction normal
    0xA4,       // Entire display ON from RAM
    0xA6,       // Normal display
    0xA8, 0x7F, // Multiplex ratio 128
    0xD3, 0x60, // Display offset
    0xD5, 0x51, // Oscillator frequency
    0xD9, 0x22, // Pre-charge period
    0xDB, 0x35, // VCOM deselect level
    0xAD, 0x8A, // DC-DC control
    0xAF,       // Display ON
];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut delay = arduino_hal::Delay::new();

    let mut i2c = i2c::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    let serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut serial_wrapper = UnoWrapper(serial);
    writeln!(serial_wrapper, "[log] Start SH1107G auto-init test").unwrap();

    scan_i2c(&mut i2c, &mut serial_wrapper, dvcdbg::scanner::LogLevel::Quiet);

    let address = 0x3C;

    // --- SH1107G_INIT_CMDS を CmdNode 化する ---
    // 基本ルール:
    //   - コマンド自体は deps: &[]
    //   - 直後にパラメータがある場合、そのパラメータの deps に親コマンドを指定
    let nodes: &[CmdNode] = &[
        CmdNode { cmd: 0xAE, deps: &[] },              // Display OFF
        CmdNode { cmd: 0xDC, deps: &[] },          // Display start line
        CmdNode { cmd: 0x00, deps: &[] },          // param
        CmdNode { cmd: 0x81, deps: &[] },          // Contrast
        CmdNode { cmd: 0x2F, deps: &[] },          // param
        CmdNode { cmd: 0x20, deps: &[] },          // Mem mode
        CmdNode { cmd: 0x02, deps: &[] },          // param
        CmdNode { cmd: 0xA0, deps: &[] },          // Segment remap
        CmdNode { cmd: 0xC0, deps: &[] },          // COM scan dir
        CmdNode { cmd: 0xA4, deps: &[] },          // Entire display
        CmdNode { cmd: 0xA6, deps: &[] },          // Normal display
        CmdNode { cmd: 0xA8, deps: &[] },          // Multiplex
        CmdNode { cmd: 0x7F, deps: &[] },          // param
        CmdNode { cmd: 0xD3, deps: &[] },          // Offset
        CmdNode { cmd: 0x60, deps: &[] },          // param
        CmdNode { cmd: 0xD5, deps: &[] },          // Oscillator
        CmdNode { cmd: 0x51, deps: &[] },          // param
        CmdNode { cmd: 0xD9, deps: &[] },          // Pre-charge
        CmdNode { cmd: 0x22, deps: &[] },          // param
        CmdNode { cmd: 0xDB, deps: &[] },          // VCOM
        CmdNode { cmd: 0x35, deps: &[] },          // param
        CmdNode { cmd: 0xAD, deps: &[] },          // DC-DC
        CmdNode { cmd: 0x8A, deps: &[] },          // param
        CmdNode { cmd: 0xAF, deps: &[0xAE] },          // Display ON
    ];

    let explorer = Explorer { sequence: nodes };
    explorer.explore(&mut i2c, &mut serial_wrapper).ok();

    writeln!(serial_wrapper, "[oled] init sequence applied").unwrap();

    // --- 動作確認: 中央クロス表示 ---
    let mut page_buf: [u8; DISPLAY_WIDTH] = [0; DISPLAY_WIDTH];

    for page in 0..(DISPLAY_HEIGHT / PAGE_HEIGHT) {
        for x in 0..DISPLAY_WIDTH {
            page_buf[x] = if page == DISPLAY_HEIGHT / 2 / PAGE_HEIGHT { 0xFF } else { 0x00 };
        }
        i2c.write(address, &[0x00, 0xB0 + page as u8]).ok();
        i2c.write(address, &[0x00, 0x00 + COLUMN_OFFSET as u8]).ok();
        i2c.write(address, &[0x00, 0x10]).ok();

        for chunk in page_buf.chunks(I2C_MAX_WRITE - 1) {
            let mut data: heapless::Vec<u8, I2C_MAX_WRITE> = heapless::Vec::new();
            data.push(0x40).ok();
            data.extend_from_slice(chunk).ok();
            i2c.write(address, &data).ok();
        }
    }

    writeln!(serial_wrapper, "[oled] cross drawn").unwrap();

    loop { delay.delay_ms(1000u16); }
}
