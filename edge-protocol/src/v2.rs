use crate::proto::*;
use trouble_host::types::gatt_traits::{FromGatt, AsGatt, FromGattError};
use micropb::*;
use core::cell::UnsafeCell;

const MAX_BUFFER_SIZE: usize = 512;

struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SyncUnsafeCell<T> {}

static GATT_BUFFER: SyncUnsafeCell<[u8; MAX_BUFFER_SIZE]> =
    SyncUnsafeCell(UnsafeCell::new([0; MAX_BUFFER_SIZE]));

macro_rules! as_gatt {
    ($($t:ty),*) => {
        $(
            impl AsGatt for $t
            where
                $t: micropb::MessageEncode
            {
                const MIN_SIZE: usize = match <$t as micropb::MessageEncode>::MAX_SIZE {
                    Some(n) => n,
                    None => MAX_BUFFER_SIZE
                };
                const MAX_SIZE: usize = match <$t as micropb::MessageEncode>::MAX_SIZE {
                    Some(n) => n,
                    None => MAX_BUFFER_SIZE
                };
                
                fn as_gatt(&self) -> &'static [u8] {
                    let buffer: &mut [u8; MAX_BUFFER_SIZE] = unsafe { &mut *GATT_BUFFER.0.get() };
                    let ptr = buffer.as_ptr();
                    let mut writer = &mut buffer[..];
                    let mut encoder = micropb::PbEncoder::new(&mut writer);

                    self.encode(&mut encoder).expect("Encoding failed...");
                    
                    unsafe { core::slice::from_raw_parts(ptr, <$t as AsGatt>::MAX_SIZE) }
                }
            }
        )*
    };
}

macro_rules! from_gatt {
    ($($t:ty),*) => {
        $(
            impl FromGatt for $t
            where
                $t: micropb::MessageDecode + Default,
            {
                fn from_gatt(data: &[u8]) -> Result<Self, FromGattError> {
                    let mut message = Self::default();
                    message.decode_from_bytes(data).expect("Unable to decode");
                    Ok(message)
                }
            }
        )*
    };
}

pub const STATION_SERVICE_UUID_16: u16 = 0xFFFF;
pub const STATION_MAC_ADDR_CHARACTERISTIC_UUID_16: u16 =  0xFFF0;
pub const STATION_PLANT_PROFILE_CHARACTERISTIC_UUID_16: u16 =  0xFFF1;
pub const STATION_EVENTS_CHARACTERISTIC_UUID_16: u16 =  0xFFF2;
pub const STATION_CURRENT_TIME_CHARACTERISTIC_UUID_16: u16 =  0xFFF3;
pub const STATION_SYNC_STATE_CHARACTERISTIC_UUID_16: u16 =  0xFFF3;


impl AsGatt for SyncState {
    const MIN_SIZE: usize = 1;
    const MAX_SIZE: usize = 1;

    fn as_gatt(&self) -> &[u8] {
        let buffer: &mut [u8; MAX_BUFFER_SIZE] = unsafe { &mut *GATT_BUFFER.0.get() };
        let ptr = buffer.as_ptr();
        buffer[0] = self.0 as u8;
        
        unsafe { core::slice::from_raw_parts(ptr, SyncState::MAX_SIZE) }
    }
}

impl FromGatt for SyncState {
    fn from_gatt(data: &[u8]) -> Result<Self, FromGattError> {
        Ok(SyncState::from(data[0] as i8))
    }
}

as_gatt!(
    PlantProfile,
    Timestamp,
    Events,
    MacAddress
);

from_gatt!(
    PlantProfile,
    Timestamp,
    Events,
    MacAddress
);