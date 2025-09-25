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
const { fileName, source } = await bundleIsolatedWebApp({
  baseURL: isolatedWebAppURL,
  static: { dir: "iwa-bundle" },
  formatVersion: "b2",
  headerType: "h2",
  primaryURL: isolatedWebAppURL,
  privateKey: cryptoKey.privateKey,
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