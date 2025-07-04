use core::cell::UnsafeCell;

use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_time::Timer;
use defmt::{info, warn, Debug2Format};
use esp_hal::rtc_cntl::Rtc;
use trouble_host::prelude::*;
use edge_protocol::*;
use heapless::Vec;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

const L2CAP_MTU: usize = 255;

// Thread-local buffer for preparing GATT data
// struct SyncUnsafeCell<T>(UnsafeCell<T>);

// unsafe impl<T> Sync for SyncUnsafeCell<T> {}

// static GATT_BUFFER: SyncUnsafeCell<[u8; 198]> =
//     SyncUnsafeCell(UnsafeCell::new([0; 198]));

// pub struct MeasurementSeries(pub Vec<MeasurementSerieEntry, 6>);

// impl Default for MeasurementSeries {
//     fn default() -> Self {
//         Self(Vec::new())
//     }
// }

// impl AsGatt for MeasurementSeries {
//     fn as_gatt(&self) -> &[u8] {
//         let buffer = unsafe { &mut *GATT_BUFFER.0.get() };

//         for (i, item) in self.0.iter().enumerate() {
//             buffer[i * 33..(i + 1) * 33].copy_from_slice(&item.to_tlv());
//         }

//         unsafe { core::slice::from_raw_parts(buffer.as_ptr(), 198) }

//     }
//     const MIN_SIZE: usize = 198;
//     const MAX_SIZE: usize = 198;
// }

#[gatt_server]
struct Server {
    // measurement_service: MeasurementService,
    time_service: TimeService
}

#[gatt_service(uuid = CURRENT_TIME_SERVICE_UUID)]
struct TimeService {
    #[characteristic(uuid = CURRENT_TIME_CHARACTERISTIC_UUID, write, read)]
    current_time: [u8; 10]
}

// #[gatt_service(uuid = MEASUREMENT_SERVICE_UUID)]
// struct MeasurementService {
//     #[characteristic(uuid = MEASUREMENT_CHARACTERISTIC_UUID, read)]
//     measurement: MeasurementSeries
// }

/// Run the BLE stack.
pub async fn run<C>(controller: C, rtc: &mut Rtc<'_>)
where
    C: Controller,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let mut resources: HostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral, runner, ..
    } = stack.build();

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "TrouBLE",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .expect("Unable to start GATT service");

    info!("Starting adv and event loop");

    let _ = join(ble_task(runner), async {
        
        info!("Advertising...");

        loop {
            match advertise("Trouble Example", &mut peripheral, &server).await {
                Ok(conn) => {
                    info!("Got gatt connection");
                    // set up tasks when the connection is established to a central, so they don't run when no one is connected.
                    let a = gatt_events_task(&server, &conn, rtc);
                    let b = custom_task(&server, &conn, &stack);
                    // run until any task ends (usually because the connection has been closed),
                    // then return to advertising state.
                    select(a, b).await;
                }
                Err(e) => {
                    let e = defmt::Debug2Format(&e);
                    panic!("[adv] error: {:?}", e);
                }
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
async fn gatt_events_task(server: &Server<'_>, conn: &GattConnection<'_, '_>, rtc: &mut Rtc<'_>) -> Result<(), Error> {
    let current_time = server.time_service.current_time;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event: Err(e) } => warn!("[gatt] error processing event: {:?}", e),
            GattConnectionEvent::Gatt { event: Ok(event) } => {
                match &event {
                    GattEvent::Read(event) => {
     

                        if(event.handle() == current_time.handle) {
                            // let ct = CurrentTime::from_naivedatetime(rtc.current_time());

                            let now = rtc.current_time();
                            let ct = CurrentTime::from_naivedatetime(now);
                            let value = ct.to_bytes();
                            current_time.set(&server, &value).expect("Unable to set the time");
                            info!("[gatt] Read Event to curren time Characteristic: {:?}", Debug2Format(&now));
                        }
                    }
                    GattEvent::Write(event) => {


                        if event.handle() == current_time.handle {
                            let bytes = event.data();
                            let ct = CurrentTime::from_bytes(&bytes);
                            info!("[gatt] Write Event to current time Characteristic: {:?}", Debug2Format(&ct));

                            rtc.set_current_time(ct.to_naivedatetime());
                        }
                    }
                };
                // This step is also performed at drop(), but writing it explicitly is necessary
                // in order to ensure reply is sent.
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
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
            AdStructure::ServiceUuids128(&[CURRENT_TIME_SERVICE_UUID, MEASUREMENT_SERVICE_UUID]),
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

/// Example task to use the BLE notifier interface.
/// This task will notify the connected central of a counter value every 2 seconds.
/// It will also read the RSSI value every 2 seconds.
/// and will stop when the connection is closed by the central or an error occurs.
async fn custom_task<C: Controller>(
    server: &Server<'_>,
    conn: &GattConnection<'_, '_>,
    stack: &Stack<'_, C,>
) {
    let current_time = server.time_service.current_time;
    loop {

        // let ct = CurrentTime::from_naivedatetime(rtc.current_time());
        // current_time.set(&server, &ct.to_bytes()).expect("Unable to set the time");
        Timer::after_secs(1).await;
    }
}
