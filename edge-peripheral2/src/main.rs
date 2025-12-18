#![no_std]
#![no_main]

mod processor;
mod battery;
mod moisture;
mod gauge;
mod state;
mod utils;

use chrono::NaiveDateTime;
use edge_protocol::Measurement;
use embassy_executor::Spawner;
use timeseries::Series;
use crate::{processor::process, processor::Processor, state::get_device_state, state::set_device_state};
use {esp_alloc as _, esp_backtrace as _};
use esp_hal::{peripherals::GPIO34, rtc_cntl::sleep::{RtcSleepConfig, TimerWakeupSource}};
use log::{info, trace, error};
use crate::utils::rtc::RtcExt;

esp_bootloader_esp_idf::esp_app_desc!();

struct DebugProcessor;

impl Processor for DebugProcessor {
    async fn awaiting_time_sync(&self, 
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        mac: [u8; 6], 
        controller: trouble_host::prelude::ExternalController<esp_radio::ble::controller::BleConnector<'_>, 20>) -> anyhow::Result<state::DeviceState> {
        
        info!("Awaiting time sync ... ");

        Ok(state::DeviceState::Buffering(Series::new(Measurement::MAX_DEVIATION)))
    }
    
    async fn buffering(&self,
        state: &crate::state::Measurements,
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        gauge: &mut crate::gauge::Gauge<'_, GPIO34<'_>>,
        rng: esp_hal::rng::Rng
    )  -> anyhow::Result<state::DeviceState> {

        info!("Measuring ... {}/6 ", &state.buckets.len());

        let sample = gauge.sample().await?;
        let mut measurements = state.clone();

        measurements.append_monotonic(rtc.now_naivedatetime(), sample);

        let next_state = if(measurements.is_full()) {
            state::DeviceState::Flush(measurements)
        } else {
            state::DeviceState::Buffering(measurements)
        };

        Ok(next_state)
    }
    
    async fn flushing(&self,
        state: &crate::state::Measurements,
        rtc: &esp_hal::rtc_cntl::Rtc<'_>, 
        gauge: &mut crate::gauge::Gauge<'_, GPIO34<'_>>,
        mac: [u8; 6], 
        controller: trouble_host::prelude::ExternalController<esp_radio::ble::controller::BleConnector<'_>, 20>,
        rng: esp_hal::rng::Rng
    )  -> anyhow::Result<state::DeviceState> {

        for m in &state.buckets {
            info!("At {:?} got .. {} % RH, {} lux, {} pF, {} C, {} battery", m.range, m.value.humidity, m.value.lux, m.value.soil_pf, m.value.temperature, m.value.battery);
        }

        embassy_time::Timer::after_millis(150).await;
        
        Ok(state::DeviceState::Buffering(Series::new(Measurement::MAX_DEVIATION)))
    }

    
}

#[esp_rtos::main]
async fn main(_s: Spawner) {
    let state = get_device_state();
    let mut cfg = RtcSleepConfig::deep();
    cfg.set_rtc_fastmem_pd_en(false);
    let wakeup_source = TimerWakeupSource::new(core::time::Duration::from_secs(1));

    match process(state, DebugProcessor {}).await {
        Ok(result) => {
            set_device_state(result.next_state);
            let mut rtc = result.rtc;
            rtc.sleep(&cfg, &[&wakeup_source]);
        },
        Err(err) => {
            error!("Process crashed! {}", err);
        }
    }
}
