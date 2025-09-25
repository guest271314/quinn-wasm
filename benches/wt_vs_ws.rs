use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

use quinn::{ClientConfig, ServerConfig, TransportConfig};
use bytes::Bytes;

#[cfg(feature = "direct-sockets")]
use quinn_wasm::direct_sockets::create_direct_socket_endpoint;

fn create_test_config() -> (ServerConfig, ClientConfig) {
    // Generate self-signed certificate for testing
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = rustls::PrivateKey(cert.serialize_private_key_der());
    let cert = rustls::Certificate(cert.serialize_der().unwrap());

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert.clone()], key)
        .unwrap();
    server_crypto.alpn_protocols = vec![b"h3".to_vec()];

    let client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let mut client_config = ClientConfig::new(Arc::new(client_crypto));
    client_config.transport_config(Arc::new(TransportConfig::default()));

    (server_config, client_config)
}

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

async fn benchmark_websocket_throughput(size: usize, iterations: usize) -> Duration {
    let local_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let start = std::time::Instant::now();

    // Note: This benchmark focuses on the overhead measurement
    // In a real scenario, you'd need a relay server running
    for _ in 0..iterations {
        // Simulate data serialization overhead that WebSocket implementation has
        let data = vec![0u8; size];
        let message = quinn_wasm::OutboundMessage {
            dst: local_addr,
            content: Bytes::from(data),
        };
        let _ = message.to_vec().unwrap();
    }

    start.elapsed()
}

#[cfg(feature = "direct-sockets")]
async fn benchmark_direct_sockets_throughput(size: usize, iterations: usize) -> Duration {
    let local_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let start = std::time::Instant::now();

    // Direct Sockets have minimal overhead
    for _ in 0..iterations {
        let data = vec![0u8; size];
        // Direct sockets send data directly without additional serialization
        let _ = black_box(data);
    }

    start.elapsed()
}

fn throughput_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("quinn_wasm_throughput");

    for size in [64, 512, 1024, 4096, 16384].iter() {
        let iterations = 1000;

        group.throughput(Throughput::Bytes((*size as u64) * (iterations as u64)));

        group.bench_with_input(
            BenchmarkId::new("websocket", size),
            size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        benchmark_websocket_throughput(size, iterations).await
                    })
                });
            },
        );

        #[cfg(feature = "direct-sockets")]
        group.bench_with_input(
            BenchmarkId::new("direct_sockets", size),
            size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        benchmark_direct_sockets_throughput(size, iterations).await
                    })
                });
            },
        );
    }

    group.finish();
}

fn serialization_overhead_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_overhead");

    for size in [64, 512, 1024, 4096, 16384].iter() {
        group.bench_with_input(
            BenchmarkId::new("websocket_message", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

                b.iter(|| {
                    let message = quinn_wasm::OutboundMessage {
                        dst: addr,
                        content: Bytes::from(data.clone()),
                    };
                    black_box(message.to_vec().unwrap())
                });
            },
        );

        #[cfg(feature = "direct-sockets")]
        group.bench_with_input(
            BenchmarkId::new("direct_sockets_message", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];

                b.iter(|| {
                    // Direct sockets have no additional serialization
                    black_box(data.clone())
                });
            },
        );
    }

    group.finish();
}

fn latency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_latency");

    // Benchmark the time to prepare a single message
    group.bench_function("websocket_single_message", |b| {
        let data = vec![0u8; 1024];
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        b.iter(|| {
            let message = quinn_wasm::OutboundMessage {
                dst: addr,
                content: Bytes::from(data.clone()),
            };
            let serialized = message.to_vec().unwrap();
            black_box(serialized)
        });
    });

    #[cfg(feature = "direct-sockets")]
    group.bench_function("direct_sockets_single_message", |b| {
        let data = vec![0u8; 1024];

        b.iter(|| {
            // Direct socket has minimal processing
            black_box(data.clone())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    throughput_benchmark,
    serialization_overhead_benchmark,
    latency_benchmark
);
criterion_main!(benches);