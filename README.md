mycelium
---

![mycelium logo](/logo-mycelium.jpg)

The mycelium project is too monitor and water plants automatically in house holds and gardens by using the following components:

- **edge-peripheral** - A custom PCB designed for low energy use, using deep sleep and optimized protocols. We utilize deep sleep and Bluetooth Low Energy (BLE) to achieve that. 
- **edge-central** - This device also resides at the edge and will scan continously for peripherals to collect measurements, sync time and schedule commands to the peripherals when they need to perform an action.
- **edge-protocol** - This crate is the bridge between peripheral and central, to talk the same protocol in between


