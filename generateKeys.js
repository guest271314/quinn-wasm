// Generate Ed25519 keys for IWA signing
import { webcrypto } from "node:crypto";
import { writeFileSync } from "node:fs";

const algorithm = { name: "Ed25519" };

console.log("Generating Ed25519 key pair for IWA signing...");

const keyPair = await webcrypto.subtle.generateKey(
  algorithm.name,
  true,
  ["sign", "verify"]
);

const privateKey = await webcrypto.subtle.exportKey("jwk", keyPair.privateKey);
const publicKey = await webcrypto.subtle.exportKey("jwk", keyPair.publicKey);

writeFileSync("privateKey.json", JSON.stringify(privateKey, null, 2));
writeFileSync("publicKey.json", JSON.stringify(publicKey, null, 2));

console.log("✅ Keys generated successfully!");
console.log("   - privateKey.json");
console.log("   - publicKey.json");
console.log("\nKeep your private key secure!");