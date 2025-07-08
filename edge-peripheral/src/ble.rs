use core::cell::UnsafeCell;
use defmt::{info, warn, error, Debug2Format};
use embassy_futures::select::select;
use embassy_time::{Delay, Duration, Timer};
use esp_hal::rtc_cntl::Rtc;
use trouble_host::prelude::*;
use trouble_host::types::gatt_traits::FromGattError;

use edge_protocol::*;
use heapless::Vec;

pub type MeasurementSerieEntryVec = Vec<MeasurementSerieEntry, 6>;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

const L2CAP_MTU: usize = 255;

// Thread-local buffer for preparing GATT data
struct SyncUnsafeCell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for SyncUnsafeCell<T> {}

static GATT_BUFFER: SyncUnsafeCell<[u8; 198]> =
    SyncUnsafeCell(UnsafeCell::new([0; 198]));


pub struct MeasurementSeries(MeasurementSerieEntryVec);


impl Default for MeasurementSeries {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl AsGatt for MeasurementSeries {
    fn as_gatt(&self) -> &[u8] {
        let buffer = unsafe { &mut *GATT_BUFFER.0.get() };

        for (i, item) in self.0.iter().enumerate() {
            buffer[i * 33..(i + 1) * 33].copy_from_slice(&item.to_tlv());
        }

        unsafe { core::slice::from_raw_parts(buffer.as_ptr(), 198) }

    }
    const MIN_SIZE: usize = 198;
    const MAX_SIZE: usize = 198;
}

impl FromGatt for MeasurementSeries {
    fn from_gatt(data: &[u8]) -> Result<Self, FromGattError> {
        let mut measurements = Vec::<MeasurementSerieEntry, 6>::new();
        for i in 0..6 {
            let start = i * 33;
            let end = start + 33;
            let segment = &data[start..end];

            let entry = MeasurementSerieEntry::from_tlv(segment).expect("Unable to decode measurement entry");  
            if measurements.push(entry).is_err() {
                break;
            }
        }

        Ok(MeasurementSeries(measurements))
    }
}

#[gatt_server]
struct Server {
    address_service: AddressService,
    time_service: TimeService,
    measurement_service: MeasurementService
}

#[gatt_service(uuid = ADDRESS_SERVICE_UUID_16)]
struct AddressService {
    #[characteristic(uuid = ADDRESS_CHARACTERISTIC_UUID_16, read)]
    address: [u8; 6]
}

/// Time service
#[gatt_service(uuid = BluetoothUuid16::new(CURRENT_TIME_SERVICE_UUID))]
struct TimeService {
    #[characteristic(uuid = BluetoothUuid16::new(CURRENT_TIME_CHARACTERISTIC_UUID), write, read)]
    current_time: [u8; 10]
}


#[gatt_service(uuid = MEASUREMENT_SERVICE_UUID_16)]
struct MeasurementService {
    #[characteristic(uuid = MEASUREMENT_CHARACTERISTIC_UUID_16, read)]
    measurement: MeasurementSeries
}

/// Run the BLE stack.
pub async fn run<C>(controller: C, rtc: &mut Rtc<'_>, address: [u8; 6], measurements: MeasurementSerieEntryVec)
where
    C: Controller,
{

    let mut resources: HostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(Address::random(address));
    let Host {
        mut peripheral, runner, ..
    } = stack.build();

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Mycelium",
        appearance: &appearance::UNKNOWN,
    }))
    .expect("Unable to start GATT service");

    info!("Starting adv and event loop");


    
    let _ = select(ble_task(runner), async {
        
        info!("Advertising...");

        match advertise("Mycelium", &mut peripheral, &server).await {
            Ok(conn) => {
                info!("Got gatt connection");
                match gatt_events_task(&server, &conn, rtc, address, measurements.clone()).await {
                    Ok(_) => (),
                    Err(e) => {
                        let e = defmt::Debug2Format(&e);
                        error!("[adv] error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                let e = defmt::Debug2Format(&e);
                panic!("[adv] error: {:?}", e);
            }
        }

        
    })
    .await;
}


async fn ble_task<C: Controller>(mut runner: Runner<'_, C>) {
    loop {
        if let Err(e) = runner.run().await {
            let e = defmt::Debug2Format(&e);
            panic!("[ble_task] error: {:?}", e);
        }
    }
}

// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
async fn gatt_events_task(server: &Server<'_>, conn: &GattConnection<'_, '_>, rtc: &mut Rtc<'_>, address: [u8; 6], measurements: MeasurementSerieEntryVec) -> Result<(), Error> {

    let series = MeasurementSeries(measurements.clone());
    server.measurement_service.measurement.set(&server, &series).expect("Unable to set measurement data");
    server.address_service.address.set(&server, &address).expect("Unable to set address");

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Gatt { event: Err(e) } => warn!("[gatt] error processing event: {:?}", e),
            GattConnectionEvent::Gatt { event: Ok(event_) } => {
                
                match &event_ {
                    GattEvent::Read(event) => {
                        if event.handle() == server.measurement_service.measurement.handle {
                            info!("[gatt] Read Event to measurement Characteristic");
                            match event_.accept() {
                                Ok(reply) => {
                                    reply.send().await;
                                    Timer::after_millis(300).await;
                                    break
                                },
                                Err(e) => warn!("[gatt] error sending response: {:?}", e),
                            };
                        } else if event.handle() == server.time_service.current_time.handle {
                            info!("[gatt] Read Event to current time Characteristic");
                            let now = rtc.current_time();
                            let ct = CurrentTime::from_naivedatetime(now);
                            let value = ct.to_bytes();
                            server.time_service.current_time.set(&server, &value).expect("Unable to set the time");
                            match event_.accept() {
                                Ok(reply) => reply.send().await,
                                Err(e) => warn!("[gatt] error sending response: {:?}", e),
                            }
                        } else {
                            match event_.accept() {
                                Ok(reply) => reply.send().await,
                                Err(e) => warn!("[gatt] error sending response: {:?}", e),
                            }
                        }

                        
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == server.time_service.current_time.handle {
                            let bytes = event.data();
                            let ct = CurrentTime::from_bytes(&bytes);
                             info!("[gatt] Write Event to current time Characteristic: {:?}", Debug2Format(&ct));

                            rtc.set_current_time(ct.to_naivedatetime());
                            match event_.accept() {
                                Ok(reply) => reply.send().await,
                                Err(e) => warn!("[gatt] error sending response: {:?}", e),
                            };
                        }
                    }
                };
            }
            _ => {} // ignore other Gatt Connection Events
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}


/// Create an advertiser to use to connect to a BLE Central, and wait for it to connect.
async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[
                CURRENT_TIME_SERVICE_UUID.to_le_bytes(), 
                MEASUREMENT_SERVICE_UUID_16.to_le_bytes()
            ]),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}