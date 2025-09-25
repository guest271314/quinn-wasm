# Installation Guide for Direct Sockets QUIC IWA

## Prerequisites

1. **Chrome/Chromium Browser** (version 100+ with Direct Sockets support)
2. **Node.js** (for creating the signed web bundle)

## Step 1: Install Required Tools

### Install the Web Bundle CLI tools

```bash
# Install the wbn tool globally
npm install -g @chromium/web-bundle-cli

# Or use npx without installing
npx @chromium/web-bundle-cli --help
```

## Step 2: Prepare the IWA Files

Since the WASM build has compilation issues, we'll create a simpler test IWA first:

```bash
cd /Volumes/LLM/quinn-wasm

# Create IWA directory
mkdir -p iwa-bundle/assets
cd iwa-bundle

# Copy the necessary files
cp ../examples/direct-sockets-web/index.html .
cp ../examples/direct-sockets-web/manifest.webmanifest .

# Create a .well-known directory for the manifest
mkdir -p .well-known
mv manifest.webmanifest .well-known/
```

## Step 3: Create a Test Script

Create a simple test file to verify Direct Sockets work:

```bash
cat > test-direct-sockets.js << 'EOF'
// Test Direct Sockets API
async function testDirectSockets() {
    const status = document.getElementById('status') || document.body;

    if ('UDPSocket' in window) {
        status.innerHTML += '<div style="color: green;">✅ Direct Sockets API is available!</div>';

        try {
            const socket = new UDPSocket({
                localAddress: '0.0.0.0',
                localPort: 0
            });

            const { readable, writable } = await socket.opened;
            status.innerHTML += '<div style="color: green;">✅ Successfully created UDP socket!</div>';

            // Try to send a test packet to the QUIC server
            const writer = writable.getWriter();
            await writer.write({
                data: new Uint8Array([0x00, 0x01, 0x02]),
                remoteAddress: '127.0.0.1',
                remotePort: 4000
            });

            status.innerHTML += '<div style="color: green;">✅ Sent test packet to QUIC server!</div>';

            socket.close();
        } catch (error) {
            status.innerHTML += '<div style="color: red;">❌ Error: ' + error.message + '</div>';
        }
    } else {
        status.innerHTML += '<div style="color: red;">❌ Direct Sockets API not available. Make sure this is running as an IWA.</div>';
    }
}

// Run test when page loads
window.addEventListener('load', testDirectSockets);
EOF
```

## Step 4: Update the HTML

```bash
cat > index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Direct Sockets QUIC Test</title>
    <style>
        body { font-family: system-ui; padding: 20px; }
        h1 { color: #333; }
        #status div { margin: 10px 0; padding: 10px; background: #f0f0f0; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>Direct Sockets QUIC Test</h1>
    <div id="status"></div>
    <script src="test-direct-sockets.js"></script>
</body>
</html>
EOF
```

## Step 5: Generate Keys for Signing (One-time)

```bash
# Generate a private key for signing
openssl genrsa -out private.pem 2048

# Extract public key
openssl rsa -in private.pem -pubout -out public.pem

# Convert to the format needed for web bundles
openssl pkcs8 -topk8 -inform PEM -outform DER -in private.pem -out private.key -nocrypt
```

## Step 6: Create the Unsigned Web Bundle

```bash
# Go back to quinn-wasm directory
cd /Volumes/LLM/quinn-wasm

# Create the unsigned bundle first
npx @chromium/web-bundle-cli create \
  --baseURL "isolated-app://quic-direct-sockets/" \
  --output bundle.wbn \
  iwa-bundle/
```

## Step 7: Sign the Web Bundle

```bash
# Sign the bundle
npx @chromium/web-bundle-cli sign \
  --input bundle.wbn \
  --output signed.swbn \
  --privateKey private.key
```

## Step 8: Install in Chrome

### Option A: Via Chrome Flags (Developer Mode)

1. **Enable required flags in Chrome:**
   ```
   chrome://flags/#enable-isolated-web-apps
   chrome://flags/#enable-isolated-web-app-dev-mode
   ```
   Set both to "Enabled" and restart Chrome.

2. **Launch Chrome with Direct Sockets enabled:**
   ```bash
   # macOS
   /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
     --enable-features=IsolatedWebApps,IsolatedWebAppDevMode,DirectSockets \
     --allow-insecure-localhost

   # Linux
   google-chrome \
     --enable-features=IsolatedWebApps,IsolatedWebAppDevMode,DirectSockets \
     --allow-insecure-localhost

   # Windows
   chrome.exe \
     --enable-features=IsolatedWebApps,IsolatedWebAppDevMode,DirectSockets \
     --allow-insecure-localhost
   ```

3. **Install the IWA:**
   - Navigate to `chrome://web-app-internals/`
   - Click on "Install IWA from Signed Web Bundle"
   - Select your `signed.swbn` file
   - The app should appear in the list

### Option B: Developer Mode Installation

For testing without signing:

1. Navigate to `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select the `iwa-bundle` directory

## Step 9: Launch and Test

1. **Find your app:**
   - Go to `chrome://apps/`
   - Or check `chrome://web-app-internals/`

2. **Launch the IWA:**
   - Click on your "Direct Sockets QUIC Test" app
   - The app should open in its own window

3. **Verify Direct Sockets:**
   - You should see green checkmarks if Direct Sockets is working
   - Check Chrome DevTools console for detailed logs

## Troubleshooting

### If Direct Sockets API is not available:

1. **Verify Chrome version:**
   ```bash
   google-chrome --version
   ```
   Must be 100+ (preferably latest)

2. **Check flags are enabled:**
   - Visit `chrome://flags/`
   - Search for "Isolated Web Apps"
   - Ensure it's enabled

3. **Check IWA installation:**
   - Visit `chrome://web-app-internals/`
   - Your app should be listed
   - Check for any error messages

4. **Verify manifest permissions:**
   ```json
   {
     "permissions": ["direct-sockets"]
   }
   ```

### If connection to QUIC server fails:

1. **Ensure server is running:**
   ```bash
   ps aux | grep quic-echo-server
   ```

2. **Test UDP port:**
   ```bash
   nc -uz 127.0.0.1 4000
   ```

3. **Check Chrome console for errors:**
   - Right-click in IWA window
   - Select "Inspect"
   - Check Console tab

## Quick Test Commands

```bash
# 1. Check if QUIC server is running
curl http://localhost:4000 2>&1 | grep -q "Connection refused" && echo "Server not HTTP (good for QUIC)" || echo "Check server"

# 2. Verify IWA files exist
ls -la /Volumes/LLM/quinn-wasm/iwa-bundle/

# 3. Check Chrome is running with correct flags
ps aux | grep -i chrome | grep -i "DirectSockets"
```

## Next Steps

Once the basic IWA is working with Direct Sockets:

1. **Fix WASM compilation** (requires ring patches)
2. **Integrate quinn-wasm** QUIC client
3. **Test full QUIC echo communication**
4. **Deploy production version**

## Summary

This installation process will:
1. ✅ Create an Isolated Web App with Direct Sockets permission
2. ✅ Install it in Chrome with proper flags
3. ✅ Verify Direct Sockets API is accessible
4. ✅ Test UDP communication to the QUIC server

The QUIC echo server at `127.0.0.1:4000` is already running and waiting for connections!