#!/bin/bash

# Build script for quinn-wasm with Direct Sockets support
set -e

echo "Building quinn-wasm with Direct Sockets support..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Please install it first:"
    echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Build the Direct Sockets example
cd examples/direct-sockets-web
echo "Building Direct Sockets WASM module..."
wasm-pack build --target web --out-dir pkg

echo "Build complete!"
echo ""
echo "Next steps:"
echo "1. Create an Isolated Web App bundle"
echo "2. Install the IWA in Chrome/Chromium"
echo "3. Make sure to launch Chrome with:"
echo "   --enable-features=DirectSockets"
echo ""
echo "For testing locally, you can use the QUIC echo server:"
echo "   cd examples/quic-echo-server && cargo run"