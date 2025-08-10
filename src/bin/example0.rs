#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::default_serial;
use panic_halt as _;
use dvcdbg::logger::SerialLogger;
use dvcdbg::log;
use core::fmt::Write;
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;
use embedded_hal::blocking::i2c::Write as I2cWrite; // embedded_hal::blocking::i2c::Write に変更
use dvcdbg::scanner::scan_i2c;
use heapless::Vec;

// SH1107G の初期化コマンド (データシートから抜粋)
const SH1107G_INIT_CMDS: &[u8] = &[
    0xAE, // Display OFF
    0xDC, 0x00, // Display start line = 0
    0x81, 0x2F, // Contrast
    0x20, // Memory addressing mode: page
    0xA0, // Segment remap normal
    0xC0, // Common output scan direction normal
    0xA4, // Entire display ON from RAM
    0xA6, // Normal display
    0xA8, 0x7F, // Multiplex ratio 128
    0xD3, 0x60, // Display offset
    0xD5, 0x51, // Oscillator frequency
    0xD9, 0x22, // Pre-charge period
    0xDB, 0x35, // VCOM deselect level
    0xAD, 0x8A, // DC-DC control
    0xAF,       // Display ON
];

// arduino_hal::DefaultSerial を core::fmt::Write に適合させるラッパー
struct SerialWriter<'a, W: EmbeddedHalSerialWrite<u8>> {
    writer: &'a mut W,
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> SerialWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W: EmbeddedHalSerialWrite<u8>> Write for SerialWriter<'a, W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            nb::block!(self.writer.write(byte)).map_err(|_| core::fmt::Error)?;
        }
        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50000,
    );

    // I2Cバスのスキャンを実行
    scan_i2c(&mut i2c, &mut logger);

    let mut found_sh1107g_addr: Option<u8> = None;
    for addr in 0x03..=0x77 {
        if I2cWrite::write(&mut i2c, addr, &[]).is_ok() {
            log!(&mut logger, "Found device at 0x{:02X}", addr);
            if addr == 0x3C || addr == 0x3D {
                found_sh1107g_addr = Some(addr);
                break; // SH1107Gが見つかったらスキャンを終了
            }
        }
    }

    if let Some(addr) = found_sh1107g_addr {
        log!(&mut logger, "SH1107G 初期化開始 (アドレス: 0x{:02X})", addr);

        // SH1107G 初期化コマンドを送信
        // コントロールバイト 0x00 はコマンド続き送信の意味
        let mut payload = Vec::<u8, 64>::new();
        payload.push(0x00).unwrap();
        payload.extend_from_slice(SH1107G_INIT_CMDS).unwrap();

        if let Err(_e) = I2cWrite::write(&mut i2c, addr, &payload) {
            log!(&mut logger, "Init failed: {:?}", e);
        } else {
            log!(&mut logger, "Init OK");

            // 画面を真っ白に塗りつぶす (手動で実装)
            // SH1107G はページアドレス指定とカラムアドレス指定が必要
            // 128x128ディスプレイの場合、16ページ (0-15)
            // 各ページは8行、128カラム
            for page in 0..16 {
                // ページアドレスセット (0xB0 + page)
                I2cWrite::write(&mut i2c, addr, &[0x00, 0xB0 + page as u8]).unwrap();
                // カラムアドレスセット (0x00 + lower_nibble, 0x10 + upper_nibble)
                I2cWrite::write(&mut i2c, addr, &[0x00, 0x10]).unwrap();

                // ページデータを送信 (128バイト)
                // コントロールバイト 0x40 はデータ送信の意味
                let mut page_data = Vec::<u8, 129>::new();
                page_data.push(0x40).unwrap(); // データ送信コントロールバイト
                for _ in 0..128 {
                    page_data.push(0xFF).unwrap(); // 全て白 (0xFF)
                }
                I2cWrite::write(&mut i2c, addr, &page_data).unwrap();
            }
            log!(&mut logger, "Display filled with white.");
        }
    } else {
        log!(&mut logger, "SH1107G ディスプレイが見つかりませんでした。");
    }

    loop {}
}
