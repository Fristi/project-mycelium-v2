use core::cell::RefCell;

use bh1730fvc::{blocking::BH1730FVC};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{analog::adc::AdcChannel, gpio::Output, i2c::master::I2c, Blocking};
use edge_protocol::Measurement;
use crate::battery::BatteryMeasurement;

pub struct Gauge<'a, P : AdcChannel> {
    i2c: RefCell<I2c<'a, Blocking>>,
    pcb_pwr: Output<'a>,
    bm: BatteryMeasurement<'a, P>
}

impl <'a, P : AdcChannel> Gauge<'a, P> {
    pub fn new(i2c: RefCell<I2c<'a, Blocking>>, pcb_pwr: Output<'a>, bm: BatteryMeasurement<'a, P>) -> Self {
        Self {
            i2c,
            pcb_pwr,
            bm
        }
    }

    pub async fn sample(&mut self) -> anyhow::Result<Measurement> {
        self.pcb_pwr.set_high();

        Timer::after_millis(300).await;

        let mut i2c_pcb_sht = RefCellDevice::new(&self.i2c);
        let mut i2c_pcb_bh1730fvc = RefCellDevice::new(&self.i2c);

        let mut sht = shtcx::blocking::shtc3(RefCellDevice::new(&self.i2c));
        let mut delay = Delay;
        let mut bh1730fvc = BH1730FVC::new(&mut delay, &mut i2c_pcb_bh1730fvc)
            .with_anyhow("BH1730FVC init failed")?;
        
        sht.start_measurement(shtcx::blocking::PowerMode::NormalMode)
            .with_anyhow("SHT start measurement failed")?;
        bh1730fvc.set_mode(bh1730fvc::Mode::SingleShot, &mut i2c_pcb_bh1730fvc)
            .with_anyhow("BH1730FVC set mode failed")?;

        Timer::after_millis(300).await;

        let lux = bh1730fvc.read_ambient_light_intensity(&mut i2c_pcb_sht).with_anyhow("BH1730FVC read failed")?;
        let battery = self.bm.sample();
        
        Timer::after_millis(300).await;
        let measurement = sht.get_measurement_result().with_anyhow("SHT read failed")?;

        self.pcb_pwr.set_low();

        let measurement = Measurement {
            battery,
            lux,
            temperature: measurement.temperature.as_degrees_celsius(),
            humidity: measurement.humidity.as_percent()
        };

        Ok(measurement)
    }
}

trait ResultAny<T, E> {
    fn with_anyhow(self, ctx: &'static str) -> Result<T, anyhow::Error>;
}

impl<T, E: core::fmt::Debug> ResultAny<T, E> for Result<T, E> {
    fn with_anyhow(self, ctx: &'static str) -> Result<T, anyhow::Error> {
        self.map_err(|e| anyhow::anyhow!("{}: {:?}", ctx, e))
    }
}

