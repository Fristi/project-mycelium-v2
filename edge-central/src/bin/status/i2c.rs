use crate::status::{Status, StatusSummary};
use embedded_graphics::{
    mono_font::{
        MonoTextStyle, MonoTextStyleBuilder, ascii::{FONT_6X10, FONT_9X18_BOLD}
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

    fn show(&mut self, status: &StatusSummary) -> Result<()> {
        let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        // --- First row: time range header ---
        let from_str = status.from.format("%H:%M").to_string();
        let till_str = status.till.format("%H:%M").to_string();
        let header = format!("{} till {}", from_str, till_str);

        Text::with_alignment(&header, Point::new(64, 8), style, Alignment::Center)
            .draw(&mut self.display).map_err(|e| anyhow::anyhow!("Unable to draw: {:?}", e))?;

        // --- Sensor rows ---
        let lines = [
            format!("Temp      {:.1} C", status.temperature),
            format!("Humidity  {:.1} RH", status.humidity),
            format!("Soil      {:.1} pF", status.soil_moisture),
            format!("Light     {:.0} lx", status.light),
        ];

        let mut y = 20;
        for line in lines {
            Text::new(&line, Point::new(0, y), style)
                .draw(&mut self.display)
                .map_err(|e| anyhow::anyhow!("Unable to draw: {:?}", e))?;

            y += 10; // Line spacing
        }
        // Clear display buffer
        self.display.clear(BinaryColor::On).map_err(|e| anyhow::anyhow!("Unable to clear display: {:?}", e))?;

        Ok(())
    }
}