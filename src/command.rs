//! Display commands.

// Shamefully taken from https://github.com/EdgewaterDevelopment/rust-sh1106

#[cfg(feature = "async")]
use display_interface::AsyncWriteOnlyDataCommand;
use display_interface::{DataFormat::U8, DisplayError, WriteOnlyDataCommand};

/// SH1106 Commands
#[maybe_async_cfg::maybe(sync(keep_self), async(feature = "async"))]
#[derive(Debug, Copy, Clone)]
pub enum Command {
    /// 81h Set contrast. Higher number is higher contrast. Default = 0x80
    Contrast(u8),
    /// A4h/A5h Turn entire display on. If set, all pixels will
    /// be set to on, if not, the value in memory will be used.
    AllOn(bool),
    /// A6h/A7h Invert display.
    Invert(bool),
    /// AEh/AFh Turn display on or off.
    DisplayOn(bool),
    /// 00h-0Fh Set the lower nibble of the column start address
    /// register for Page addressing mode, using the lower
    /// 4 bits given.
    /// This is only for page addressing mode
    LowerColStart(u8),
    /// 10h-1Fh Set the upper nibble of the column start address
    /// register for Page addressing mode, using the lower
    /// 4 bits given.
    /// This is only for page addressing mode
    UpperColStart(u8),
    /// Set the column start address register
    /// Combines LowerColStart and UpperColStart
    ColStart(u8),
    ///  B0h-B7h Set Page Address
    ///
    PageStart(Page),
    /// 40h-7Fh Set display start line from 0-63
    StartLine(u8),
    /// A0h/A1h Reverse columns from 127-0
    SegmentRemap(bool),
    /// A8h Set multiplex ratio from 1-64. Default is 64
    Multiplex(u8),
    /// C0h/C8h If true, scan from COM[n-1] to COM0 (where N is mux ratio)
    /// Default is false
    ReverseComDir(bool),
    /// D3h Set vertical shift from 0-63
    DisplayOffset(u8),
    /// Dah Setup COM hardware configuration
    /// Indicates sequential (false) or alternative (true)
    /// pin configuration.
    ComPinConfig(bool),
    /// D5h Set up display clock.
    /// First value is oscillator frequency, increasing with higher value. POR value is 5
    /// Second value is divide ratio - 1
    DisplayClockDiv(u8, u8),
    /// D9h Set up phase 1 and 2 of precharge period. Each value must be in the range 1 - 15.
    /// Default is 2
    PreChargePeriod(u8, u8),
    /// Set Vcomh Deselect level
    VcomhDeselect(VcomhLevel),
    /// NOOP
    Noop,
    /// 8Ah/8Bh Enable charge pump
    ChargePump(bool),
    /// 30h - 33h Set Pump voltage value
    SetPumpVoltage(PumpVoltage),
    /// E0h Start Read-Modify-Write
    /// A pair of Read-Modify-Write and End commands must always be used. Once read-modify-write is issued,
    /// column address is not incremental by read display data command but incremental by write display data command only.
    /// It continues until End command is issued. When the End is issued, column address returns to the address
    /// when read-modify-write is issued. This can reduce the microprocessor load when data of a specific display area
    /// is repeatedly changed during cursor blinking or others.
    ReadModifyWriteStart,
    /// EEh End Read-Modify-Write
    /// Cancels Read-Modify-Write mode and returns column address to the original address (when Read-Modify-Write is issued.)
    ReadModifyWriteEnd,
}

#[maybe_async_cfg::maybe(
    sync(keep_self),
    async(
        feature = "async",
        idents(WriteOnlyDataCommand(async = "AsyncWriteOnlyDataCommand"))
    )
)]
impl Command {
    /// Send command to SH1106
    pub async fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        match self {
            Command::Contrast(val) => Self::send_commands(iface, &[0x81, val]).await,
            Command::AllOn(on) => Self::send_commands(iface, &[0xA4 | (on as u8)]).await,
            Command::Invert(inv) => Self::send_commands(iface, &[0xA6 | (inv as u8)]).await,
            Command::DisplayOn(on) => Self::send_commands(iface, &[0xAE | (on as u8)]).await,
            Command::LowerColStart(addr) => Self::send_commands(iface, &[0xF & addr]).await,
            Command::UpperColStart(addr) => {
                Self::send_commands(iface, &[0x10 | (0xF & addr)]).await
            }
            Command::ColStart(addr) => {
                Self::send_commands(iface, &[0xF & addr, 0x10 | (0xF & (addr >> 4))]).await
            }
            Command::PageStart(page) => Self::send_commands(iface, &[0xB0 | (page as u8)]).await,
            Command::StartLine(line) => Self::send_commands(iface, &[0x40 | (0x3F & line)]).await,
            Command::SegmentRemap(remap) => {
                Self::send_commands(iface, &[0xA0 | (remap as u8)]).await
            }
            Command::Multiplex(ratio) => Self::send_commands(iface, &[0xA8, ratio]).await,
            Command::ReverseComDir(rev) => {
                Self::send_commands(iface, &[0xC0 | ((rev as u8) << 3)]).await
            }
            Command::DisplayOffset(offset) => Self::send_commands(iface, &[0xD3, offset]).await,
            Command::ComPinConfig(alt) => {
                Self::send_commands(iface, &[0xDA, 0x2 | ((alt as u8) << 4)]).await
            }
            Command::DisplayClockDiv(fosc, div) => {
                Self::send_commands(iface, &[0xD5, ((0xF & fosc) << 4) | (0xF & div)]).await
            }
            Command::PreChargePeriod(phase1, phase2) => {
                Self::send_commands(iface, &[0xD9, ((0xF & phase2) << 4) | (0xF & phase1)]).await
            }
            Command::VcomhDeselect(level) => {
                Self::send_commands(iface, &[0xDB, (level as u8) << 4]).await
            }
            Command::Noop => Self::send_commands(iface, &[0xE3]).await,
            Command::ChargePump(en) => Self::send_commands(iface, &[0xAD, 0x8A | (en as u8)]).await,
            Command::SetPumpVoltage(voltage) => {
                Self::send_commands(iface, &[0x30 | (voltage as u8)]).await
            }
            Command::ReadModifyWriteStart => Self::send_commands(iface, &[0xE0]).await,
            Command::ReadModifyWriteEnd => Self::send_commands(iface, &[0xEE]).await,
        }
    }

    async fn send_commands<DI>(iface: &mut DI, data: &[u8]) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        iface.send_commands(U8(data)).await
    }
}

/// Display page
#[derive(Debug, Clone, Copy)]
pub enum Page {
    /// Page 0
    Page0 = 0b0000,
    /// Page 1
    Page1 = 0b0001,
    /// Page 2
    Page2 = 0b0010,
    /// Page 3
    Page3 = 0b0011,
    /// Page 4
    Page4 = 0b0100,
    /// Page 5
    Page5 = 0b0101,
    /// Page 6
    Page6 = 0b0110,
    /// Page 7
    Page7 = 0b0111,
}

impl From<u8> for Page {
    fn from(val: u8) -> Page {
        match val / 8 {
            0 => Page::Page0,
            1 => Page::Page1,
            2 => Page::Page2,
            3 => Page::Page3,
            4 => Page::Page4,
            5 => Page::Page5,
            6 => Page::Page6,
            7 => Page::Page7,
            _ => unreachable!(),
        }
    }
}

/// VCOM voltage levels based on the formula:
/// VCOM = (0.430 + A\[7:0\] * 0.006415) * VREF
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum VcomhLevel {
    /// 0.430 * VREF
    V0430 = 0x00,
    /// 0.436415 * VREF
    V0436 = 0x01,
    /// 0.44283 * VREF
    V0442 = 0x02,
    /// 0.449245 * VREF
    V0449 = 0x03,
    /// 0.45566 * VREF
    V0455 = 0x04,
    /// 0.462075 * VREF
    V0462 = 0x05,
    /// 0.46849 * VREF
    V0468 = 0x06,
    /// 0.474905 * VREF
    V0474 = 0x07,
    /// 0.48132 * VREF
    V0481 = 0x08,
    /// 0.487735 * VREF
    V0487 = 0x09,
    /// 0.49415 * VREF
    V0494 = 0x0A,
    /// 0.500565 * VREF
    V0500 = 0x0B,
    /// 0.50698 * VREF
    V0506 = 0x0C,
    /// 0.513395 * VREF
    V0513 = 0x0D,
    /// 0.51981 * VREF
    V0519 = 0x0E,
    /// 0.526225 * VREF
    V0526 = 0x0F,
    /// 0.53264 * VREF
    V0532 = 0x10,
    /// 0.539055 * VREF
    V0539 = 0x11,
    /// 0.54547 * VREF
    V0545 = 0x12,
    /// 0.551885 * VREF
    V0551 = 0x13,
    /// 0.5583 * VREF
    V0558 = 0x14,
    /// 0.564715 * VREF
    V0564 = 0x15,
    /// 0.57113 * VREF
    V0571 = 0x16,
    /// 0.577545 * VREF
    V0577 = 0x17,
    /// 0.58396 * VREF
    V0583 = 0x18,
    /// 0.590375 * VREF
    V0590 = 0x19,
    /// 0.59679 * VREF
    V0596 = 0x1A,
    /// 0.603205 * VREF
    V0603 = 0x1B,
    /// 0.60962 * VREF
    V0609 = 0x1C,
    /// 0.616035 * VREF
    V0616 = 0x1D,
    /// 0.62245 * VREF
    V0622 = 0x1E,
    /// 0.628865 * VREF
    V0628 = 0x1F,
    /// 0.63528 * VREF
    V0635 = 0x20,
    /// 0.641695 * VREF
    V0641 = 0x21,
    /// 0.64811 * VREF
    V0648 = 0x22,
    /// 0.654525 * VREF
    V0654 = 0x23,
    /// 0.66094 * VREF
    V0660 = 0x24,
    /// 0.667355 * VREF
    V0667 = 0x25,
    /// 0.67377 * VREF
    V0673 = 0x26,
    /// 0.680185 * VREF
    V0680 = 0x27,
    /// 0.6866 * VREF
    V0686 = 0x28,
    /// 0.693015 * VREF
    V0693 = 0x29,
    /// 0.69943 * VREF
    V0699 = 0x2A,
    /// 0.705845 * VREF
    V0705 = 0x2B,
    /// 0.71226 * VREF
    V0712 = 0x2C,
    /// 0.718675 * VREF
    V0718 = 0x2D,
    /// 0.72509 * VREF
    V0725 = 0x2E,
    /// 0.731505 * VREF
    V0731 = 0x2F,
    /// 0.73792 * VREF
    V0737 = 0x30,
    /// 0.744335 * VREF
    V0744 = 0x31,
    /// 0.75075 * VREF
    V0750 = 0x32,
    /// 0.757165 * VREF
    V0757 = 0x33,
    /// 0.76358 * VREF
    V0763 = 0x34,
    /// 0.769995 * VREF
    #[default]
    V0769 = 0x35,
    /// 0.77641 * VREF
    V0776 = 0x36,
    /// 0.782825 * VREF
    V0782 = 0x37,
    /// 0.78924 * VREF
    V0789 = 0x38,
    /// 0.795655 * VREF
    V0795 = 0x39,
    /// 0.80207 * VREF
    V0802 = 0x3A,
    /// 0.808485 * VREF
    V0808 = 0x3B,
    /// 0.8149 * VREF
    V0814 = 0x3C,
    /// 0.821315 * VREF
    V0821 = 0x3D,
    /// 0.82773 * VREF
    V0827 = 0x3E,
    /// 0.834145 * VREF
    V0834 = 0x3F,
    /// 1 * VREF
    V1000 = 0x40,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
/// Pump output voltage (VPP)
pub enum PumpVoltage {
    /// 6.4V
    V64 = 0,
    /// 7.4V
    V74 = 1,
    /// 8V
    #[default]
    V80 = 2,
    /// 9V
    V90 = 3,
}
