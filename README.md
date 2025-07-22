## mycelium

![mycelium logo](/logo-mycelium.jpg)

The mycelium project is a plant monitoring system that automatically monitors
environmental conditions in households and gardens using IoT edge devices. The
system consists of three main components:

## Architecture

- **edge-peripheral** - ESP32-based sensor device that collects environmental
  measurements (temperature, humidity, light, battery level) using deep sleep
  and Bluetooth Low Energy (BLE) for power optimization
- **edge-central** - Central hub that continuously scans for peripheral devices,
  collects measurements, manages time synchronization, and handles data
  persistence with cloud integration
- **edge-protocol** - Shared protocol library that defines the communication
  format between peripheral and central devices using TLV (Type-Length-Value)
  encoding

## Features

### Environmental Monitoring

- **Temperature & Humidity**: SHTC3 sensor integration
- **Light Measurement**: BH1730FVC ambient light sensor
- **Battery Monitoring**: ADC-based battery level tracking
- **Time Series Data**: Efficient data compression using deviation-based
  filtering

### Connectivity & Power Management

- **BLE Communication**: Custom GATT services for data exchange
- **Deep Sleep**: Optimized power consumption with configurable wake intervals
- **Time Synchronization**: BLE Current Time Service implementation
- **Data Buffering**: Local measurement storage with batch transmission

### Data Management

- **SQLite Database**: Local data persistence with WAL mode
- **Measurement Storage**: Timestamped sensor readings with MAC address tracking
- **Edge State Management**: WiFi credentials and Auth0 token storage
- **Database Migrations**: Automated schema updates

### Configuration & Deployment

- **Environment-based Config**: Support for BLE and local onboarding strategies
- **Auth0 Integration**: OAuth2 authentication for cloud services
- **WiFi Management**: Configurable network connectivity
- **Multiple Sync Modes**: BLE scanning or random data generation for testing
- **Onboarding Strategies**: Support for local and BLE onboarding methods

## Database Schema

### Measurements Table

```sql
CREATE TABLE measurements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mac BLOB NOT NULL,           -- 6-byte device MAC address
    timestamp DATETIME NOT NULL,  -- Measurement timestamp
    battery INTEGER NOT NULL,     -- Battery level (0-100)
    lux REAL NOT NULL,           -- Light intensity in lux
    temperature REAL NOT NULL,    -- Temperature in Celsius
    humidity REAL NOT NULL        -- Relative humidity (0-100%)
);
```

### Edge State Table

```sql
CREATE TABLE edge_state (
    id INTEGER PRIMARY KEY,
    wifi_ssid TEXT NOT NULL,
    wifi_password TEXT NOT NULL,
    auth0_access_token TEXT NOT NULL,
    auth0_refresh_token TEXT NOT NULL,
    auth0_expires_at DATETIME NOT NULL
);
```

## Getting Started

### Prerequisites

- Rust toolchain with embedded targets
- ESP32 development environment
- SQLite for central device storage
- Auth0 account for authentication
- dotenv for environment configuration

### Configuration

Copy `edge-central/.env.sample` to `edge-central/.env` and configure:

```bash
# New configuration format uses APP prefix
APP.DATABASE_URL=sqlite://mycelium.db
APP.PERIPHERAL_SYNC_MODE=ble  # or 'random' for testing
APP.ONBOARDING_STRATEGY=local  # or 'ble'
APP.AUTH0.DOMAIN=your-domain.auth0.com
APP.AUTH0.CLIENT_ID=your-client-id
APP.AUTH0.CLIENT_SECRET=your-client-secret
APP.WIFI.SSID=your-wifi-ssid
APP.WIFI.PASSWORD=your-wifi-password
```

Note: The environment variable format has been updated with an `APP` prefix and
hierarchical structure.

### Auth0 Configuration

The project uses Auth0 for authentication. Default values for Auth0 domain and
client ID are provided in the code, but you should configure your own Auth0
application for production use:

1. Create an Auth0 application in your Auth0 dashboard
2. Configure the application for Device Authorization Flow
3. Set the appropriate audience and scopes
4. Update the `.env` file with your Auth0 credentials

The application uses the dotenv crate to load environment variables from the
`.env` file at runtime.

### Database Configuration

The application uses SQLite with Write-Ahead Logging (WAL) mode for improved
performance and concurrency. The database file is created automatically if it
doesn't exist, and migrations are applied at startup.

To initialize or update the database schema:

```bash
# The application will run migrations automatically at startup
# No manual migration commands needed
```

The database schema includes tables for measurements and edge state as described
in the Database Schema section.

## Development Notes

### Known Issues

- There are some unused imports in the main.rs file that should be cleaned up:
  - `config::Config` is imported but not used
  - The `repo` variable is declared but not used

### Security Considerations

- The Auth0 client secret should be properly secured and not committed to
  version control
- WiFi credentials in the .env file should be protected
- Consider using environment variables for sensitive information in production
  deployments
