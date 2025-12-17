#![no_std]
#![no_main]

mod anyhow_utils;
mod battery;
mod moisture;
mod gauge;

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::ExternalController;
use {esp_alloc as _, esp_backtrace as _};
use edge_protocol::Measurement;
use timeseries::Series;
use chrono::NaiveDateTime;
use esp_hal::ram;

pub enum DeviceState {
    AwaitingTimeSync,
    Buffering(Series<6, NaiveDateTime, Measurement>),
    Flush(Series<6, NaiveDateTime, Measurement>)
}


// TODO: This is a hack to get the state of the device across the different states.
// It is not thread safe and should be replaced with a more robust solution.
// see: https://stackoverflow.com/questions/79177001/esp-no-std-rust-persist-data-during-deep-sleeps
#[ram(unstable(rtc_fast))]
static mut STATE: DeviceState = DeviceState::AwaitingTimeSync;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    let radio = esp_radio::init().unwrap();
    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(&radio, bluetooth, Default::default()).unwrap();
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    loop {

    }
}