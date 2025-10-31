// Bundle and sign IWA for Direct Sockets QUIC test
import { bundleIsolatedWebApp, WebBundleId } from "./wbn-bundle.js";
import { readFileSync, writeFileSync } from "node:fs";
import { webcrypto } from "node:crypto";

const algorithm = { name: "Ed25519" };
const decoder = new TextDecoder();

// Load keys
const privateKey = readFileSync("./privateKey.json");
const publicKey = readFileSync("./publicKey.json");

// Import keys for Web Crypto
const cryptoKey = {
  privateKey: await webcrypto.subtle.importKey(
    "jwk",
    JSON.parse(decoder.decode(privateKey)),
    algorithm.name,
    true,
    ["sign"],
  ),
  publicKey: await webcrypto.subtle.importKey(
    "jwk",
    JSON.parse(decoder.decode(publicKey)),
    algorithm.name,
    true,
    ["verify"],
  ),
};

// Generate Web Bundle ID
const webBundleId = await new WebBundleId(
  cryptoKey.publicKey,
).serialize();

const isolatedWebAppURL = await new WebBundleId(
  cryptoKey.publicKey,
).serializeWithIsolatedWebAppOrigin();

console.log("Web Bundle ID:", webBundleId);
console.log("Isolated Web App URL:", isolatedWebAppURL);

// Create the signed web bundle
const { fileName, source, baseURL } = await bundleIsolatedWebApp({
  baseURL: isolatedWebAppURL,
  static: { dir: "examples/direct-sockets-web" },
  formatVersion: "b2",
  output: "signed.swbn",
  integrityBlockSign: {
    webBundleId,
    isIwa: true,
    // https://github.com/GoogleChromeLabs/webbundle-plugins/blob/d251f6efbdb41cf8d37b9b7c696fd5c795cdc231/packages/rollup-plugin-webbundle/test/test.js#L408
    // wbn-sign/lib/signers/node-crypto-signing-strategy.js
    strategies: [new (class CustomSigningStrategy {
      async sign(data) {
        return new Uint8Array(
          await webcrypto.subtle.sign(algorithm, cryptoKey.privateKey, data),
        );
      }
      async getPublicKey() {
        return cryptoKey.publicKey;
      }
    })()],
  },
  headerOverride: {
    "cross-origin-embedder-policy": "unsafe-none",
    "cross-origin-opener-policy": "unsafe-none",
    "cross-origin-resource-policy": "unsafe-none",
    "content-security-policy": // ; require-trusted-types-for 'script'
      "base-uri 'none'; default-src 'self'; object-src 'none'; frame-src 'self' http: https: blob: data: chrome-extension:; connect-src 'self' https: wss: chrome-extension:; script-src 'self' 'wasm-unsafe-eval' chrome-extension: http: blob: data:; img-src 'self' https: blob: data:; media-src 'self' https: blob: data:; font-src 'self' blob: data:; style-src 'self' 'unsafe-inline';frame-ancestors 'self' chrome-extension: http: blob: data:;",
  },
});

// Write the signed bundle
const outputFileName = fileName || "signed.swbn";
writeFileSync(outputFileName, new Uint8Array(source));

console.log("\n✅ Signed Web Bundle created successfully!");
console.log(`   File: ${outputFileName}`);
console.log(`   Size: ${(source.byteLength / 1024).toFixed(2)} KB`);
console.log("\n📦 To install:");
console.log("   1. Open Chrome with flags:");
console.log("      --enable-features=DirectSockets,IsolatedWebApps");
console.log("   2. Navigate to: chrome://web-app-internals/");
console.log("   3. Install from: " + outputFileName);
console.log("\n🆔 Web Bundle ID to remember:");
console.log(`   ${webBundleId}`);