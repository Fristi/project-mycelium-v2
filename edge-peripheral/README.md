## edge-peripheral

ESP32-based sensor device for the mycelium plant monitoring system. This
firmware implements a low-power IoT sensor that collects environmental
measurements and communicates via Bluetooth Low Energy.

## Hardware Requirements

- ESP32 microcontroller
- SHTC3 temperature/humidity sensor (I2C)
- BH1730FVC ambient light sensor (I2C)
- Battery monitoring circuit (ADC)
- Custom PCB with power management

## Features

### Sensor Integration

- **Temperature & Humidity**: SHTC3 sensor via I2C
- **Ambient Light**: BH1730FVC sensor for lux measurements
- **Battery Level**: ADC-based battery voltage monitoring
- **MAC Address**: Unique device identification using ESP32 efuse

### Power Management

- **Deep Sleep**: Configurable sleep intervals (default: 10 seconds)
- **RTC Fast Memory**: State persistence across sleep cycles
- **Power Control**: GPIO-controlled sensor power management
- **Wake Sources**: Timer-based wake from deep sleep

### Communication Protocol

- **BLE GATT Services**: Custom services for data exchange
- **Time Synchronization**: BLE Current Time Service support
- **Data Buffering**: Local time series storage with compression
- **Batch Transmission**: Efficient data upload when buffer is full

### Device States

The device operates in three main states:

1. **AwaitingTimeSync**: Initial state waiting for time synchronization from
   central
2. **Buffering**: Collecting and storing measurements locally (up to 6 samples)
3. **Flush**: Transmitting buffered data to central device via BLE

### Data Flow

1. Device wakes from deep sleep
2. Samples sensors (temperature, humidity, light, battery)
3. Stores measurement in time series buffer with deviation filtering
4. When buffer is full, switches to flush mode
5. Establishes BLE connection and transmits data
6. Returns to buffering mode and enters deep sleep

## Development

### Prerequisites

- Rust 1.88+ with ESP32 target support
- ESP toolchain installed via `espup`
- USB-to-serial programmer

### Setup ESP Environment

```bash
# Install espup and ESP toolchain
cargo install espup --locked
espup install

# Source the ESP environment (add to your shell profile)
source ~/export-esp.sh
```

### Building and Flashing

```bash
# Flash to device
cargo run --release
```

### Dagger Build (Alternative)

```bash
# Build using Dagger (containerized)
dagger call build-peripheral
```

### Programmer Selection

When flashing, select the appropriate programmer:

```
/dev/tty.usbserial-11103 - Quad RS232-HS
```

### Configuration

The device uses compile-time configuration through Cargo.toml features and
environment variables. Key settings include:

- **Sleep Duration**: Configurable wake interval
- **Buffer Size**: Maximum measurements before flush (currently 6)
- **Sensor Pins**: I2C and ADC pin assignments
- **BLE Services**: Custom UUID definitions in edge-protocol

## Troubleshooting

### Common Issues

- **Flash Errors**: Ensure batteries have enough energy and the cables are connected
- **BLE Connection**: Verify central device is scanning and in range

### Debug Output

The firmware uses `defmt` for structured logging. Connect a serial monitor to
view debug output during development.
