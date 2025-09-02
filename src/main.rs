#![no_std]
#![no_main]

use core::fmt::Write;
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


    let _ = scan_i2c(&mut i2c, &mut serial, PREFIX);
    arduino_hal::delay_ms(1000);
    let _ = scan_init_sequence(&mut i2c, &mut serial, PREFIX, &INIT_SEQUENCE);
    arduino_hal::delay_ms(1000);

    let explorer_instance = nodes! {
        prefix = PREFIX,
        [
            [0xAE],
            [0xD5, 0x51] @ [0],
            [0xA8, 0x3F] @ [1],
            [0xD3, 0x60] @ [2],
            [0x40] @ [3],
            [0xA1] @ [4],
            [0xA0] @ [5],
            [0xC8] @ [6],
            [0xAD, 0x8B] @ [7],
            [0xD9, 0x22] @ [8],
            [0xDB, 0x35] @ [9],
            [0x8D, 0x14] @ [10],
            [0x20, 0x00] @ [11],
            [0xA6] @ [12],
            [0xAF] @ [13],
            [0xB0] @ [14]
        ]
    };
    arduino_hal::delay_ms(1000);


    const NODES_COUNT: usize = 16;
    const CMD_BUFFER_SIZE: usize = 47;

    const INIT_SEQUENCE_LEN: usize = INIT_SEQUENCE.len();

    let (explorer, _executor) = explorer_instance;
    arduino_hal::delay_ms(1000);

    let res = pruning_sort!(explorer, &mut i2c, &mut serial, PREFIX, &INIT_SEQUENCE, NODES_COUNT, INIT_SEQUENCE_LEN, CMD_BUFFER_SIZE, 15);

    
    match res {
        Ok(()) => Write::write_str(&mut serial, "[I] Explorer OK.").ok(),
        Err(e) => writeln!(&mut serial, "[E] Explorer failed: {e}").ok(),
    };

    core::fmt::Write::write_str(&mut serial, "Enter main loop.").ok();
    loop {
        arduino_hal::delay_ms(1000);
    }
}
