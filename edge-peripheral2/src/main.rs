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

esp_bootloader_esp_idf::esp_app_desc!();

struct NoopProcessor;

impl Processor for NoopProcessor {
    async fn awaiting_time_sync(&self, state: &state::DeviceState, rtc: esp_hal::rtc_cntl::Rtc<'_>, mac: [u8; 6], controller: trouble_host::prelude::ExternalController<esp_radio::ble::controller::BleConnector<'_>, 20>) -> anyhow::Result<state::DeviceState> {
        Ok(state::DeviceState::AwaitingTimeSync)
    }
}

#[esp_rtos::main]
async fn main(_s: Spawner) {
    let state = get();
    match process(state, NoopProcessor {}).await {
        Ok(next_state) => {
            set(next_state)
        },
        Err(err) => {
            todo!()
        }
    }
}
