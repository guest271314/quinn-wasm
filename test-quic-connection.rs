// Test script to verify QUIC server connectivity
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== QUIC Connection Test ===\n");

    // Test 1: Check if server is reachable
    let server_addr: SocketAddr = "127.0.0.1:4000".parse()?;
    println!("✓ Server address parsed: {}", server_addr);

    // Test 2: Create a simple QUIC client
    println!("\nAttempting to connect to QUIC server...");

    // Configure client with self-signed cert acceptance
    let mut roots = rustls::RootCertStore::empty();
    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    // Try to connect
    match endpoint.connect(server_addr, "localhost") {
        Ok(connecting) => {
            println!("✓ Connection initiated");

            match connecting.await {
                Ok(connection) => {
                    println!("✓ Successfully connected to QUIC server!");
                    println!("  Connection ID: {:?}", connection.stable_id());

                    // Test 3: Open a bidirectional stream
                    match connection.open_bi().await {
                        Ok((mut send, mut recv)) => {
                            println!("✓ Bidirectional stream opened");

                            // Test 4: Send test message
                            let test_message = b"Hello from test client!";
                            send.write_all(test_message).await?;
                            send.finish().await?;
                            println!("✓ Sent test message: {:?}", String::from_utf8_lossy(test_message));

                            // Test 5: Receive echo
                            let response = recv.read_to_end(1024).await?;
                            println!("✓ Received echo: {:?}", String::from_utf8_lossy(&response));

                            if response == test_message {
                                println!("\n🎉 SUCCESS: Echo server working correctly!");
                            } else {
                                println!("\n⚠️  WARNING: Echo mismatch");
                            }
                        }
                        Err(e) => println!("✗ Failed to open stream: {}", e),
                    }

                    connection.close(0u32.into(), b"test complete");
                }
                Err(e) => println!("✗ Connection failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to initiate connection: {}", e),
    }

    println!("\n=== Test Complete ===");
    Ok(())
}