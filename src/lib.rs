#![no_std]
//! This crate provides a ILI9225 driver to connect to TFT displays.
//  width
// --------
// |      |
// |      |
// |      | height
// |      |
// |      |
// --------

pub mod instruction;
use crate::instruction::Instruction;
use core::convert::Infallible;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi;

const ENTRY_MODE_BGR: u16 = 1 << 12;
const ENTRY_MODE_ID1: u16 = 1 << 5;
const ENTRY_MODE_ID0: u16 = 1 << 4;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub spi_mode: SpiMode,
    pub color_order: ColorOrder,
    pub orientation: Orientation,
    pub inverted: bool,
    pub display_enabled: bool,
    pub width: u32,
    pub height: u32,
}

impl Config {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            spi_mode: SpiMode::WithDataCommandPin,
            color_order: ColorOrder::Rgb,
            orientation: Orientation::Portrait,
            inverted: false,
            display_enabled: true,
            width,
            height,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(176, 220)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOrder {
    Rgb,
    Bgr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiMode {
    WithDataCommandPin,
    Native { id: bool },
}

pub struct NoPin;

impl embedded_hal::digital::ErrorType for NoPin {
    type Error = Infallible;
}

impl OutputPin for NoPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error<SpiError, RsError, RstError> {
    Spi(SpiError),
    DataCommandPin(RsError),
    ResetPin(RstError),
    MissingDataCommandPin,
    NativeDataChunkTooLarge,
}

pub type Ili9225Error<SPI, RS, RST> = Error<
    <SPI as spi::ErrorType>::Error,
    <RS as embedded_hal::digital::ErrorType>::Error,
    <RST as embedded_hal::digital::ErrorType>::Error,
>;

pub struct ILI9225<SPI, RS, RST>
where
    SPI: spi::SpiDevice,
    RS: OutputPin,
    RST: OutputPin,
{
    /// SPI
    spi: SPI,
    /// Data/command pin.
    rs: Option<RS>,
    /// Reset pin.
    rst: Option<RST>,
    spi_mode: SpiMode,
    color_order: ColorOrder,
    /// Whether the colours are inverted (true) or not (false)
    inverted: bool,
    orientation: Orientation,
    display_enabled: bool,
    /// Global image offset
    dx: u16,
    dy: u16,
    width: u32,
    height: u32,
}
/// Display orientation.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum Orientation {
    Portrait = ENTRY_MODE_ID1 | ENTRY_MODE_ID0,
    Landscape = ENTRY_MODE_ID1 | (1 << 3),
    PortraitSwapped = 0,
    LandscapeSwapped = ENTRY_MODE_ID0 | (1 << 3),
}

impl<SPI, RS, RST> ILI9225<SPI, RS, RST>
where
    SPI: spi::SpiDevice,
    RS: OutputPin,
    RST: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    pub fn new(
        spi: SPI,
        rs: RS,
        rst: Option<RST>,
        rgb: bool,
        inverted: bool,
        width: u32,
        height: u32,
    ) -> Self {
        let mut config = Config::new(width, height);
        config.color_order = if rgb {
            ColorOrder::Rgb
        } else {
            ColorOrder::Bgr
        };
        config.inverted = inverted;
        Self::new_with_config(spi, rs, rst, config)
    }
    pub fn new_with_config(spi: SPI, rs: RS, rst: Option<RST>, config: Config) -> Self {
        ILI9225 {
            spi,
            rs: Some(rs),
            rst,
            spi_mode: config.spi_mode,
            color_order: config.color_order,
            inverted: config.inverted,
            orientation: config.orientation,
            display_enabled: config.display_enabled,
            dx: 0,
            dy: 0,
            width: config.width,
            height: config.height,
        }
    }
    pub fn new_native_spi(spi: SPI, rst: Option<RST>, config: Config) -> ILI9225<SPI, NoPin, RST> {
        ILI9225 {
            spi,
            rs: None,
            rst,
            spi_mode: match config.spi_mode {
                SpiMode::Native { id } => SpiMode::Native { id },
                SpiMode::WithDataCommandPin => SpiMode::Native { id: false },
            },
            color_order: config.color_order,
            inverted: config.inverted,
            orientation: config.orientation,
            display_enabled: config.display_enabled,
            dx: 0,
            dy: 0,
            width: config.width,
            height: config.height,
        }
    }
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), Ili9225Error<SPI, RS, RST>>
    where
        DELAY: DelayNs,
    {
        self.hard_reset(delay)?;
        // Power-on sequence
        //
        self.write_command(Instruction::PowerCtrl1, &[0x0000])?; //Set SAP,DSTB,STB
        self.write_command(Instruction::PowerCtrl2, &[0x0000])?; // Set APON,PON,AON,VCI1EN,VC
        self.write_command(Instruction::PowerCtrl3, &[0x0000])?; // Set BT,DC1,DC2,DC3
        self.write_command(Instruction::PowerCtrl4, &[0x0000])?; // Set GVDD
        self.write_command(Instruction::PowerCtrl5, &[0x0000])?; // Set VCOMH/VCOML voltage
        delay.delay_ms(50);
        // Power-on sequence
        self.write_command(Instruction::PowerCtrl2, &[0x00, 0x18])?; // Set APON,PON,AON,VCI1EN,VC
        self.write_command(Instruction::PowerCtrl3, &[0x61, 0x21])?; // Set BT,DC1,DC2,DC3
        self.write_command(Instruction::PowerCtrl4, &[0x00, 0x6F])?; // Set GVDD  /*007F 0088 */
        self.write_command(Instruction::PowerCtrl5, &[0x49, 0x5F])?; // Set VCOMH/VCOML voltage
        self.write_command(Instruction::PowerCtrl1, &[0x08, 0x00])?; // Set SAP,DSTB,STB
        delay.delay_ms(50);
        self.write_command(Instruction::PowerCtrl2, &[0x10, 0x3B])?; // Set APON,PON,AON,VCI1EN,VC
        delay.delay_ms(50);
        self.write_command(Instruction::DriverOutputCtrl, &[0x01, 0x1C])?; // set the display line number and display direction
        self.write_command(Instruction::LcdAcDrivingCtrl, &[0x01, 0x00])?; // set 1 line inversion
        self.write_register(
            Instruction::EntryMode,
            Self::entry_mode_value(self.orientation, self.color_order),
        )?;
        self.write_command(Instruction::DispCtrl1, &[0x00, 0x00])?; // Display off
        self.write_command(Instruction::DispCtrl2, &[0x08, 0x08])?; // Set the back porch and front porch
        self.write_command(Instruction::FrameCycleCtrl, &[0x11, 0x00])?; // Set the clocks number per line
        self.write_command(Instruction::InterfaceCtrl, &[0x00, 0x00])?; // CPU interface
        self.write_command(Instruction::OscCtrl, &[0x0D, 0x01])?; // Set Osc
        self.write_command(Instruction::VciRecycling, &[0x00, 0x20])?; // Set VCI recycling
        self.write_command(Instruction::RamAddrSet1, &[0x00, 0x00])?; // RAM Address
        self.write_command(Instruction::RamAddRSet2, &[0x00, 0x00])?; // RAM Address
        delay.delay_ms(20);
        //-------------- Set GRAM area -----------------//
        self.write_command(Instruction::GateScanCtrl, &[0x00, 0x00])?;
        self.write_command(Instruction::VerticalScrollCtrl1, &[0x00, 0xDB])?;
        self.write_command(Instruction::VerticalScrollCtrl2, &[0x00, 0x00])?;
        self.write_command(Instruction::VerticalScrollCtrl3, &[0x00, 0x00])?; //0x0000
        self.write_command(Instruction::PartialDrivingPos1, &[0x00, 0xDB])?; //0x00DB
        self.write_command(Instruction::PartialDrivingPos2, &[0x00, 0x00])?; //0x0000
        self.write_command(Instruction::HorizontalWindowAddr1, &[0x00, 0xAF])?; //0x00AF
        self.write_command(Instruction::HorizontalWindowAddr2, &[0x00, 0x00])?; //0x0000
        self.write_command(Instruction::VerticalWindowAddr1, &[0x00, 0xDB])?; //0x00DB
        self.write_command(Instruction::VerticalWindowAddr2, &[0x00, 0x00])?; //0x0000
                                                                              // ----------- Adjust the Gamma Curve ----------//
        self.write_command(Instruction::GammaCtrl1, &[0x00, 0x00])?; //0x0000
        self.write_command(Instruction::GammaCtrl2, &[0x08, 0x08])?; //0x0808
        self.write_command(Instruction::GammaCtrl3, &[0x08, 0x0A])?; //0x080A
        self.write_command(Instruction::GammaCtrl4, &[0x00, 0x0A])?; //0x000A
        self.write_command(Instruction::GammaCtrl5, &[0x0A, 0x08])?; //0x0A08
        self.write_command(Instruction::GammaCtrl6, &[0x08, 0x08])?; //0x0808
        self.write_command(Instruction::GammaCtrl7, &[0x00, 0x00])?; //0x0000
        self.write_command(Instruction::GammaCtrl8, &[0x0A, 0x00])?; //0x0A00
        self.write_command(Instruction::GammaCtrl9, &[0x07, 0x10])?;
        self.write_command(Instruction::GammaCtrl10, &[0x07, 0x10])?;
        self.write_register(Instruction::DispCtrl1, Self::display_control(false, self.inverted))?;
        delay.delay_ms(50);
        self.set_display_enabled(self.display_enabled)?;
        Ok(())
    }
    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), Ili9225Error<SPI, RS, RST>>
    where
        DELAY: DelayNs,
    {
        if let Some(rst) = &mut self.rst {
            rst.set_high().map_err(Error::ResetPin)?;
            delay.delay_ms(10);
            rst.set_low().map_err(Error::ResetPin)?;
            delay.delay_ms(10);
            rst.set_high().map_err(Error::ResetPin)?;
        }
        Ok(())
    }
    fn write_command(
        &mut self,
        command: Instruction,
        params: &[u8],
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        // 发送命令给 LCD 控制器的功能。  它还具备发送命令参数的能力
        match self.spi_mode {
            SpiMode::WithDataCommandPin => {
                self.rs
                    .as_mut()
                    .ok_or(Error::MissingDataCommandPin)?
                    .set_low()
                    .map_err(Error::DataCommandPin)?;
                self.spi.write(&[command as u8]).map_err(Error::Spi)?;
                if !params.is_empty() {
                    self.start_data()?;
                    self.write_data(params)?;
                }
            }
            SpiMode::Native { id } => {
                self.spi
                    .write(&[Self::spi_start_byte(id, false, false), command as u8])
                    .map_err(Error::Spi)?;
                if !params.is_empty() {
                    self.write_data(params)?;
                }
            }
        }
        Ok(())
    }
    fn start_data(&mut self) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        match self.spi_mode {
            SpiMode::WithDataCommandPin => self
                .rs
                .as_mut()
                .ok_or(Error::MissingDataCommandPin)?
                .set_high()
                .map_err(Error::DataCommandPin),
            SpiMode::Native { .. } => Ok(()),
        }
    }
    fn write_data(&mut self, data: &[u8]) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        match self.spi_mode {
            SpiMode::WithDataCommandPin => self.spi.write(data).map_err(Error::Spi),
            SpiMode::Native { id } => {
                let mut buffer = [0u8; 33];
                if data.len() + 1 > buffer.len() {
                    return Err(Error::NativeDataChunkTooLarge);
                }
                buffer[0] = Self::spi_start_byte(id, true, false);
                buffer[1..=data.len()].copy_from_slice(data);
                self.spi.write(&buffer[..=data.len()]).map_err(Error::Spi)
            }
        }
    }
    fn write_register(
        &mut self,
        command: Instruction,
        value: u16,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.write_command(command, &value.to_be_bytes())
    }
    /// Writes a data word to the display.
    fn write_word(&mut self, value: u16) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.write_data(&value.to_be_bytes())
    }
    fn entry_mode_value(orientation: Orientation, color_order: ColorOrder) -> u16 {
        let mut value = orientation as u16;
        if matches!(color_order, ColorOrder::Bgr) {
            value |= ENTRY_MODE_BGR;
        }
        value
    }
    fn spi_start_byte(id: bool, rs: bool, rw: bool) -> u8 {
        0b0111_0000 | ((id as u8) << 2) | ((rs as u8) << 1) | (rw as u8)
    }
    fn display_control(on: bool, inverted: bool) -> u16 {
        let mut value = if on { 0x1017 } else { 0x0012 };
        if inverted {
            value |= 1 << 2;
        }
        value
    }
    pub fn set_display_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.display_enabled = enabled;
        self.write_register(
            Instruction::DispCtrl1,
            Self::display_control(enabled, self.inverted),
        )
    }
    pub fn enable_display(&mut self) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.set_display_enabled(true)
    }
    pub fn disable_display(&mut self) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.set_display_enabled(false)
    }
    fn write_words_buffered(
        &mut self,
        words: impl IntoIterator<Item = u16>,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        let mut buffer = [0; 32];
        let mut index = 0;
        for word in words {
            let as_bytes = word.to_be_bytes();
            buffer[index] = as_bytes[0];
            buffer[index + 1] = as_bytes[1];
            index += 2;
            if index >= buffer.len() {
                self.write_data(&buffer)?;
                index = 0;
            }
        }
        self.write_data(&buffer[0..index])
    }
    pub fn set_orientation(
        &mut self,
        orientation: &Orientation,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.orientation = *orientation;
        self.write_register(
            Instruction::EntryMode,
            Self::entry_mode_value(*orientation, self.color_order),
        )
    }
    pub fn set_color_order(
        &mut self,
        color_order: ColorOrder,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.color_order = color_order;
        self.write_register(
            Instruction::EntryMode,
            Self::entry_mode_value(self.orientation, self.color_order),
        )
    }
    pub fn set_inverted(&mut self, inverted: bool) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.inverted = inverted;
        self.set_display_enabled(self.display_enabled)
    }
    pub fn config(&self) -> Config {
        Config {
            spi_mode: self.spi_mode,
            color_order: self.color_order,
            orientation: self.orientation,
            inverted: self.inverted,
            display_enabled: self.display_enabled,
            width: self.width,
            height: self.height,
        }
    }

    // pub fn set_orientation(&mut self, orientation: &Orientation) -> Result<(), ()> {
    //     if self.rgb {
    //         self.write_command(Instruction::MADCTL, &[*orientation as u8])?;
    //     } else {
    //         self.write_command(Instruction::MADCTL, &[orientation as u8 | 0x08])?;
    //     }
    //     Ok(())
    // }
    /// Sets the global offset of the displayed image
    pub fn set_offset(&mut self, dx: u16, dy: u16) {
        self.dx = dx;
        self.dy = dy;
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    /// Sets the address window for the display.
    pub fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        let sx = sx + self.dx;
        let sy = sy + self.dy;
        let ex = ex + self.dx;
        let ey = ey + self.dy;

        self.write_register(Instruction::HorizontalWindowAddr2, sx)?;
        self.write_register(Instruction::HorizontalWindowAddr1, ex)?;
        self.write_register(Instruction::VerticalWindowAddr2, sy)?;
        self.write_register(Instruction::VerticalWindowAddr1, ey)?;
        self.write_register(Instruction::RamAddrSet1, sx)?;
        self.write_register(Instruction::RamAddRSet2, sy)
    }
    // Sets a pixel color at the given coords.
    pub fn set_pixel(
        &mut self,
        x: u16,
        y: u16,
        color: u16,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::GramDataReg, &[])?;
        self.start_data()?;
        self.write_word(color)
    }
    /// Writes pixel colors sequentially into the current drawing window
    pub fn write_pixels<P: IntoIterator<Item = u16>>(
        &mut self,
        colors: P,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.write_command(Instruction::GramDataReg, &[])?;
        self.start_data()?;
        for color in colors {
            self.write_word(color)?;
        }
        Ok(())
    }
    pub fn write_pixels_buffered<P: IntoIterator<Item = u16>>(
        &mut self,
        colors: P,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.write_command(Instruction::GramDataReg, &[])?;
        self.start_data()?;
        self.write_words_buffered(colors)
    }
    /// Sets pixel colors at the given drawing window
    pub fn set_pixels<P: IntoIterator<Item = u16>>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: P,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_pixels(colors)
    }
    pub fn set_pixels_buffered<P: IntoIterator<Item = u16>>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: P,
    ) -> Result<(), Ili9225Error<SPI, RS, RST>> {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_pixels_buffered(colors)
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics_core;
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::{
    draw_target::DrawTarget,
    pixelcolor::{
        raw::{RawData, RawU16},
        Rgb565,
    },
    prelude::*,
    primitives::Rectangle,
};

#[cfg(feature = "graphics")]
impl<SPI, RS, RST> DrawTarget for ILI9225<SPI, RS, RST>
where
    SPI: spi::SpiDevice,
    RS: OutputPin,
    RST: OutputPin,
{
    type Error = Ili9225Error<SPI, RS, RST>;
    type Color = Rgb565;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Only draw pixels that would be on screen
            if coord.x >= 0
                && coord.y >= 0
                && coord.x < self.width as i32
                && coord.y < self.height as i32
            {
                self.set_pixel(
                    coord.x as u16,
                    coord.y as u16,
                    RawU16::from(color).into_inner(),
                )?;
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // Clamp area to drawable part of the display target
        let drawable_area = area.intersection(&Rectangle::new(Point::zero(), self.size()));

        if drawable_area.size != Size::zero() {
            self.set_pixels_buffered(
                drawable_area.top_left.x as u16,
                drawable_area.top_left.y as u16,
                (drawable_area.top_left.x + (drawable_area.size.width - 1) as i32) as u16,
                (drawable_area.top_left.y + (drawable_area.size.height - 1) as i32) as u16,
                area.points()
                    .zip(colors)
                    .filter(|(pos, _color)| drawable_area.contains(*pos))
                    .map(|(_pos, color)| RawU16::from(color).into_inner()),
            )?;
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.set_pixels_buffered(
            0,
            0,
            self.width as u16 - 1,
            self.height as u16 - 1,
            core::iter::repeat_n(RawU16::from(color).into_inner(), (self.width * self.height) as usize),
        )
    }
}

#[cfg(feature = "graphics")]
impl<SPI, RS, RST> OriginDimensions for ILI9225<SPI, RS, RST>
where
    SPI: spi::SpiDevice,
    RS: OutputPin,
    RST: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
