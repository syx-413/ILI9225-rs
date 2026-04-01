TFT_22_ILI9225
==============

SPI driver for the ILI9225 LCD display controller

## Introduction

This is a library for the ILI9225 based 2.2" 176x220 TFT LCD shields commonly found on eBay, originally forked from the screen_4D_22_library library. The ability to use GLCD fonts has been added and the syntax has been changed to match the Adafruit libraries somewhat.

*Note that there is a commonly available 2.2" 240x320 TFT module very similar in appearance but using the
ILI9341 driver.*

![ILI9225](/images/ILI9225_TFT.jpg)

thanks https://github.com/sajattack/st7735-lcd-rs

## Status

This crate currently provides:

- hardware reset and power-on initialization
- address window and pixel/area write helpers
- runtime display/config control
- both `DC/RS` pin SPI and native ILI9225 SPI start-byte transport modes
- optional `embedded-graphics-core` integration through the default `graphics` feature

## Hardware modes

This crate supports two different wiring models:

- `SpiMode::WithDataCommandPin`
  Use this for common TFT modules that expose a dedicated `DC` or `RS` pin in addition to `CS`, `SCK`, `MOSI`, and optional `RST`.
- `SpiMode::Native { id }`
  Use this when the hardware follows the ILI9225 datasheet SPI protocol directly and does not expose a separate `DC/RS` pin. In this mode the driver sends the ILI9225 start byte with embedded `RS` and `R/W` bits.

As a rule of thumb:

- If the module pinout includes `DC` or `RS`, start with `SpiMode::WithDataCommandPin`.
- If you are wiring directly to the controller SPI pins described in the datasheet, use `SpiMode::Native`.

## Errors

Public fallible APIs now return a typed `Error<_, _, _>` instead of `()`, so SPI,
`DC/RS`, and reset failures can be distinguished during bring-up.

The main variants are:

- `Error::Spi(...)`
- `Error::DataCommandPin(...)`
- `Error::ResetPin(...)`
- `Error::MissingDataCommandPin`
- `Error::NativeDataChunkTooLarge`

## Configuration

The driver is configured with [`Config`] in code. The most important fields are:

- `spi_mode`
  Selects module-style SPI with a `DC/RS` pin or native ILI9225 SPI start-byte mode.
- `color_order`
  `ColorOrder::Rgb` or `ColorOrder::Bgr`.
- `orientation`
  One of `Portrait`, `Landscape`, `PortraitSwapped`, or `LandscapeSwapped`.
- `inverted`
  Controls display inversion through `Display Control 1`.
- `display_enabled`
  Controls whether `init()` finishes with the panel enabled or disabled.
- `width`, `height`
  Logical drawable size used by the driver and `embedded-graphics`.

## Minimal usage

```rust
use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::*;
use lcd_ili9225_rs::{
    ColorOrder, Config, Ili9225Error, ILI9225, Orientation, SpiMode,
};

fn init_display<SPI, RS, RST, D>(
    spi: SPI,
    rs: RS,
    rst: Option<RST>,
    delay: &mut D,
) -> Result<ILI9225<SPI, RS, RST>, Ili9225Error<SPI, RS, RST>>
where
    SPI: embedded_hal::spi::SpiDevice,
    RS: embedded_hal::digital::OutputPin,
    RST: embedded_hal::digital::OutputPin,
    D: embedded_hal::delay::DelayNs,
{
    let mut config = Config::new(176, 220);
    config.spi_mode = SpiMode::WithDataCommandPin;
    config.color_order = ColorOrder::Rgb;
    config.orientation = Orientation::Portrait;
    config.display_enabled = true;

    let mut display = ILI9225::new_with_config(spi, rs, rst, config);
    display.init(delay)?;
    display.clear(Rgb565::BLACK)?;
    Ok(display)
}
```

At runtime you can also update the panel state:

- `display.set_display_enabled(...)`
- `display.set_orientation(...)`
- `display.set_color_order(...)`
- `display.set_inverted(...)`
- `display.config()`

## Native ILI9225 SPI mode

If your hardware does not expose a dedicated `DC/RS` pin and instead follows the
ILI9225 datasheet SPI protocol, use `SpiMode::Native` with `new_native_spi(...)`.

```rust
use lcd_ili9225_rs::{Config, ILI9225, SpiMode};

let mut config = Config::new(176, 220);
config.spi_mode = SpiMode::Native { id: false };

let display = ILI9225::new_native_spi(spi, rst, config);
```

## Notes

- The current driver targets write operations needed for initialization and drawing.
- The crate is `no_std`.
- The default feature enables `embedded-graphics-core` support.
- See `examples/dc_pin.rs` for module-style SPI with a `DC/RS` pin.
- See `examples/native_spi.rs` for native ILI9225 SPI start-byte mode.
