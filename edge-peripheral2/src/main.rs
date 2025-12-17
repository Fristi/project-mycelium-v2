#![no_std]
#![no_main]

mod processor;
mod anyhow_utils;
mod battery;
mod moisture;
mod gauge;
mod state;

use embassy_executor::Spawner;
use crate::{processor::process, processor::Processor, state::get, state::set};
use {esp_alloc as _, esp_backtrace as _};
use esp_hal::{peripherals::GPIO34, rtc_cntl::sleep::{RtcSleepConfig, TimerWakeupSource}};

esp_bootloader_esp_idf::esp_app_desc!();

struct NoopProcessor;

impl Processor for NoopProcessor {
    async fn awaiting_time_sync(&self, 
        state: &state::DeviceState, 
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        mac: [u8; 6], 
        controller: trouble_host::prelude::ExternalController<esp_radio::ble::controller::BleConnector<'_>, 20>) -> anyhow::Result<state::DeviceState> {
        
        Ok(state::DeviceState::AwaitingTimeSync)
    }
    
    async fn buffering(&self,
        state: &state::DeviceState, 
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        gauge: crate::gauge::Gauge<'_, GPIO34<'_>>,
        rng: esp_hal::rng::Rng
    )  -> anyhow::Result<state::DeviceState> {

        Ok(state::DeviceState::AwaitingTimeSync)
    }

    
}

#[esp_rtos::main]
async fn main(_s: Spawner) {
    let state = get();
    let mut cfg = RtcSleepConfig::deep();
    cfg.set_rtc_fastmem_pd_en(false);
    let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(1 * 10));

    match process(state, NoopProcessor {}).await {
        Ok(result) => {
            set(result.next_state);
            let mut rtc = result.rtc;
            rtc.sleep(&cfg, &[&wakeup_source]);
        },
        Err(err) => {
            todo!()
        }
    }
}
