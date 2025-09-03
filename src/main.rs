#![no_std]
#![no_main]

use dvcdbg::prelude::*;
use panic_abort as _;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 115200));
    arduino_hal::delay_ms(1000);

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100_000,
    );

    const PREFIX: u8 = 0x00;

    // --- Explorer nodes define ---
    const INIT_SEQUENCE: [u8; 24] = [
    0xAE,       // Display OFF
    0xD5, 0x51, // Set Display Clock
    0xA8, 0x3F, // Set MUX Ratio
    0xD3, 0x60, // Set Display Offset
    0x40,       // Set Start Line
    0xA1,       // Segment remap
    0xA0,       // Scan direction
    0xC8,       // COM output scan
    0x81, 0x2F, // Contrast
    0xAD, 0x8B, // External VCC / VCOMH
    0xD9, 0x22, // Pre-charge
    0xDB, 0x35, // VCOMH
    0x8D, 0x14, // Charge pump
    0xA4,       // RAM display normal
    0xB0,       // Set page start (will loop 0~7)
    0xAF        // Display ON
];

let explorer_instance = nodes! {
    prefix = PREFIX,
    [
        // 基本初期化コマンド
        [0xAE],
        [0xD5, 0x51] @ [0],
        [0xA8, 0x3F] @ [1],
        [0xD3, 0x60] @ [2],
        [0x40] @ [3],
        [0xA1] @ [4],
        [0xA0] @ [5],
        [0xC8] @ [6],
        [0x81, 0x2F] @ [7],
        [0xAD, 0x8B] @ [8],
        [0xD9, 0x22] @ [9],
        [0xDB, 0x35] @ [10],
        [0x8D, 0x14] @ [11],
        [0xA4] @ [12],
        // ページごとの初期化
        [0xB0, 0x00, 0x00] @ [12], // Page 0, Column 0
        [0xB1, 0x00, 0x00] @ [12], // Page 1
        [0xB2, 0x00, 0x00] @ [12], // Page 2
        [0xB3, 0x00, 0x00] @ [12], // Page 3
        [0xB4, 0x00, 0x00] @ [12], // Page 4
        [0xB5, 0x00, 0x00] @ [12], // Page 5
        [0xB6, 0x00, 0x00] @ [12], // Page 6
        [0xB7, 0x00, 0x00] @ [12], // Page 7
        [0xAF] @ [0]               // Display ON 最後
    ]
};

    arduino_hal::delay_ms(1000);

    let _ = pruning_sort!(explorer_instance.0, &mut i2c, &mut serial, PREFIX, 23, 256, 22);

    loop {}
}
