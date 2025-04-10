# iot-driver

This crate provides a generic framework for building robust and scalable IoT drivers for interacting with fieldbus devices. It leverages the power of asynchronous programming with Tokio, enabling efficient and concurrent communication.  At its core, the framework features a `Monitor` that continuously polls data points and seamlessly handles configuration updates.

## Key Features

* **Asynchronous Operations:** Built on Tokio, the framework utilizes asynchronous programming for highly efficient and concurrent communication with fieldbus devices.
* **Dynamic Configuration Updates:** The `Subscriber` and `Configuration` traits facilitate dynamic configuration updates during runtime, allowing the driver to adapt to changing conditions.
* **Connection Pooling:**  The `Fieldbus` and `ConnectionPool` traits provide connection pooling, optimizing resource utilization and improving performance.
* **Generic and Extensible Design:** The trait-based design promotes flexibility and customization, enabling support for various fieldbus protocols (e.g., Modbus, Profibus) and diverse data types.
* **Testability:** Traits enable easy mocking and unit testing of individual components, ensuring code reliability and maintainability.

## Architecture

The framework revolves around several key traits:

* **`Config`**: Defines the associated types for a specific driver implementation, such as the subscriber, configuration, connection pool, fieldbus, data point, and fieldbus data types.  Serves as the central configuration point for the driver.
* **`Subscriber<T: Config>`**: Represents the subscription mechanism for receiving configuration updates. The `subscribe` method returns a future that resolves to an updated subscriber instance.
* **`Configuration<T: Config>`**: Manages the driver's configuration. It employs a `Subscriber` to receive updates and provides access to defined data points.
* **`Datapoint`**: Represents a single data point on the fieldbus, typically identified by a topic and URL.
* **`FieldbusData`**: Represents the data read from the fieldbus. This trait allows for custom data structures based on the fieldbus being used.
* **`Monitor<T: Config>`**: The core component responsible for polling data points, handling configuration updates, and processing the retrieved data. It orchestrates the interaction between the other components.
* **`Fieldbus<T: Config>`**:  Represents the interface to the physical fieldbus. It manages connection pools and provides the `read` method for retrieving data from specific data points.
* **`Connection`**: Represents a single connection to a fieldbus device.
* **`ConnectionPool<T: Config>`**:  Manages a pool of connections to a fieldbus device, enabling efficient connection reuse and management.
* **`GenericDriver<T: Config>`**: Provides a default `run` implementation that sets up and starts the `Monitor`, simplifying the process of creating a new driver.


## Usage Example (with Mocks)

```rust
use iot_driver::{GenericDriver, mocks::MockConfig};

#[tokio::main]
async fn main() {
    MockConfig::run().await;
}
```

## Example Implementation and Testing (Mocks)

The provided `mocks` module demonstrates a simplified implementation of the core traits using mock objects.  This example serves as a starting point for developing custom drivers and provides a clear illustration of how the components interact.

The `tests` module further showcases how to effectively test a `Monitor` implementation using the mock objects. These tests verify the correct interaction between the monitor, configuration, and fieldbus, ensuring that data is polled, processed, and configuration updates are handled as expected.  This robust testing framework promotes confidence in the reliability of your custom driver implementations.

## Crate Descriptions

### iot-driver
This crate provides a generic framework for interacting with fieldbus devices. It uses asynchronous programming with Tokio for efficient communication. The core of the framework is a `Monitor` that polls data points and handles configuration updates.

### iot-bridge

This crate acts as a bridge, connecting different parts of the IoT system. It facilitates communication and data transfer between various components.

### iot-dds

This crate provides Data Distribution Service (DDS) integration for the IoT platform. It allows for real-time data exchange between devices and applications using DDS.

### iot-driver-modbus

This crate provides a Modbus implementation of the `iot-driver` framework.  It enables communication with Modbus devices over TCP.

### iot-graphql

This crate provides GraphQL integration for the IoT platform.  It enables querying and managing IoT devices using GraphQL.

### iot-motion

This crate provides motion control functionalities for IoT devices. It handles motion-related operations.

### iot-tcp-connpool

This crate provides a connection pool specifically designed for Modbus TCP connections using bb8.  It's used by iot-driver-modbus to maintain and reuse connections efficiently.


## Future Enhancements

* Comprehensive error handling and retry mechanisms for improved resilience.
* More advanced connection management strategies to optimize performance and resource usage in various scenarios.
* Expanded support for different fieldbus protocols, including Modbus, Profibus, and others, broadening the applicability of the framework.
