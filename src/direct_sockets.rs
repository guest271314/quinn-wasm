use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::ready;
use std::{
    fmt,
    io::{self, IoSliceMut},
    net::SocketAddr,
    sync::Arc,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use quinn::{
    udp::{RecvMeta, Transmit},
    AsyncUdpSocket, ClientConfig, Endpoint, ServerConfig,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys;

pub use quinn;

// Direct Sockets UDP message structure
#[derive(Serialize, Deserialize, Debug)]
pub struct UdpMessage {
    pub data: Vec<u8>,
    pub remote_address: String,
    pub remote_port: u16,
}

#[wasm_bindgen]
extern "C" {
    // Direct Sockets API bindings
    pub type UDPSocket;

    #[wasm_bindgen(constructor)]
    pub fn new(options: &JsValue) -> UDPSocket;

    #[wasm_bindgen(method, getter)]
    pub fn opened(this: &UDPSocket) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn close(this: &UDPSocket);
}

#[wasm_bindgen]
extern "C" {
    pub type UDPSocketStreams;

    #[wasm_bindgen(method, getter)]
    pub fn readable(this: &UDPSocketStreams) -> web_sys::ReadableStream;

    #[wasm_bindgen(method, getter)]
    pub fn writable(this: &UDPSocketStreams) -> web_sys::WritableStream;
}

pub async fn create_direct_socket_endpoint(
    local_addr: SocketAddr,
    server_config: Option<ServerConfig>,
    client_config: Option<ClientConfig>,
) -> anyhow::Result<Endpoint> {
    // Create Direct Socket options
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("localAddress"),
        &JsValue::from_str(&local_addr.ip().to_string()),
    )?;
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("localPort"),
        &JsValue::from(local_addr.port()),
    )?;

    // Create UDP socket
    let udp_socket = UDPSocket::new(&options.into());

    // Wait for socket to open
    let opened = JsFuture::from(udp_socket.opened()).await?;
    let streams: UDPSocketStreams = opened.dyn_into()?;

    let socket = Arc::new(DirectSocketUdp::new(streams, local_addr)?);
    let runtime = Arc::new(crate::runtime::JsRuntime);

    let mut endpoint =
        Endpoint::new_with_abstract_socket(Default::default(), server_config, socket, runtime)?;
    if let Some(client_config) = client_config {
        endpoint.set_default_client_config(client_config);
    }
    Ok(endpoint)
}

pub struct DirectSocketUdp {
    readable: Mutex<Pin<Box<web_sys::ReadableStream>>>,
    writable: Mutex<Pin<Box<web_sys::WritableStream>>>,
    local_addr: SocketAddr,
    send_queue: Mutex<Vec<UdpMessage>>,
}

impl fmt::Debug for DirectSocketUdp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DirectSocketUdp")
            .field("local_addr", &self.local_addr)
            .finish()
    }
}

// Direct Sockets are only available in Isolated Web Apps, which are single-threaded
unsafe impl Sync for DirectSocketUdp {}
unsafe impl Send for DirectSocketUdp {}

impl DirectSocketUdp {
    pub fn new(streams: UDPSocketStreams, local_addr: SocketAddr) -> anyhow::Result<Self> {
        Ok(Self {
            readable: Mutex::new(Box::pin(streams.readable())),
            writable: Mutex::new(Box::pin(streams.writable())),
            local_addr,
            send_queue: Mutex::new(Vec::new()),
        })
    }

    async fn write_to_socket(&self, message: UdpMessage) -> Result<(), io::Error> {
        // Convert message to JavaScript object
        let js_message = js_sys::Object::new();

        // Set data field
        let data_array = js_sys::Uint8Array::from(&message.data[..]);
        js_sys::Reflect::set(
            &js_message,
            &JsValue::from_str("data"),
            &data_array,
        ).map_err(|e| io::Error::new(ErrorKind::Other, format!("{:?}", e)))?;

        // Set remote address
        js_sys::Reflect::set(
            &js_message,
            &JsValue::from_str("remoteAddress"),
            &JsValue::from_str(&message.remote_address),
        ).map_err(|e| io::Error::new(ErrorKind::Other, format!("{:?}", e)))?;

        // Set remote port
        js_sys::Reflect::set(
            &js_message,
            &JsValue::from_str("remotePort"),
            &JsValue::from(message.remote_port),
        ).map_err(|e| io::Error::new(ErrorKind::Other, format!("{:?}", e)))?;

        // Write to the writable stream
        let writable = self.writable.lock().unwrap();
        // Note: Actual writing implementation would require more complex JavaScript interop
        // This is a simplified version showing the structure

        Ok(())
    }
}

impl AsyncUdpSocket for DirectSocketUdp {
    fn max_transmit_segments(&self) -> usize {
        1
    }

    fn max_receive_segments(&self) -> usize {
        1
    }

    fn poll_send(
        &self,
        cx: &mut Context,
        transmits: &[Transmit],
    ) -> Poll<Result<usize, io::Error>> {
        debug!("DirectSocket: sending {} transmits", transmits.len());

        for transmit in transmits {
            let message = UdpMessage {
                data: transmit.contents.to_vec(),
                remote_address: transmit.destination.ip().to_string(),
                remote_port: transmit.destination.port(),
            };

            debug!(
                "DirectSocket: sending to {}:{} len {}",
                message.remote_address,
                message.remote_port,
                message.data.len()
            );

            // Queue the message for sending
            self.send_queue.lock().unwrap().push(message);
        }

        // Process send queue
        // Note: This would need proper async handling with the WritableStream
        // For now, we'll mark as ready immediately
        Poll::Ready(Ok(transmits.len()))
    }

    fn poll_recv(
        &self,
        cx: &mut Context,
        bufs: &mut [IoSliceMut<'_>],
        meta: &mut [RecvMeta],
    ) -> Poll<io::Result<usize>> {
        // Note: This would need proper integration with ReadableStream
        // to read incoming UDP messages
        // For now, returning pending
        Poll::Pending
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.local_addr)
    }
}