
use core::cell::RefCell;

use esp_hal::rng::Rng;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::clock::CpuClock;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::peripherals::GPIO34;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::analog::adc::{Adc, AdcConfig};
use esp_radio::{Controller, ble::controller::BleConnector};

use trouble_host::prelude::ExternalController;

use crate::battery::BatteryMeasurement;
use crate::gauge::Gauge;
use crate::state::{DeviceState, Measurements};

pub struct ProcessorResult {
    pub next_state: DeviceState,
    pub rtc: esp_hal::rtc_cntl::Rtc<'static>
}

pub trait Processor {
    async fn awaiting_time_sync(&self, 
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        mac: [u8; 6], 
        controller: trouble_host::prelude::ExternalController<BleConnector<'_>, 20>) -> anyhow::Result<DeviceState>;

    async fn buffering(&self,
        state: &crate::state::Measurements,
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        gauge: &mut crate::gauge::Gauge<'_, GPIO34<'_>>,
        rng: esp_hal::rng::Rng
    )  -> anyhow::Result<DeviceState>;

    async fn flushing(&self,
        state: &crate::state::Measurements,
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        gauge: &mut crate::gauge::Gauge<'_, GPIO34<'_>>,
        mac: [u8; 6], 
        controller: trouble_host::prelude::ExternalController<BleConnector<'_>, 20>,
        rng: esp_hal::rng::Rng
    )  -> anyhow::Result<DeviceState>;
}

pub async fn process<P : Processor>(state: &DeviceState, processor: P) -> anyhow::Result<ProcessorResult> {

        let cpu_clock = match state {
            DeviceState::AwaitingTimeSync | DeviceState::Flush(_) => CpuClock::max(),
            DeviceState::Buffering(_) => CpuClock::_80MHz,
        };

        esp_println::logger::init_logger_from_env();
        let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(cpu_clock));
        esp_alloc::heap_allocator!(size: 72 * 1024);
        let timg0 = TimerGroup::new(peripherals.TIMG0);

        esp_rtos::start(timg0.timer0);

        let mac = esp_hal::efuse::Efuse::mac_address();
        let mut rtc = Rtc::new(peripherals.LPWR);
        let rng = Rng::new();

    
        match state {
            DeviceState::AwaitingTimeSync => {
        
                let radio = esp_radio::init().unwrap();
                let bluetooth = peripherals.BT;
                let connector = BleConnector::new(&radio, bluetooth, Default::default()).unwrap();
                let controller = ExternalController::new(connector);

                let next_state = processor.awaiting_time_sync(&rtc, mac, controller).await?;
                let processor_result = ProcessorResult { next_state, rtc };

                Ok(processor_result)
            }
            DeviceState::Buffering(measurements) => {

                let adc_pin = peripherals.GPIO34;
                let mut adc_config = AdcConfig::new();
                let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                let adc = Adc::new(peripherals.ADC1, adc_config);
                let output_config_pcb = OutputConfig::default();
    
                let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);

                let i2c_pcb = esp_hal::i2c::master::I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
                    .expect("I2c pcb init failed")
                    .with_sda(peripherals.GPIO21)
                    .with_scl(peripherals.GPIO22);
                    
                let i2c_pcb_refcell = RefCell::new(i2c_pcb);
                
                let output_config_ext = OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                    .with_pull(esp_hal::gpio::Pull::Up);

                let i2c_ext = esp_hal::i2c::master::I2c::new(peripherals.I2C1, esp_hal::i2c::master::Config::default())
                    .expect("I2c ext init failed")
                    .with_sda(peripherals.GPIO27)
                    .with_scl(peripherals.GPIO26);

                let i2c_ext_refcell = RefCell::new(i2c_ext);
            
                let battery = BatteryMeasurement::new(adc, pin);
                let mut gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);

                let next_state = processor.buffering(&measurements, &rtc, &mut gauge, rng).await?;
                let processor_result = ProcessorResult { next_state, rtc };

                Ok(processor_result)
            }
            DeviceState::Flush(measurements) => {
                let adc_pin = peripherals.GPIO34;
                let mut adc_config = AdcConfig::new();
                let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                let adc = Adc::new(peripherals.ADC1, adc_config);
                let output_config_pcb = OutputConfig::default();
    
                let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);

                let i2c_pcb = esp_hal::i2c::master::I2c::new(peripherals.I2C0, esp_hal::i2c::master::Config::default())
                    .expect("I2c pcb init failed")
                    .with_sda(peripherals.GPIO21)
                    .with_scl(peripherals.GPIO22);
                    
                let i2c_pcb_refcell = RefCell::new(i2c_pcb);
                
                let output_config_ext = OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                    .with_pull(esp_hal::gpio::Pull::Up);

                let i2c_ext = esp_hal::i2c::master::I2c::new(peripherals.I2C1, esp_hal::i2c::master::Config::default())
                    .expect("I2c ext init failed")
                    .with_sda(peripherals.GPIO27)
                    .with_scl(peripherals.GPIO26);

                let i2c_ext_refcell = RefCell::new(i2c_ext);
            
                let battery = BatteryMeasurement::new(adc, pin);
                let mut gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);

                let radio = esp_radio::init().unwrap();
                let bluetooth = peripherals.BT;
                let connector = BleConnector::new(&radio, bluetooth, Default::default()).unwrap();
                let controller = ExternalController::new(connector);

                let next_state = processor.flushing(&measurements, &rtc, &mut gauge, mac, controller, rng).await?;
                let processor_result = ProcessorResult { next_state, rtc };

                Ok(processor_result)
            }
        }
    }