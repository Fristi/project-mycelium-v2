# Mycelium Desktop App

Cross-platform desktop application for the Mycelium plant monitoring system, built with Tauri (Rust backend + React frontend).

## Overview

The desktop app provides a user-friendly interface for:
- Monitoring connected edge devices and their sensor readings
- Viewing historical measurement data
- Managing system configuration and authentication
- Controlling peripheral device synchronization

## Technology Stack

- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Tauri 2.0 (Rust)
- **UI Framework**: React with modern hooks
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
- **Frontend**: React SPA running in a webview
- **Backend**: Rust application providing system APIs
- **IPC**: Communication between frontend and backend via Tauri's invoke system

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Build Configuration

The app is configured through:
- `src-tauri/tauri.conf.json` - Tauri-specific configuration
- `vite.config.ts` - Vite build configuration
- `tsconfig.json` - TypeScript configuration
