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

pub fn get() -> &'static DeviceState {
    return unsafe { &STATE };
}

pub fn set(state: DeviceState) {
    unsafe {
        STATE = state;
    }
}