use chrono::NaiveDateTime;
use edge_protocol::{Measurement};
use timeseries::Series;

pub enum DeviceState {
    AwaitingTimeSync,
    Buffering(Series<6, NaiveDateTime, Measurement>),
    Flush(Series<6, NaiveDateTime, Measurement>)
}
