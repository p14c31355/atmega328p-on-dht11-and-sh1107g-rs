#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use dvcdbg::logger::SerialLogger;
use dvcdbg::log;
use embedded_hal::serial::Write as EmbeddedHalSerialWrite;
use core::fmt::Write;
use dvcdbg::logger::Logger;
use panic_halt as _;

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

fn send_cmd<I2C, L>(i2c: &mut I2C, addr: u8, cmd: u8, logger: &mut L)
    where
        I2C: embedded_hal::blocking::i2c::Write,
        L: Logger,
    {
        let buf = [0x00, cmd];
        if i2c.write(addr, &buf).is_ok() {
            log!(logger, "Sent cmd 0x{:02X} to 0x{:02X}", cmd, addr);
        } else {
            log!(logger, "Failed cmd 0x{:02X} to 0x{:02X}", cmd, addr);
        }
    }


    fn init_sh1107(i2c: &mut arduino_hal::I2c, addr: u8) {
        let cmds: [u8; 23] = [
            0xAE,       // Display OFF
            0xDC, 0x00, // Display start line
            0x81, 0x2F, // Contrast
            0x20,       // Page addressing mode
            0xA0,       // Segment remap normal
            0xC0,       // COM scan dir normal
            0xA4,       // Display from RAM
            0xA6,       // Normal display
            0xA8, 0x7F, // Multiplex ratio
            0xD3, 0x60, // Display offset
            0xD5, 0x51, // Oscillator freq
            0xD9, 0x22, // Pre-charge
            0xDB, 0x35, // VCOM level
            0xAD, 0x8A, // DC-DC control
            0xAF,       // Display ON
        ];

        let mut i = 0;
        while i < cmds.len() {
            send_cmd(i2c, addr, cmds[i], &mut logger);
            i += 1;
        }
    }

#[arduino_hal::entry]
fn main() -> ! {
    // シリアルとロガー初期化
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut serial_writer = SerialWriter::new(&mut serial);
    let mut logger = SerialLogger::new(&mut serial_writer);

    // I2C初期化
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000, // 100kHz
    );

    log!(&mut logger, "I2Cスキャン開始");

    init_sh1107(&mut i2c, addr);

    // スキャン
    for addr in 0x03..=0x77 {
        if i2c.write(addr, &[]).is_ok() {
            log!(&mut logger, "Found device at 0x{:02X}", addr);

            // SH1107G検出時
            if addr == 0x3C || addr == 0x3D {
                log!(&mut logger, "SH1107G 初期化開始");
                init_sh1107(&mut i2c, addr);
                log!(&mut logger, "Init OK");
            }
        }
    }

    loop {}
}
