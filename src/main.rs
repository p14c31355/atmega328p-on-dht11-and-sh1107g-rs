#![no_std]
#![no_main]

use embedded_io::Write;
use panic_abort as _;

use dvcdbg::prelude::*;

adapt_serial!(UnoWrapper);

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Serial init
    let mut serial = UnoWrapper(arduino_hal::default_serial!(dp, pins, 57600));
    arduino_hal::delay_ms(1000);
    writeln!(serial, "[Test] SH1107G minimal Explorer").ok();

    // I2C init
    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(), // SDA
        pins.a5.into_pull_up_input(), // SCL
        100_000,
    );
    writeln!(serial, "[Info] I2C initialized").ok();

    // Minimal single command test
    static TEST_CMD: [CmdNode; 1] = [
        CmdNode { bytes: &[0xAE], deps: &[] }, // Display off
    ];
    let explorer = Explorer::<1> { sequence: &TEST_CMD };

    // Minimal executor
    struct SingleExecutor;
    impl<I2C: dvcdbg::compat::I2cCompat> CmdExecutor<I2C> for SingleExecutor {
        fn exec(&mut self, i2c: &mut I2C, addr: u8, cmd: &[u8]) -> Result<(), ExecutorError> {
            let buf = [0x00, cmd[0]]; // prefix 0x00 for command
            i2c.write(addr, &buf).map_err(|e| ExecutorError::I2cError(e.to_compat(Some(addr))))
        }
    }

    let mut executor = SingleExecutor;

    // NullLogger: just to satisfy Explorer interface
    let _logger = NullLogger;

    // Iterate permutations and try writing
    if let Ok(iter) = explorer.permutations() {
        writeln!(serial, "[Info] Starting minimal permutation test").ok();
        for perm in iter {
            for addr in 0x3C..=0x3C { // test only the known device
                if let Err(e) = perm.iter().try_for_each(|cmd| executor.exec(&mut i2c, addr, cmd)) {
                    writeln!(serial, "[error] Execution failed at addr {:#X}: {:?}", addr, e).ok();
                } else {
                    writeln!(serial, "[OK] Command sent to addr {:#X}", addr).ok();
                }
            }
        }
    } else {
        writeln!(serial, "[error] Explorer failed to generate permutations").ok();
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}
