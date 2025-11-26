use crate::status::Status;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use sqlx::any;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use ssd1306::mode::BufferedGraphicsMode;
use linux_embedded_hal::I2cdev;
use anyhow::*;


pub struct I2cStatus {
    display: Ssd1306<I2CInterface<I2cdev>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>
}

impl I2cStatus {
    pub fn new(path: &str) -> Result<I2cStatus> {
        let dev = I2cdev::new(path)?;
        let mut display= Ssd1306::new(
            I2CDisplayInterface::new(dev),
            DisplaySize128x32,
            DisplayRotation::Rotate0
        ).into_buffered_graphics_mode();

        display.init().map_err(|e| anyhow::anyhow!("Unable to init: {:?}", e))?;

        Ok(Self { display })
    }
}

impl Status for I2cStatus {

    fn show(&mut self, small_text: &str, big_text: &str) -> Result<()> {
        // Specify different text styles
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        let text_style_big = MonoTextStyleBuilder::new()
            .font(&FONT_9X18_BOLD)
            .text_color(BinaryColor::On)
            .build();

        // Fill display bufffer with a centered text with two lines (and two text
        // styles)
        Text::with_alignment(
            big_text,
            self.display.bounding_box().center() + Point::new(0, 0),
            text_style_big,
            Alignment::Center,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Unable to draw: {:?}", e))?;

        Text::with_alignment(
            small_text,
            self.display.bounding_box().center() + Point::new(0, 14),
            text_style,
            Alignment::Center,
        )
        .draw(&mut self.display)
        .map_err(|e| anyhow::anyhow!("Unable to draw: {:?}", e))?;

        // Write buffer to display
        self.display.flush().map_err(|e| anyhow::anyhow!("Unable to flush display: {:?}", e))?;
        // Clear display buffer
        self.display.clear(BinaryColor::On).map_err(|e| anyhow::anyhow!("Unable to clear display: {:?}", e))?;

        Ok(())
    }
}