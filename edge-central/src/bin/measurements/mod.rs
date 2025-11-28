use chrono::TimeDelta;
use crate::cfg::PeripheralSyncMode;
use crate::measurements::random::RandomPeripheralSyncResultStreamProvider;
use crate::measurements::types::PeripheralSyncResultStreamProvider;

pub mod random;
pub mod types;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod trouble;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod btleplug;

pub async fn make_peripheral_sync_stream_provider(
    mode: &PeripheralSyncMode,
) -> anyhow::Result<Box<dyn PeripheralSyncResultStreamProvider>> {
    match mode {
        PeripheralSyncMode::Ble => {
            {
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                {
                    let provider = btleplug::BtleplugPeripheralSyncResultStreamProvider::new().await?;

                    anyhow::Ok(Box::new(provider))
                }

                #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
                {
                    todo!("Not implemented")
                }
            }

        }
        PeripheralSyncMode::Random => {
            let provider = RandomPeripheralSyncResultStreamProvider::new(
                [0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa],
                TimeDelta::seconds(2),
            );

            anyhow::Ok(Box::new(provider))
        }
    }
}