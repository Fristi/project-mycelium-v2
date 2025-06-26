use chrono::DateTime;
use current_time::CurrentTime;
use embassy_futures::join::join;
use esp_hal::rtc_cntl::Rtc;
use trouble_host::prelude::*;
use defmt::*;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

/// Time service
#[gatt_service(uuid = service::CURRENT_TIME)]
struct TimeService {
    #[characteristic(uuid = characteristic::CURRENT_TIME, write, read)]
    current_time: [u8; 10]
}

// GATT Server definition
#[gatt_server]
struct Server {
    time_service: TimeService
}

/// Run the BLE stack.
pub async fn run<C>(controller: C, rtc: &mut Rtc<'_>)
where
    C: Controller,
{
    // Using a fixed "random" address can be useful for testing. In real scenarios, one would
    // use e.g. the MAC 6 byte array as the address (how to get that varies by the platform).
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources).set_random_address(address);
    let Host {
        mut peripheral, runner, ..
    } = stack.build();

    info!("Starting advertising and GATT service");
    let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "TrouBLE",
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    }))
    .unwrap();

    let _ = join(ble_task(runner), async {
        loop {
            match advertise("Mycelium", &mut peripheral, &server).await {
                Ok(conn) => {
                    gatt_events_task(&server, &conn, rtc).await.expect("Failed to handle GATT events");
                }
                Err(e) => {
                    defmt::panic!("[adv] error: {:?}", Debug2Format(&e));
                }
            }
        }
    })
    .await;
}

async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            defmt::panic!("[ble_task] error: {:?}", Debug2Format(&e));
        }
    }
}

/// Stream Events until the connection closes.
///
/// This function will handle the GATT events and process them.
/// This is how we interact with read and write requests.
async fn gatt_events_task<P: PacketPool>(server: &Server<'_>, conn: &GattConnection<'_, '_, P>, rtc: &mut Rtc<'_>) -> Result<(), Error> {
    let current_time = server.time_service.current_time;
    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == current_time.handle {
                            let ct = CurrentTime::from_naivedatetime(
                                DateTime::from_timestamp_nanos((rtc.current_time_us() as i64) * 1000)
                                    .naive_local(),
                            );
                            let bytes = ct.to_bytes();
                            server
                                .set(&current_time, &bytes)
                                .expect("Failed to set current time");
                            let value = server.get(&current_time);
                            info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == current_time.handle {
                            let current_time = CurrentTime::from_bytes(event.data());
                            let ts = current_time.to_naivedatetime().expect("Conversion to NaiveDateTime failed").and_utc().timestamp();
                            rtc.set_current_time_us(ts as u64);
                        }
                    }
                    _ => {}
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
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server Server<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[[0x05, 0x18]]),
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