# Mycelium Desktop App

Cross-platform desktop application for the Mycelium plant monitoring system, built with Tauri (Rust backend + React frontend).

## Overview

The desktop app provides a user-friendly interface for:
- Monitoring plant stations and their sensor readings
- Viewing historical measurement data with interactive charts
- Managing plant configurations and watering schedules
- Auth0 authentication and cloud synchronization
- Real-time data visualization (temperature, humidity, light, soil moisture, battery levels)

## Technology Stack

- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Tauri 2.0 (Rust)
- **UI Framework**: Tailwind CSS + Headless UI
- **Charts**: Recharts for data visualization
- **Authentication**: Auth0 React SDK
- **HTTP Client**: Axios with React Query
- **Forms**: Formik with Zod validation
- **Build System**: Vite for fast development and optimized builds

## Development Setup

### Prerequisites

- Node.js 20+
- Rust toolchain (1.88+)
- Platform-specific dependencies for Tauri

### Installation

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Build Tauri app
npm run tauri build
```

### Development Commands

- `npm run dev` - Start Vite development server
- `npm run build` - Build frontend for production
- `npm run preview` - Preview production build
- `npm run tauri dev` - Start Tauri development mode
- `npm run tauri build` - Build production Tauri app

## Architecture

The app follows Tauri's architecture pattern:
- **Frontend**: React SPA running in a webview with TypeScript
- **Backend**: Tauri Rust application providing system APIs
- **API Integration**: REST API communication with Scala backend
- **Authentication**: Auth0 integration for secure access
- **State Management**: React Query for server state, React hooks for local state
- **Routing**: React Router with hash-based routing for Tauri compatibility

## Key Features

### Plant Management
- Add new plant stations with BLE device provisioning
- Edit plant details (name, location, description)
- Configure watering schedules (interval-based or threshold-based)
- Delete plant stations

### Data Visualization
- Interactive area charts for sensor data
- Historical data with configurable time periods
- Real-time updates from backend API
- Battery level monitoring

### Authentication & Security
- Auth0 integration with JWT tokens
- Secure API communication
- User-specific data isolation

### Device Integration
- BLE device onboarding workflow
- WiFi configuration for edge devices
- Device state monitoring

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Configuration

### Environment Variables

The app uses environment variables for configuration:

```bash
# API Base URL (defaults to localhost:8080 in development)
VITE_API_BASE_URL=http://localhost:8080

# Auth0 Configuration (hardcoded in main.tsx for now)
# Domain: mycelium-greens.eu.auth0.com
# Client ID: TTqNjNFpS7J158xPzznXSAMK302F6Amc
```

### Build Configuration

The app is configured through:
- `src-tauri/tauri.conf.json` - Tauri-specific configuration
- `vite.config.ts` - Vite build configuration  
- `tsconfig.json` - TypeScript configuration
- `tailwind.config.js` - Tailwind CSS configuration
- `postcss.config.js` - PostCSS configuration

### Dependencies

Key dependencies include:
- **@auth0/auth0-react**: Authentication
- **@tanstack/react-query**: Server state management
- **axios**: HTTP client
- **formik**: Form handling
- **zod**: Schema validation
- **recharts**: Data visualization
- **@headlessui/react**: Accessible UI components
- **@heroicons/react**: Icon library
- **moment**: Date/time handling
