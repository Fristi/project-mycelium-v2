use heapless::Vec;
use edge_protocol::{Measurement};

pub enum DeviceState {
    AwaitingTimeSync,
    Buffering(Vec<Measurement, 10>),
}