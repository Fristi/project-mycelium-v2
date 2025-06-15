#![no_std]
#![no_main]

pub mod ble;
pub mod battery;
pub mod gauge;
pub mod types;

// Basics
use core::cell::RefCell;

// Logging
use defmt::{error, flush, info, println, Debug2Format};

// Embassy
use embassy_executor::Spawner;

// ESP-HAL
use esp_hal::analog::adc::{Adc, AdcConfig};
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::i2c::master::BusTimeout;
use esp_hal::ram;
use esp_hal::rng::Rng;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    peripherals::{Peripherals},
    rtc_cntl::sleep::{RtcSleepConfig, TimerWakeupSource}
};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, time::Rate};

// BLE 
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::{init, EspWifiController};
use bt_hci::controller::ExternalController;

// This is used to print to the console, if we don't it will crash the build: undefined reference to _defmt_flush
use esp_println as _;
use timeseries::Series;

use crate::gauge::Gauge;
use crate::battery::BatteryMeasurement;
use crate::types::{EdgeState, Measurement};

#[ram(rtc_fast)]
static mut STATE: EdgeState = EdgeState::WaitingForTimeSync;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Panic occurred: {}", defmt::Display2Format(info));
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
// #[esp_hal_embassy::main]
// async fn main(_spawner: Spawner) {
//     let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
//     let peripherals: Peripherals = esp_hal::init(config);

//     esp_alloc::heap_allocator!(size: 72 * 1024);

//     let mut rtc = Rtc::new(peripherals.LPWR);
//     rtc.rwdt.enable();

//     let timer0 = TimerGroup::new(peripherals.TIMG1);
//     esp_hal_embassy::init(timer0.timer0);
//     info!("Embassy initialized!");

//     let mut cfg = RtcSleepConfig::deep();
//     cfg.set_rtc_fastmem_pd_en(false);
//     let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(10 * 60));
//     rtc.sleep(&cfg, &[&wakeup_source]);

//     loop {
//         info!("Waiting 1 second");
//         embassy_time::Timer::after_secs(1).await;
//     }
// }

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // generator version: 0.3.1 

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals: Peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let mut rtc = Rtc::new(peripherals.LPWR);
    rtc.rwdt.enable();

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);
    info!("Embassy initialized!");

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

    let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth);
    let controller: ExternalController<BleConnector<'static>, 20> = ExternalController::new(connector);
    
    // let adc_pin = peripherals.GPIO34;
    // let mut adc_config = AdcConfig::new();
    // let pin = adc_config.enable_pin(adc_pin, esp_hal::analog::adc::Attenuation::_11dB);
    // let adc = Adc::new(peripherals.ADC1, adc_config);
    // let output_config = OutputConfig::default();

    // let i2c_pcb_sda = Output::new(peripherals.GPIO21, esp_hal::gpio::Level::Low, output_config);
    // let i2c_pcb_scl = Output::new(peripherals.GPIO22, esp_hal::gpio::Level::Low, output_config);
    // let pcb_pwr = Output::new(peripherals.GPIO23, esp_hal::gpio::Level::High, output_config);
    
    // let i2c_pcb_refcell = RefCell::new(esp_hal::i2c::master::I2c::new(
    //     peripherals.I2C0,
    //     esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)).with_timeout(BusTimeout::Maximum),
    // )
    // .expect("I2c init failed")
    // .with_sda(i2c_pcb_sda)
    // .with_scl(i2c_pcb_scl));

    // let battery = BatteryMeasurement::new(adc, pin);
    // let mut gauge = Gauge::new(i2c_pcb_refcell, pcb_pwr, battery);

    #[allow(static_mut_refs)]
    let mut state = unsafe { STATE.clone() };
    
    println!("Starting main loop: {:?}", Debug2Format(&state));

    match &mut state {
        EdgeState::WaitingForTimeSync => {
            ble::run(controller, &mut rtc).await;
            state = EdgeState::Buffering { buffer: Series::new(Measurement::max_deviation()) };
            unsafe { STATE = state; }
            // esp_hal::system::software_reset_cpu(esp_hal::system::Cpu::AppCpu);

        },
        EdgeState::Buffering { buffer } => {
            // let measurement = gauge.sample().await;
            let now = rtc.current_time();
            // buffer.append_monotonic(now, measurement);
            
            if buffer.is_full() {
                // start BLE
            } else {
                unsafe { STATE = state; }
                let mut cfg = RtcSleepConfig::deep();
                cfg.set_rtc_fastmem_pd_en(false);
                let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(10 * 60));
                rtc.sleep(&cfg, &[&wakeup_source]);
            }
            
        }
    }

    loop {}
}