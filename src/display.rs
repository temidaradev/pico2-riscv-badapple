use embedded_graphics::{image::ImageRaw, pixelcolor::BinaryColor, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use crate::rle::FRAME_SIZE;

pub type Display<I2C> = Ssd1306<
    I2CInterface<I2C>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;

pub fn init_display<I2C>(i2c: I2C) -> Display<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();
    display.flush().unwrap();

    display
}

pub fn render_frame<I2C>(display: &mut Display<I2C>, frame_buffer: &[u8; FRAME_SIZE]) -> bool
where
    I2C: embedded_hal::i2c::I2c,
{
    let image = ImageRaw::<BinaryColor>::new(frame_buffer, 128);
    if image.draw(display).is_err() {
        return false;
    }
    display.flush().is_ok()
}
