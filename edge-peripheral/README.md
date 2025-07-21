edge-peripheral
---

ESP32-based sensor device for the mycelium plant monitoring system. This firmware implements a low-power IoT sensor that collects environmental measurements and communicates via Bluetooth Low Energy.

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

1. **AwaitingTimeSync**: Initial state waiting for time synchronization from central
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
- Rust with ESP32 target support
- ESP-IDF development environment
- USB-to-serial programmer

### Building and Flashing
```bash
# Build the firmware
cargo build --release

# Flash to device
cargo run
```

### Programmer Selection
When flashing, select the appropriate programmer:
```
/dev/tty.usbserial-11103 - Quad RS232-HS
```

### Configuration
The device uses compile-time configuration through Cargo.toml features and environment variables. Key settings include:

- **Sleep Duration**: Configurable wake interval
- **Buffer Size**: Maximum measurements before flush (currently 6)
- **Sensor Pins**: I2C and ADC pin assignments
- **BLE Services**: Custom UUID definitions in edge-protocol

### Dependencies
- `esp-hal`: ESP32 hardware abstraction layer
- `embassy-executor`: Async runtime for embedded systems
- `bt-hci` & `trouble-host`: Bluetooth Low Energy stack
- `edge-protocol`: Shared protocol definitions
- `timeseries`: Data compression and buffering
- Custom sensor drivers: `shtcx`, `bh1730fvc`

## Troubleshooting

### Common Issues
- **Flash Errors**: Ensure correct programmer selection and ESP32 is in download mode
- **I2C Failures**: Check sensor connections and power supply
- **BLE Connection**: Verify central device is scanning and in range
- **Deep Sleep**: RTC fast memory corruption may require full reset

### Debug Output
The firmware uses `defmt` for structured logging. Connect a serial monitor to view debug output during development.

