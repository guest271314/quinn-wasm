# Quinn-WASM with Direct Sockets API

This fork extends quinn-wasm to support the Direct Sockets API, enabling QUIC connections directly from the browser without requiring a WebSocket relay server.

## What's Different from Original quinn-wasm

The original quinn-wasm uses a WebSocket relay to forward UDP packets. This version adds support for the Direct Sockets API, which allows direct UDP communication from an Isolated Web App.

### Key Changes:

1. **New `direct_sockets` module** - Implements `AsyncUdpSocket` using Direct Sockets API
2. **No relay server needed** - Direct UDP communication to QUIC servers
3. **Isolated Web App support** - Designed to run as an IWA with Direct Sockets permission

## Architecture

```
┌─────────────────┐
│  Browser (IWA)  │
│                 │
│  Quinn (WASM)   │
│       ↓         │
│  Direct Sockets │
│   UDPSocket     │
└────────┬────────┘
         │ UDP
         ↓
┌─────────────────┐
│  QUIC Server    │
│   (External)    │
└─────────────────┘
```

Compare to original WebSocket relay architecture:

```
Browser → WebSocket → Relay Server → UDP → QUIC Server
```

## Building

### Prerequisites

1. Rust toolchain with wasm32 target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. wasm-pack:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

### Build Steps

```bash
# Build the Direct Sockets example
./build-direct-sockets.sh

# Or manually:
cd examples/direct-sockets-web
wasm-pack build --target web --out-dir pkg
```

## Running as Isolated Web App

### 1. Create Signed Web Bundle

You'll need to create a signed web bundle for the Isolated Web App. The manifest is already configured in `examples/direct-sockets-web/.well-known/manifest.webmanifest`.


#### Node.js

   ```js
   node bundle-iwa.js
   ```

#### Deno
   ```js
   DENO_COMPAT=1 deno -A bundle-iwa.js
   ```

#### Bun

   ```js
   bun bundle-iwa.js
   ```

### 2. Install in Chrome

1. Navigate to `chrome://web-app-internals/`
2. Install the signed web bundle
3. Launch with Direct Sockets enabled

### 3. Chrome Flags

Launch Chrome with:
```bash
chrome --enable-features=DirectSockets
```

## Testing

### Run QUIC Echo Server

```bash
cd examples/quic-echo-server
cargo run
```

This starts a QUIC server on port 4000 that echoes back messages.

### Connect from IWA

Open the installed Isolated Web App and:
1. Enter server address: `127.0.0.1:4000`
2. Type a message
3. Click "Connect & Send"

## Implementation Details

### Direct Sockets Integration

The `src/direct_sockets.rs` module implements:

- `DirectSocketUdp` - Wrapper around Direct Sockets `UDPSocket`
- `AsyncUdpSocket` trait implementation for Quinn
- JavaScript interop using wasm-bindgen

### Certificate Handling

Like the original, this uses a custom certificate verifier that accepts self-signed certificates. For production use, implement proper certificate validation.

### Current Limitations

1. **Isolated Web App only** - Direct Sockets API requires IWA context
2. **Chrome/Chromium only** - Direct Sockets is a Chrome-specific API
3. **Experimental status** - Direct Sockets API is still experimental
4. **Simplified implementation** - Some advanced UDP features not yet implemented

## Differences from WebSocket Relay Approach

| Feature | WebSocket Relay | Direct Sockets |
|---------|----------------|----------------|
| Relay Server | Required | Not needed |
| Latency | Higher (relay hop) | Lower (direct) |
| Browser Support | Any modern browser | Chrome with IWA |
| Deployment | Need relay infrastructure | IWA only |
| Security | Relay sees packet flow | Direct E2E encryption |

## Future Work

- [ ] Complete ReadableStream/WritableStream integration
- [ ] Add proper error handling for Direct Sockets failures
- [ ] Implement connection migration support
- [ ] Add performance benchmarks vs WebSocket relay
- [ ] Support for bound mode (server-side) UDP sockets

## Security Notes

⚠️ **Important Security Considerations:**

1. This implementation skips certificate validation for testing
2. Direct Sockets requires explicit user permission via IWA installation
3. Ensure proper certificate validation for production use
4. Consider implementing certificate pinning or custom PKI

## Contributing

This is an experimental fork demonstrating Direct Sockets integration. For production use, consider:

1. Completing the Stream API integration
2. Adding comprehensive error handling
3. Implementing proper certificate validation
4. Testing with various QUIC implementations

## Original Project

This is based on [Frando/quinn-wasm](https://github.com/Frando/quinn-wasm), which provides QUIC in the browser using WebSocket relay.