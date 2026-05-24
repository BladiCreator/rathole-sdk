use rathole_sdk::{Client, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Local service configuration using the Builder
    let my_api = Service::builder()
        .name("api_backend")
        .addr("127.0.0.1:3000")
        .token("secure_token_123")
        .build()?;

    // 2. Client configuration
    let client = Client::builder()
        .addr("127.0.0.1:2333") // using localhost for safe run
        .add_service(my_api)
        .build()?;

    // 3. Start in the background (Non-blocking)
    let tunnel_handle = client.spawn_background().await;

    // --- Host application logic runs here ---
    println!("Tunnel is operating in the background. Current status: {:?}", tunnel_handle.status());
    
    // 4. Graceful shutdown
    tunnel_handle.stop();
    println!("Shutdown signal sent.");

    Ok(())
}
