#![no_std]
#![no_main]

pub mod ble;
pub mod battery;
pub mod gauge;

use core::cell::RefCell;

use bt_hci::controller::ExternalController;
use esp_hal::analog::adc::{Adc, AdcConfig};
use esp_hal::gpio::{Output, OutputConfig};
use defmt::{error, info, flush};
use embassy_executor::Spawner;
use esp_hal::ram;
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    peripherals::{Peripherals, BT },
    rtc_cntl::sleep::{RtcSleepConfig, TimerWakeupSource}
};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, time::Rate};
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::{init, EspWifiController};
use gauge::Gauge;
use heapless::Vec;
use timeseries::{Series, Deviate};
use esp_println as _;
use esp_println::println;

use crate::battery::BatteryMeasurement;
use crate::gauge::Measurement;

#[ram(rtc_fast)]
static mut MEASUREMENTS: Vec<Measurement, 10> = Vec::<Measurement, 10>::new();

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

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // generator version: 0.3.1 

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals: Peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    // let mut cfg = RtcSleepConfig::deep();
    // cfg.set_rtc_fastmem_pd_en(false);
    // let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(10));
    // let mut rtc = Rtc::new(peripherals.LPWR);
    // rtc.rwdt.enable();

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);
    info!("Embassy initialized!");
    
    /// BLE SECTION
    
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        init(
            timer1.timer0,
            Rng::new(peripherals.RNG),
            peripherals.RADIO_CLK,
        )
        .unwrap()
    );
    let bluetooth = peripherals.BT;
    // _spawner.spawn(ble_embassy_task(esp_wifi_ctrl, bluetooth)).unwrap();

    let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    ble::run(controller).await;

    
    


    // let adc_pin = peripherals.GPIO34;
    // let mut adc_config = AdcConfig::new();
    // let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
    // let adc = Adc::new(peripherals.ADC1, adc_config);

    


    // unsafe {
    //     println!("measurement: {}", MEASUREMENTS.len());
    // }

    // let output_config = OutputConfig::default();

    // let mut i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config);
    // let mut i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config);
    // let mut pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config);
    
    // let mut i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
    //     peripherals.I2C0,
    //     esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
    // )
    // .expect("I2c init failed")
    // .with_sda(i2c_pcb_sda)
    // .with_scl(i2c_pcb_scl));

    // let battery = BatteryMeasurement::new(adc, pin);
    // let mut gauge = Gauge::new(i2c_pcb_refcell, pcb_pwr, battery);

    // let measurement = gauge.sample().await;

    // unsafe {
    //     let entries = MEASUREMENTS.len();

    //     if(entries == 10) {
    //         MEASUREMENTS.clear();
    //     }

    //     MEASUREMENTS.push(measurement);
    // }

    // rtc.sleep(&cfg, &[&wakeup_source]);

    loop {}
}

#[embassy_executor::task]
async fn ble_embassy_task(
    init: &'static EspWifiController<'static>,
    bt: BT
) {
    let connector = BleConnector::new(&init, bt);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    ble::run(controller).await;
}

impl Deviate for Measurement {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        return 
            self.battery - other.battery > max_deviation.battery ||
            self.lux - other.lux > max_deviation.lux ||
            self.temperature - other.temperature > max_deviation.temperature ||
            self.humidity - other.humidity > max_deviation.humidity;
    }
}