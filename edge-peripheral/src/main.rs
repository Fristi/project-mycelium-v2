#![no_std]
#![no_main]

pub mod ble;
pub mod battery;
pub mod gauge;
pub mod types;
pub mod moisture;
pub mod anyhow_utils;

use core::cell::RefCell;
use bt_hci::controller::ExternalController;
use chrono::NaiveDateTime;
use edge_protocol::{Measurement, MeasurementSerieEntry};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Timer};
use esp_hal::analog::adc::{Adc, AdcConfig};
use esp_hal::gpio::{GpioPin, Output, OutputConfig};
use defmt::{error, flush, info};
use embassy_executor::Spawner;
use esp_hal::i2c::master::BusTimeout;
use esp_hal::ram;
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    peripherals::Peripherals,
    rtc_cntl::sleep::{RtcSleepConfig, TimerWakeupSource}
};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, time::Rate};
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::{init, EspWifiController};
use gauge::Gauge;
use esp_println::{self as _};
use heapless::Vec;
use timeseries::Series;

use crate::battery::BatteryMeasurement;
use crate::types::DeviceState;


// TODO: This is a hack to get the state of the device across the different states.
// It is not thread safe and should be replaced with a more robust solution.
// see: https://stackoverflow.com/questions/79177001/esp-no-std-rust-persist-data-during-deep-sleeps
#[ram(rtc_fast)]
static mut STATE: DeviceState = DeviceState::AwaitingTimeSync;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {

    let mut cfg = RtcSleepConfig::deep();
    cfg.set_rtc_fastmem_pd_en(false);
    let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(1 * 10));
    let state = unsafe { &STATE };
    let boot_args = DeviceBootArgs::boot(&state);
    
    match boot_args {
        DeviceBootArgs::AwaitingTimeSync { mut rtc, mac, ble } => {
            
            info!("Awaiting time sync");

            let future = select(
                ble::run(ble, &mut rtc, mac.clone(), Vec::new()),
                Timer::after(Duration::from_secs(10))
            );

            match future.await {
                Either::First(_) => {
                    info!("Awaiting time sync: done");
                    unsafe {
                        let series: Measurements = Series::new(Measurement::MAX_DEVIATION);
                        let new_state = DeviceState::Buffering(series);
                        STATE = new_state;
                    }
                },
                Either::Second(_) => {}
            };

            rtc.sleep(&cfg, &[&wakeup_source]);
        }
        DeviceBootArgs::Buffering { mut rtc, mut gauge, measurements, mut rng } => {
            info!("Buffering, current num entries {0}", measurements.buckets.len());

            let measurement = gauge.sample().await;

            match measurement {
                Ok(m) => {
                    let mut new_measurements = (*measurements).clone();
                    
                    new_measurements.append_monotonic(rtc.current_time(), m);

                    let new_state = if new_measurements.is_full() {
                        DeviceState::Flush(new_measurements)
                    } else {  
                        DeviceState::Buffering(new_measurements)
                    };

                    unsafe {
                        STATE = new_state;
                    }
                },
                Err(_) => {}
            };

            rtc.sleep(&cfg, &[&wakeup_source]);
        }
        DeviceBootArgs::Flush { mut rtc, mac, mut gauge, measurements, ble, mut rng } => {
            info!("Flushing");

            let entries: Vec<MeasurementSerieEntry, 6> = measurements
                .buckets
                .iter()
                .map(|entry| MeasurementSerieEntry { timestamp: entry.range.start, measurement: entry.value })
                .collect();
            
            let future = select(
                ble::run(ble, &mut rtc, mac.clone(), entries),
                Timer::after(Duration::from_secs(10))
            );

            match future.await {
                Either::First(_) => {
                    match gauge.sample().await {
                        Ok(m) => {
                            let mut new_measurements: Measurements = Series::new(Measurement::MAX_DEVIATION);

                            new_measurements.append_monotonic(rtc.current_time(), m);

                            unsafe {
                                STATE = DeviceState::Buffering(new_measurements);
                            }
                        },
                        Err(_) => {}
                    }
                },
                Either::Second(_) => {}
            };
        

            rtc.sleep(&cfg, &[&wakeup_source]);
        }
    };
}


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
       // Log the panic message with defmt
    error!("Panic occurred: {}", defmt::Display2Format(info));

    // Ensure logs are flushed before halt
    flush();

    loop {}
}

extern crate alloc;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

type Measurements = Series<6, NaiveDateTime, Measurement>;

pub enum DeviceBootArgs<'a> {
    AwaitingTimeSync { rtc: Rtc<'a>, mac: [u8; 6], ble: ExternalController<BleConnector<'a>, 20> },
    Buffering { rtc: Rtc<'a>, gauge: Gauge<'a, GpioPin<34>>, measurements: &'a Measurements, rng: Rng },
    Flush { rtc: Rtc<'a>, mac: [u8; 6], gauge: Gauge<'a, GpioPin<34>>, measurements: &'a Measurements, ble: ExternalController<BleConnector<'a>, 20>, rng: Rng }
}

impl <'a> DeviceBootArgs<'a> {
    pub fn boot(state: &'a DeviceState) -> Self {

        let cpu_clock = match state {
            DeviceState::AwaitingTimeSync | DeviceState::Flush(_) => CpuClock::max(),
            DeviceState::Buffering(_) => CpuClock::_80MHz,
        };
        let config = esp_hal::Config::default().with_cpu_clock(cpu_clock);
        let mut peripherals: Peripherals = esp_hal::init(config);

        esp_alloc::heap_allocator!(size: 72 * 1024);

        let mac = esp_hal::efuse::Efuse::mac_address();
        let mut rtc = Rtc::new(peripherals.LPWR);
        let rng = Rng::new(peripherals.RNG);
    
        let timg0 = TimerGroup::new(peripherals.TIMG0);
        esp_hal_embassy::init(timg0.timer1);

        info!("Embassy initialized!");

        match state {
            DeviceState::AwaitingTimeSync => {
        
                // let timg0 = TimerGroup::new(peripherals.TIMG0);
                let esp_wifi_ctrl = &*mk_static!(
                    EspWifiController<'static>,
                    init(
                        timg0.timer0,
                        rng,
                        peripherals.RADIO_CLK,
                    )
                    .unwrap()
                );
                let bluetooth = peripherals.BT;
                let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth);
                let ble: ExternalController<_, 20> = ExternalController::new(connector);

                Self::AwaitingTimeSync { rtc, mac, ble }
            }
            DeviceState::Buffering(measurements) => {

                let adc_pin = peripherals.GPIO34;
                let mut adc_config = AdcConfig::new();
                let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                let adc = Adc::new(peripherals.ADC1, adc_config);
                let output_config_pcb = OutputConfig::default();
            
                let i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config_pcb);
                let i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config_pcb);
                let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);
                
                let i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                    peripherals.I2C0,
                    esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
                )
                .expect("I2c init failed")
                .with_sda(i2c_pcb_sda)
                .with_scl(i2c_pcb_scl));

                
                let output_config_ext = OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                    .with_pull(esp_hal::gpio::Pull::Up);

                let i2c_ext_sda = Output::new(peripherals.GPIO27, esp_hal::gpio::Level::Low, output_config_ext);
                let i2c_ext_scl = Output::new(peripherals.GPIO26, esp_hal::gpio::Level::Low, output_config_ext);

                let i2c_ext_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                    peripherals.I2C1,
                    esp_hal::i2c::master::Config::default()
                )
                .expect("I2c init failed")
                .with_sda(i2c_ext_sda)
                .with_scl(i2c_ext_scl));
            
                let battery = BatteryMeasurement::new(adc, pin);
                let gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);


                Self::Buffering { rtc, gauge, measurements, rng }
            },
            DeviceState::Flush(measurements) => {
        
    
                let esp_wifi_ctrl = &*mk_static!(
                    EspWifiController<'static>,
                    init(
                        timg0.timer0,
                        rng,
                        peripherals.RADIO_CLK,
                    )
                    .unwrap()
                );
                let bluetooth = peripherals.BT;
                let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth);

                let controller: ExternalController<_, 20> = ExternalController::new(connector);

                let adc_pin = peripherals.GPIO34;
                let mut adc_config = AdcConfig::new();
                let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
                let adc = Adc::new(peripherals.ADC1, adc_config);
                let output_config_pcb = OutputConfig::default();
            
                let i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config_pcb);
                let i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config_pcb);
                let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config_pcb);
                
                let i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                    peripherals.I2C0,
                    esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
                )
                .expect("I2c init failed")
                .with_sda(i2c_pcb_sda)
                .with_scl(i2c_pcb_scl));

                
                let output_config_ext = OutputConfig::default()
                    .with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain)
                    .with_pull(esp_hal::gpio::Pull::Up);

                let i2c_ext_sda = Output::new(peripherals.GPIO27, esp_hal::gpio::Level::Low, output_config_ext);
                let i2c_ext_scl = Output::new(peripherals.GPIO26, esp_hal::gpio::Level::Low, output_config_ext);

                let i2c_ext_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
                    peripherals.I2C1,
                    esp_hal::i2c::master::Config::default()
                )
                .expect("I2c init failed")
                .with_sda(i2c_ext_sda)
                .with_scl(i2c_ext_scl));
            
                let battery = BatteryMeasurement::new(adc, pin);
                let gauge = Gauge::new(i2c_pcb_refcell, i2c_ext_refcell, pcb_pwr, battery);

                Self::Flush { rtc, mac, gauge, measurements, ble: controller, rng }
            }
        }
    }
}
