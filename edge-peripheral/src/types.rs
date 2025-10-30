use chrono::NaiveDateTime;
use edge_protocol::{Measurement};
use timeseries::Series;

const NR_ENTRIES: usize = 4 * 6;

pub type Measurements = Series<NR_ENTRIES, NaiveDateTime, Measurement>;

pub enum DeviceState {
    AwaitingTimeSync,
    Buffering(Measurements),
    Flush(Measurements)
}
