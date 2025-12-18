use trouble_host::prelude::*;
use edge_protocol::{proto::{Events, MacAddress, PlantProfile, SyncState, Timestamp}, v2::*};

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

const L2CAP_MTU: usize = 512;


#[gatt_service(uuid = STATION_SERVICE_UUID_16)]
struct StationService {
    #[characteristic(uuid = STATION_MAC_ADDR_CHARACTERISTIC_UUID_16, read)]
    address: MacAddress,
    #[characteristic(uuid = STATION_EVENTS_CHARACTERISTIC_UUID_16, read)]
    events: Events,
    #[characteristic(uuid = STATION_PLANT_PROFILE_CHARACTERISTIC_UUID_16, read, write)]
    current_profile: PlantProfile,
    #[characteristic(uuid = STATION_CURRENT_TIME_CHARACTERISTIC_UUID_16, read, write)]
    current_time: Timestamp,
    #[characteristic(uuid = STATION_SYNC_STATE_CHARACTERISTIC_UUID_16, read, write)]
    sync_state: SyncState
}


#[gatt_server]
struct Server {
    station_service: StationService
}