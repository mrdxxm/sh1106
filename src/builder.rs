//! Interface factory

use hal;
use hal::digital::OutputPin;

use super::displaysize::DisplaySize;
use super::interface::{I2cInterface, SpiInterface};
use super::SSD1306;

/// Communication interface factory
#[derive(Clone, Copy)]
pub struct Builder {
    display_size: DisplaySize,
}

impl Builder {
    /// Create new builder for default size of 128 x 64 pixels.
    pub fn new() -> Self {
        Self {
            display_size: DisplaySize::Display128x64,
        }
    }

    /// Create new builder for a specified size.
    pub fn with_size(&self, display_size: DisplaySize) -> Self {
        Self { display_size }
    }

    /// Create i2c communication interface
    pub fn connect_i2c<I2C>(&self, i2c: I2C) -> SSD1306<I2cInterface<I2C>>
    where
        I2C: hal::blocking::i2c::Write,
    {
        SSD1306::new(I2cInterface::new(i2c), self.display_size)
    }

    /// Create spi communication interface
    pub fn connect_spi<SPI, DC>(&self, spi: SPI, dc: DC) -> SSD1306<SpiInterface<SPI, DC>>
    where
        SPI: hal::blocking::spi::Transfer<u8> + hal::blocking::spi::Write<u8>,
        DC: OutputPin,
    {
        SSD1306::new(SpiInterface::new(spi, dc), self.display_size)
    }
}
