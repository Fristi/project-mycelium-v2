use anyhow::Ok;
use chrono::Duration;
use futures::{stream, StreamExt};

use crate::measurements::types::PeripheralSyncResultStreamProvider;

pub mod measurements;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = measurements::random::RandomPeripheralSyncResultStreamProvider { 
        mac: [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa], 
        delay: Duration::milliseconds(2000)
    };
    let stream = provider.stream().flat_map(stream::iter);

    stream.for_each(|item| async move {
        println!("Sync result: {:?} got {} measurements", item.address, item.measurements.len());
    }).await;

    Ok(())
}