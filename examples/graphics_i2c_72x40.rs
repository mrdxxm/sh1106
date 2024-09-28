//! Draw a square, circle and triangle on the screen using the `embedded_graphics` crate.
//!
//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> PB7
//! (green)  SCL -> PB6
//! ```
//!
//! Run on a Blue Pill with `cargo run --example graphics_i2c`.

#![no_std]
#![no_main]

use cortex_m::asm::nop;
use cortex_m_rt::entry;
use defmt_rtt as _;
use embassy_stm32::time::Hertz;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle},
};
use panic_probe as _;
use sh1106::{prelude::*, I2CDisplayInterface, Sh1106};

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    let i2c = embassy_stm32::i2c::I2c::new_blocking(
        p.I2C1,
        p.PB6,
        p.PB7,
        Hertz::khz(400),
        Default::default(),
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Sh1106::new(interface, DisplaySize72x40, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let size = 10;
    let offset = Point::new(10, (42 / 2) - (size / 2) - 1);
    let spacing = size + 10;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    // screen outline
    // default display size is 128x64 if you don't pass a _DisplaySize_
    // enum to the _Builder_ struct
    Rectangle::with_corners(Point::new(0, 0), Point::new(71, 39))
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    // Triangle
    Triangle::new(
        Point::new(0, size),
        Point::new(size / 2, 0),
        Point::new(size, size),
    )
    .translate(offset)
    .into_styled(style)
    .draw(&mut display)
    .unwrap();

    // Move over to next position
    let offset = offset + Point::new(spacing, 0);

    // Draw a square
    Rectangle::new(Point::new(0, 0), Size::new_equal(size as u32))
        .translate(offset)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    // Move over a bit more
    let offset = offset + Point::new(spacing, 0);

    // Circle
    Circle::new(Point::zero(), size as u32)
        .translate(offset)
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    display.flush().unwrap();

    loop {
        nop()
    }
}
