# Direct Sockets IWA Installation & Testing Guide

## Prerequisites

✅ **Completed:**
- QUIC echo server running on port 4000
- Signed web bundle created: `signed.swbn`
- Web Bundle ID: `zrvn5gkytkehbpjbpl4jjbjcudzazqyzbht4blat7fs6ajwtz75qaaic`

## Installation Steps

### 1. Launch Chrome with Required Flags

Open a terminal and run:

```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --enable-features=DirectSockets,IsolatedWebApps \
  --enable-blink-features=DirectSockets
```

**Note:** Close all Chrome instances first or use a different profile:
```bash
/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
  --user-data-dir=/tmp/chrome-iwa-test \
  --enable-features=DirectSockets,IsolatedWebApps \
  --enable-blink-features=DirectSockets
```

### 2. Install the IWA

1. Navigate to: `chrome://web-app-internals/`
2. Click "Install from Signed Web Bundle"
3. Select file: `/Volumes/LLM/quinn-wasm/signed.swbn`
4. Confirm installation

### 3. Launch the IWA

After installation:
1. The IWA should appear in `chrome://apps/`
2. Click on "Direct Sockets QUIC Test" to launch
3. Or navigate to: `isolated-app://zrvn5gkytkehbpjbpl4jjbjcudzazqyzbht4blat7fs6ajwtz75qaaic/`

## Testing Direct Sockets

### Test 1: Basic UDP Socket Creation
The IWA will automatically test:
1. Direct Sockets API availability
2. UDP socket creation
3. Sending test packet to QUIC server (127.0.0.1:4000)

Expected output:
```
✅ Direct Sockets API is available!
✅ Successfully created UDP socket!
✅ Successfully sent test packet!
✅ Socket closed successfully
🎉 Direct Sockets API is working! Ready for QUIC implementation.
```

### Test 2: Manual Testing
Click "Test Direct Sockets" button to re-run tests

## Servers Running

| Server | Port | Status | Purpose |
|--------|------|--------|---------|
| QUIC Echo Server | 4000 | ✅ Running | Echo server for QUIC testing |
| WebTransport Server | 4433 | ⚠️ Port conflict | Alternative WebTransport testing |

## Troubleshooting

### Direct Sockets API not available
- Ensure Chrome was launched with correct flags
- Check chrome://flags for DirectSockets status
- Verify IWA is properly installed

### Connection Issues
- Check server is running: `lsof -i :4000`
- Verify no firewall blocking localhost connections
- Check Chrome DevTools console for errors

### Certificate Warnings
The QUIC server uses self-signed certificates. This is expected for development.

## Web Bundle Details

- **Bundle ID:** `zrvn5gkytkehbpjbpl4jjbjcudzazqyzbht4blat7fs6ajwtz75qaaic`
- **Size:** 6.28 KB
- **Contents:**
  - `/index.html` - Test interface
  - `/test-direct-sockets.js` - Direct Sockets test logic
  - `/.well-known/manifest.webmanifest` - IWA manifest

## Next Steps

Once Direct Sockets is confirmed working:
1. Implement full QUIC client in the IWA
2. Test bidirectional QUIC communication
3. Integrate with quinn-wasm WASM module (when compilation issues resolved)