#![no_std]
#![no_main]

mod ble;

use embassy_executor::Spawner;
use esp_hal::{clock::CpuClock, rtc_cntl::{sleep::{RtcSleepConfig, TimerWakeupSource}, Rtc}, timer::timg::TimerGroup};
use esp_wifi::ble::controller::BleConnector;
use trouble_host::prelude::ExternalController;
use {esp_alloc as _, esp_backtrace as _};
use esp_println as _;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut cfg = RtcSleepConfig::deep();
    cfg.set_rtc_fastmem_pd_en(false);
    let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(10));
    let mut rtc = Rtc::new(peripherals.LPWR);
    rtc.rwdt.enable();

    let init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .expect("Failed to initialize Bluetooth");

    esp_hal_embassy::init(timg0.timer1);

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, bluetooth);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    ble::run(controller, &mut rtc).await;
}