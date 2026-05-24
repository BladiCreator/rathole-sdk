use rathole_sdk::{Client, Service, TunnelStatus};
use rathole_sdk::protocol::{Hello, Ack, Auth, CURRENT_PROTO_VERSION};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_sdk_lifecycle() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    let server_task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // 1. Read hello
        let mut hello_buf = vec![0u8; 37];
        socket.read_exact(&mut hello_buf).await.unwrap();
        let _hello: Hello = bincode::deserialize(&hello_buf).unwrap();
        
        // 2. Write hello response
        let resp = Hello::ControlChannelHello(CURRENT_PROTO_VERSION, [0u8; 32]);
        let resp_bytes = bincode::serialize(&resp).unwrap();
        socket.write_all(&resp_bytes).await.unwrap();

        // 3. Read auth
        let mut auth_buf = vec![0u8; 32];
        socket.read_exact(&mut auth_buf).await.unwrap();
        let _auth: Auth = bincode::deserialize(&auth_buf).unwrap();

        // 4. Write Ack::Ok
        let ack_bytes = bincode::serialize(&Ack::Ok).unwrap();
        socket.write_all(&ack_bytes).await.unwrap();

        // Hold connection open
        let mut tmp = [0u8; 10];
        let _ = socket.read(&mut tmp).await;
    });

    let service = Service::builder()
        .name("test_svc")
        .local_addr("127.0.0.1:8080")
        .token("test_token")
        .build()
        .unwrap();

    let client = Client::builder()
        .remote_addr(&local_addr.to_string())
        .add_service(service)
        .build()
        .unwrap();

    let handle = client.spawn_background().await;

    // Check status becomes Connected
    let mut connected = false;
    for _ in 0..50 {
        if handle.status() == TunnelStatus::Connected {
            connected = true;
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }
    assert!(connected, "Tunnel did not transition to Connected status");

    // Stop tunnel
    handle.stop();

    // Check status becomes Disconnected
    let mut disconnected = false;
    for _ in 0..50 {
        if handle.status() == TunnelStatus::Disconnected {
            disconnected = true;
            break;
        }
        sleep(Duration::from_millis(50)).await;
    }
    assert!(disconnected, "Tunnel did not transition to Disconnected status");

    // Cleanup server task
    server_task.abort();
}
