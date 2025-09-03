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
    const INIT_SEQUENCE: [u8; 23] = [
    0xAE, 0xD5, 0x51, 0xA8, 0x3F, 0xD3, 0x60, 0x40,
    0x00, 0xA1, 0x00, 0xA0, 0xC8, 0xAD, 0x8B, 0xD9,
    0x22, 0xDB, 0x35, 0x8D, 0x14, 0xB0, 0xAF
];

    let explorer_instance = nodes! {
    prefix = PREFIX,
    [
        [0xAE],                   // Display OFF
        [0xD5, 0x51] @ [0],       // Clock divide
        [0xA8, 0x3F] @ [1],       // Multiplex ratio
        [0xD3, 0x60] @ [2],       // Display offset
        [0x40] @ [3],             // Start line
        [0xA1] @ [4],             // Segment remap
        [0xA0] @ [5],             // COM scan direction
        [0xC8] @ [6],             // COM output scan
        [0xAD, 0x8B] @ [7],       // DC-DC ON
        [0x81, 0x2F] @ [8],       // Contrast
        [0xD9, 0x22] @ [9],       // Pre-charge
        [0xDB, 0x35] @ [10],      // VCOMH
        [0x8D, 0x14] @ [11],      // Charge pump
        [0xA4] @ [12],            // RAM display normal
        // Page 0～7 設定＋カラム初期化
        [0xB0] @ [12],            // Page 0
        [0x00, 0x10] @ [13],      // Column low/high
        [0xB1] @ [12],            // Page 1
        [0x00, 0x10] @ [15],
        [0xB2] @ [12],            // Page 2
        [0x00, 0x10] @ [17],
        [0xB3] @ [12],            // Page 3
        [0x00, 0x10] @ [19],
        [0xB4] @ [12],            // Page 4
        [0x00, 0x10] @ [21],
        [0xB5] @ [12],            // Page 5
        [0x00, 0x10] @ [23],
        [0xB6] @ [12],            // Page 6
        [0x00, 0x10] @ [25],
        [0xB7] @ [12],            // Page 7
        [0x00, 0x10] @ [27],
        [0xAF] @ [27]             // Display ON after last page
    ]
};




    arduino_hal::delay_ms(1000);

    let _ = pruning_sort!(explorer_instance.0, &mut i2c, &mut serial, PREFIX, &INIT_SEQUENCE, 31, 23, 256, 30);

    loop {}
}
