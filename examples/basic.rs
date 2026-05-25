use rathole_sdk::{Client, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Local service configuration using the Builder
    let my_api = Service::builder()
        .name("api_backend")
        .local_addr("127.0.0.1:3000")
        .token("use_a_secret_that_only_you_know")
        .build()?;

    // 2. Configuración del cliente
    let client = Client::builder()
        .remote_addr("127.0.0.1:2333") // using localhost for safe run
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
