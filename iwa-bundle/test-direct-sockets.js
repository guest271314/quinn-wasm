// Test Direct Sockets API
async function testDirectSockets() {
    const status = document.getElementById('status');
    status.innerHTML = ''; // Clear previous results

    // Test 1: Check API availability
    if ('UDPSocket' in window) {
        addStatus('✅ Direct Sockets API is available!', 'success');

        try {
            // Test 2: Create UDP socket
            addStatus('Creating UDP socket...', 'info');
            const socket = new UDPSocket({
                localAddress: '0.0.0.0',
                localPort: 0
            });

            const { readable, writable } = await socket.opened;
            addStatus('✅ Successfully created UDP socket!', 'success');

            // Test 3: Try to send a test packet
            addStatus('Attempting to send packet to QUIC server at 127.0.0.1:4000...', 'info');
            const writer = writable.getWriter();

            // Send a simple UDP packet (not a full QUIC packet, just testing UDP)
            const testData = new TextEncoder().encode('Direct Sockets Test');
            await writer.write({
                data: testData,
                remoteAddress: '127.0.0.1',
                remotePort: 4000
            });

            addStatus('✅ Successfully sent test packet!', 'success');

            // Test 4: Try to read (with timeout)
            addStatus('Waiting for response (3 second timeout)...', 'info');
            const reader = readable.getReader();

            const timeout = setTimeout(() => {
                addStatus('⏱️ Read timeout (expected for non-QUIC packet)', 'info');
                reader.releaseLock();
                socket.close();
            }, 3000);

            try {
                const { value, done } = await reader.read();
                clearTimeout(timeout);
                if (value) {
                    addStatus(`✅ Received response: ${value.data.length} bytes`, 'success');
                }
            } catch (e) {
                clearTimeout(timeout);
                addStatus(`Read error (expected): ${e.message}`, 'info');
            }

            socket.close();
            addStatus('✅ Socket closed successfully', 'success');
            addStatus('🎉 Direct Sockets API is working! Ready for QUIC implementation.', 'success');

        } catch (error) {
            addStatus(`❌ Error: ${error.message}`, 'error');
            console.error('Full error:', error);
        }
    } else {
        addStatus('❌ Direct Sockets API not available', 'error');
        addStatus('Make sure:', 'error');
        addStatus('1. This is running as an Isolated Web App', 'error');
        addStatus('2. Chrome was launched with --enable-features=DirectSockets', 'error');
        addStatus('3. The IWA manifest includes "direct-sockets" permission', 'error');
    }

    // Additional info
    addStatus(`Browser: ${navigator.userAgent}`, 'info');
    addStatus(`Location: ${location.protocol}//${location.host}`, 'info');
}

function addStatus(message, type = 'info') {
    const status = document.getElementById('status');
    const div = document.createElement('div');
    div.textContent = message;

    if (type === 'success') {
        div.className = 'success';
        div.style.color = 'green';
    } else if (type === 'error') {
        div.className = 'error';
        div.style.color = 'red';
    } else {
        div.style.color = '#666';
    }

    status.appendChild(div);
    console.log(message);
}

// Run test when page loads
window.addEventListener('load', () => {
    addStatus('Page loaded, checking Direct Sockets...', 'info');
    testDirectSockets();
});