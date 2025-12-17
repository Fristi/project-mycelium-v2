
use esp_hal::rng::Rng;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::rtc_cntl::Rtc;
use esp_radio::{Controller, ble::controller::BleConnector};

use trouble_host::prelude::ExternalController;

use crate::gauge::Gauge;
use crate::state::DeviceState;

pub trait Processor {
    async fn awaiting_time_sync(&self, state: &DeviceState, rtc: esp_hal::rtc_cntl::Rtc<'_>, mac: [u8; 6], controller: trouble_host::prelude::ExternalController<BleConnector<'_>, 20>) -> anyhow::Result<DeviceState>;
}

pub async fn process<P : Processor>(state: &DeviceState, processor: P) -> anyhow::Result<DeviceState> {

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

                let next_state = processor.awaiting_time_sync(state, rtc, mac, controller).await?;

                Ok(next_state)
            }
            DeviceState::Buffering(measurements) => {

                // let adc_pin = peripherals.GPIO34;
                // let mut adc_config = AdcConfig::new();
                // let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                // let adc = Adc::new(peripherals.ADC1, adc_config);
                // let output_config_pcb = OutputConfig::default();
            
                // let i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config_pcb);
                // let i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config_pcb);
                // let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);
                
                // let i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                //     peripherals.I2C0,
                //     esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
                // )
                // .expect("I2c init failed")
                // .with_sda(i2c_pcb_sda)
                // .with_scl(i2c_pcb_scl));

                
                // let output_config_ext = OutputConfig::default()
                //     .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                //     .with_pull(esp_hal::gpio::Pull::Up);

                // let i2c_ext_sda = Output::new(peripherals.GPIO27, esp_hal::gpio::Level::Low, output_config_ext);
                // let i2c_ext_scl = Output::new(peripherals.GPIO26, esp_hal::gpio::Level::Low, output_config_ext);

                // let i2c_ext_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                //     peripherals.I2C1,
                //     esp_hal::i2c::master::Config::default()
                // )
                // .expect("I2c init failed")
                // .with_sda(i2c_ext_sda)
                // .with_scl(i2c_ext_scl));
            
                // let battery = BatteryMeasurement::new(adc, pin);
                // let gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);


                // Self::Buffering { rtc, gauge, measurements, rng }

                todo!()
            },
            DeviceState::Flush(measurements) => {

                // let esp_wifi_ctrl = &*mk_static!(
                //     EspWifiController<'static>,
                //     init(
                //         timg0.timer0,
                //         rng,
                //         peripherals.RADIO_CLK,
                //     )
                //     .unwrap()
                // );
                // let bluetooth = peripherals.BT;
                // let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth);

                // let controller: ExternalController<_, 20> = ExternalController::new(connector);

                // let adc_pin = peripherals.GPIO34;
                // let mut adc_config = AdcConfig::new();
                // let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                // let adc = Adc::new(peripherals.ADC1, adc_config);
                // let output_config_pcb = OutputConfig::default();
            
                // let i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config_pcb);
                // let i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config_pcb);
                // let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);
                
                // let i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                //     peripherals.I2C0,
                //     esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
                // )
                // .expect("I2c init failed")
                // .with_sda(i2c_pcb_sda)
                // .with_scl(i2c_pcb_scl));

                
                // let output_config_ext = OutputConfig::default()
                //     .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                //     .with_pull(esp_hal::gpio::Pull::Up);

                // let i2c_ext_sda = Output::new(peripherals.GPIO27, esp_hal::gpio::Level::Low, output_config_ext);
                // let i2c_ext_scl = Output::new(peripherals.GPIO26, esp_hal::gpio::Level::Low, output_config_ext);

                // let i2c_ext_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                //     peripherals.I2C1,
                //     esp_hal::i2c::master::Config::default()
                // )
                // .expect("I2c init failed")
                // .with_sda(i2c_ext_sda)
                // .with_scl(i2c_ext_scl));
            
                // let battery = BatteryMeasurement::new(adc, pin);
                // let gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);

                // Self::Flush { rtc, mac, gauge, measurements, ble: controller, rng }

                todo!()
            }
        }
    }