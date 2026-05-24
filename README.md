# rathole-sdk

A secure, stable, and high-performance Rust SDK for NAT traversal and reverse proxying, derived from the core of the [rathole](https://github.com/rapiz1/rathole) project.

`rathole-sdk` provides an ergonomic, developer-friendly API that abstracts the complexity of the `rathole` tunneling tool. It enables developers to integrate secure network capabilities (relays) directly into their applications without depending on external binaries, CLIs, or static TOML configuration files.

## Features

- **Ergonomic Builder API**: Configure your client and services programmatically with `ClientBuilder` and `ServiceBuilder`, providing compile-time validation.
- **Asynchronous & Non-Blocking**: The network engine runs in the background, making it perfect for embedding into async web servers (like Axum, Actix, or any Tokio-based app) without blocking the main thread.
- **Minimalist (Dieting)**: Stripped of the original `rathole` server-side code, CLI interface, and configuration watchers. This SDK contains only what is necessary for a client to establish tunnels.
- **Lifecycle Management**: Easy control over the tunnel's lifecycle using `TunnelHandle`, enabling you to start, stop, and monitor the connection status at runtime.

## Quickstart

Add `rathole-sdk` to your `Cargo.toml`:

```toml
[dependencies]
rathole-sdk = { path = "path/to/rathole-sdk" }
tokio = { version = "1", features = ["full"] }
```

### Basic Example

Here's how you can create and run a client in your application:

```rust
use rathole_sdk::{Client, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configure the local service using the Builder pattern
    let my_api = Service::builder()
        .name("api_backend")
        .local_addr("127.0.0.1:3000")
        .token("use_a_secret_that_only_you_know")
        .build()?;

    // 2. Configure the client
    let client = Client::builder()
        .remote_addr("myserver.com:2333") // Address of the rathole server
        .add_service(my_api)
        .build()?;

    // 3. Start in the background (Non-blocking)
    let tunnel_handle = client.spawn_background().await;

    // --- Your main application runs here ---
    println!("Tunnel is operating in the background. Current status: {:?}", tunnel_handle.status());
    
    // 4. Safe shutdown
    tunnel_handle.stop();
    println!("Shutdown signal sent.");

    Ok(())
}
```

## Architecture

- `Service`: Represents a local service you want to expose. Created via `ServiceBuilder`.
- `Client`: The main client that coordinates tunnels. Created via `ClientBuilder`.
- `TunnelHandle`: Returned when spawning a client. Allows you to check the connection status (`tunnel_handle.status()`) and stop the background tasks gracefully (`tunnel_handle.stop()`).

## License

This project is licensed under the same terms as the original `rathole` project.
