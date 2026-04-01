use core::convert::Infallible;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorType as DigitalErrorType, OutputPin};
use embedded_hal::spi::{ErrorType as SpiErrorType, Operation, SpiDevice};
use lcd_ili9225_rs::{ColorOrder, Config, ILI9225, Orientation, SpiMode};

struct MockSpi;

impl SpiErrorType for MockSpi {
    type Error = Infallible;
}

impl SpiDevice for MockSpi {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                Operation::Read(buffer) => buffer.fill(0),
                Operation::Write(_buffer) => {}
                Operation::Transfer(read, write) => {
                    let len = read.len().min(write.len());
                    read[..len].copy_from_slice(&write[..len]);
                    if read.len() > len {
                        read[len..].fill(0);
                    }
                }
                Operation::TransferInPlace(_buffer) => {}
                Operation::DelayNs(_ns) => {}
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct MockPin(bool);

impl DigitalErrorType for MockPin {
    type Error = Infallible;
}

impl OutputPin for MockPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0 = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0 = true;
        Ok(())
    }
}

struct MockDelay;

impl DelayNs for MockDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

fn main() {
    let spi = MockSpi;
    let rs = MockPin::default();
    let rst = Some(MockPin::default());
    let mut delay = MockDelay;

    let mut config = Config::new(176, 220);
    config.spi_mode = SpiMode::WithDataCommandPin;
    config.color_order = ColorOrder::Rgb;
    config.orientation = Orientation::Portrait;
    config.display_enabled = true;

    let mut display = ILI9225::new_with_config(spi, rs, rst, config);
    display.init(&mut delay).unwrap();
    display.set_display_enabled(true).unwrap();
    display.set_orientation(&Orientation::Landscape).unwrap();
    display.set_pixel(0, 0, 0xF800).unwrap();
}
