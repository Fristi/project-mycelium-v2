use bt_hci::cmd::info;
use defmt::info;
use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress},
};
use crate::anyhow_utils::*;

/// Soil sensor state
pub struct SoilSensor<I2C> {
    i2c: I2C
}

impl<I2C> SoilSensor<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()>
    where
        I2C: I2c<SevenBitAddress>
    {
        self.i2c
            .write(0x55 | 0, &[0x10 | 0x01])
            .with_anyhow("Unable to start soil conversion")?;

        self.i2c
            .write(0x55 | 0, &[0x01])
            .with_anyhow("Unable to start soil conversion")?;

        Ok(())
    }

    pub fn read<D>(&mut self, delay: &mut D) -> anyhow::Result<f32>
    where
        I2C: I2c<SevenBitAddress>,
        D: DelayNs,
    {
        self.i2c
            .write(0x55 | 0, &[0x10 | 0x02])
            .with_anyhow("Unable to trigger soil read")?;

        delay.delay_us(150);

        let mut buf = [0u8; 3];
        self.i2c
            .read(0x55 | 1, &mut buf)
            .with_anyhow("Unable to read soil data")?;

        let d0 = buf[0];
        let d1 = buf[1];
        let d2 = buf[2];

        info!("d0: {}, d1: {}, d2: {}", d0, d1, d2);

        let pf = d0 as f32 + (d1 as f32 * 256.0) + (d2 as f32 / 256.0);
        
        Ok(pf)
    }
}
