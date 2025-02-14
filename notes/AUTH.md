# Authentication

## Deterministic Public Keys

OfficeX uses pre-determined public key generation. From a seed phrase or salt, we can know our public keys ahead of time before actually deploying. This allows offchain apps to still have auth via signatures, without needing blockchain.

Offchain OfficeX clients/servers would just have a seed phrase that can be deterministically used to generate salts, which each respectively can be entered into factory contract creates, to arrive at their predetermined public keys. Similarly for end wallets.

This allows us to have a unified auth system via math without internet connection or blockchain. Classic pre-internet cryptography.

```js
import { ethers } from "ethers";
import { Secp256k1KeyIdentity } from "@dfinity/identity";
import { Principal } from "@dfinity/principal";
import { sha224 } from "@dfinity/identity";

// Common seed phrase for all derivations
const seedPhrase =
  "violin artwork tourist punch flight axis shy eagle divide bulb catalog flash";

// 1. Raw Wallet Addresses
async function deriveWalletAddresses() {
  // EVM wallet
  const evmWallet = ethers.Wallet.fromPhrase(seedPhrase);
  const evmAddress = evmWallet.address;

  // ICP identity
  const seed = ethers.utils.arrayify(ethers.utils.mnemonicToSeed(seedPhrase));
  const icpIdentity = Secp256k1KeyIdentity.fromSeed(seed.slice(0, 32));
  const icpPrincipal = icpIdentity.getPrincipal().toString();

  return {
    evmWalletAddress: evmAddress,
    icpPrincipal: icpPrincipal,
  };
}

// 2. Factory Contract Deployment Addresses
async function deriveFactoryAddresses(username = "defaultuser") {
  // Generate deterministic salts from seed phrase
  const evmSalt = ethers.keccak256(ethers.toUtf8Bytes(seedPhrase));
  const icpSalt = Array.from(sha224(new TextEncoder().encode(seedPhrase)));

  // EVM contract address prediction
  // Assumes factory is deployed at FACTORY_ADDRESS
  const FACTORY_ADDRESS = "0x123..."; // Your factory address
  const EVM_BYTECODE = "0x..."; // Your contract bytecode

  const evmContractAddress = ethers.getCreate2Address(
    FACTORY_ADDRESS,
    evmSalt,
    ethers.keccak256(
      ethers.concat([
        EVM_BYTECODE,
        ethers.AbiCoder.defaultAbiCoder().encode(
          ["address", "string"],
          [evmWallet.address, username]
        ),
      ])
    )
  );

  // ICP canister ID prediction
  // Combining all deterministic inputs
  const combinedInput = new Uint8Array([
    ...icpIdentity.getPrincipal().toUint8Array(),
    ...new TextEncoder().encode(username),
    ...icpSalt,
  ]);

  const canisterId = Principal.fromUint8Array(sha224(combinedInput));

  return {
    salts: {
      evm: evmSalt,
      icp: icpSalt,
    },
    factoryAddresses: {
      evmContract: evmContractAddress,
      icpCanister: canisterId.toString(),
    },
  };
}

// Usage example
async function main() {
  // 1. Get wallet addresses
  const walletAddresses = await deriveWalletAddresses();
  console.log("Wallet Addresses:", walletAddresses);

  // 2. Get factory-deployed addresses
  const factoryAddresses = await deriveFactoryAddresses("myusername");
  console.log("Factory Addresses:", factoryAddresses);

  return {
    wallets: walletAddresses,
    factory: factoryAddresses,
  };
}

// Example output structure:
/*
{
    wallets: {
        evmWalletAddress: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
        icpPrincipal: "dy4hl-jzp3d-qf7oj-..."
    },
    factory: {
        salts: {
            evm: "0x123...",
            icp: [1, 2, 3, ...]
        },
        factoryAddresses: {
            evmContract: "0x789...",
            icpCanister: "rrkah-fqaaa-..."
        }
    }
}
*/
```

## Signature Proofs

We can use the generated ICP address (principal) and create cryptographic signature proofs to prove a client indeed owns the address they claim. This can be used as a "native api key" for users, before we generate a normal one. Below shows this client server interaction.

```js
// client
import { Secp256k1KeyIdentity } from "@dfinity/identity";
const identity = Secp256k1KeyIdentity.fromSeed(seed);

const challenge = { timestamp: Date.now() };
const challengeBytes = new TextEncoder().encode(JSON.stringify(challenge));
const signature = await identity.sign(challengeBytes);

const proof = {
  challenge,
  signature: Array.from(signature),
  principal: identity.getPrincipal().toString(),
};

// server
import { Principal } from "@dfinity/principal";
import { Secp256k1KeyIdentity } from "@dfinity/identity";

async function verifySignature(proof) {
  const principal = Principal.fromText(proof.principal);
  const challengeBytes = new TextEncoder().encode(
    JSON.stringify(proof.challenge)
  );

  return await identity.verify(challengeBytes, new Uint8Array(proof.signature));
}
```

```rust
// server

```

## Headers

For both regular api-keys and signature based auth, we put it in the HTTP request headers as `Authorization: Bearer yourtoken...` as its a standard header (not custom) and thus wont trigger an OPTIONS cors preflight request to ICP http canister (saving a lot of gas).

In order to support both regular api-keys and signatures based auth, we prefix it so that our server has an easy time figuring out how to decipher the access method.

```js
const regularApiKeyHeader = {
  Authorization: `Bearer ApiKey_${API_KEY}`,
};
const signatureHeader = {
  Authorization: `Bearer Signature_${SIGNATURE}`,
};
```
