use embedded_hal::{
    delay::DelayNs,
    i2c::{I2c, SevenBitAddress}
};

use crate::anyhow_utils::*;

pub fn measure_soil<I2C, D>( 
    i2c: &mut I2C,
    delay: &mut D,
) -> anyhow::Result<f32>
where
    I2C: I2c<SevenBitAddress>,
    D: DelayNs,
{

    // Begin transmission to 0x55 and send command 0x01
    i2c.write(0x55, &[0x01]).with_anyhow("Unable to write to i2c")?;

    // Delay 10ms (for humidity/temperature in parallel)
    delay.delay_ms(10);

    // Request 3 bytes from 0x55
    let mut buf = [0u8; 3];
    i2c.read(0x55, &mut buf).with_anyhow("Unable to read from moisture")?;

    let dec_h = buf[0];
    let dec_l = buf[1];
    let frac = buf[2];

    let pfdec = ((dec_h as u16) << 8) | (dec_l as u16);
    let pf = pfdec as f32 + (frac as f32 / 256.0);

    Ok(pf)
}
