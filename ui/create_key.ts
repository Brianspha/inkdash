import {
  mnemonicGenerate,
  mnemonicToMiniSecret,
  mnemonicValidate,
  base58Encode,
  ed25519PairFromSeed,
} from "@polkadot/util-crypto";

async function main() {
  // Create mnemonic string for Alice using BIP39
  const mnemonicAlice = mnemonicGenerate();

  console.log(`Generated mnemonic: ${mnemonicAlice}`);

  // Validate the mnemonic string that was generated
  const isValidMnemonic = mnemonicValidate(mnemonicAlice);

  console.log(`isValidMnemonic: ${isValidMnemonic}`);

  // Create valid Substrate-compatible seed from mnemonic
  const seedAlice = mnemonicToMiniSecret(mnemonicAlice);

  // Generate new public/private keypair for Alice from the supplied seed
  const { publicKey, secretKey } = ed25519PairFromSeed(seedAlice);

  // Convert the public key to hex format
  const publicKeyHex = Buffer.from(publicKey).toString("hex");

  // Convert to base58 format (alternative representation)
  const publicKeyBase58 = base58Encode(publicKey);
  const secretKeyHex = Buffer.from(secretKey).toString("hex");

  console.log("Public Key (hex):", publicKeyHex);
  console.log("Public Key (base58):", publicKeyBase58);
  console.log("Secret Key (hex):", secretKeyHex);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
