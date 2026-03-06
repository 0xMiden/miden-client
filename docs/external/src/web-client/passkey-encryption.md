---
title: Passkey Encryption
sidebar_position: 14
---

# Passkey Encryption (WebAuthn PRF)

The Miden web SDK supports opt-in passkey-based encryption for secret keys at rest. When enabled, keys stored in IndexedDB are encrypted using AES-256-GCM with a wrapping key derived from the WebAuthn PRF extension (Touch ID, Face ID, Windows Hello).

Without passkey encryption, secret keys are stored as plaintext in IndexedDB, accessible to any JavaScript running in the same origin (XSS payloads, compromised dependencies, browser extensions). Passkey encryption adds a hardware-backed layer of protection — decrypting keys requires a biometric prompt.

## Browser Support

| Browser | Minimum Version | PRF Support |
|---------|----------------|-------------|
| Chrome | 116+ | Yes |
| Edge | 116+ | Yes |
| Safari | 18+ | Yes |
| Firefox | — | Not supported |

## Quick Start

### Web SDK (Vanilla JS/TS)

```typescript
import { MidenClient, isPasskeyPrfSupported } from "@miden-sdk/miden-sdk";

// Check browser support first
if (await isPasskeyPrfSupported()) {
  const client = await MidenClient.create({
    passkeyEncryption: true, // Triggers biometric prompt
    storeName: "my-wallet",  // Recommended: explicit store name for migration support
  });

  // All key operations are now transparently encrypted
  const wallet = await client.accounts.create();
}
```

### React SDK

```tsx
import { MidenProvider, isPasskeyPrfSupported } from "@miden-sdk/react";

function App() {
  return (
    <MidenProvider
      config={{
        rpcUrl: "testnet",
        passkeyEncryption: true,
        storeName: "my-wallet",
      }}
    >
      <YourApp />
    </MidenProvider>
  );
}
```

## How It Works

1. **Registration (once):** `navigator.credentials.create()` registers a passkey with the PRF extension, bound to the current origin.
2. **Authentication (each session):** `navigator.credentials.get()` evaluates the PRF extension, returning a deterministic 32-byte secret from the authenticator's secure enclave. This requires biometric verification (Touch ID, Face ID, PIN).
3. **Key derivation:** The PRF output is fed through HKDF-SHA256 to derive a non-extractable AES-256-GCM wrapping key.
4. **Encrypt on write:** When a new account is created, the secret key is encrypted with the wrapping key and stored in a dedicated `MidenKeystore_*` IndexedDB database.
5. **Decrypt on read:** When a key is needed (e.g., for signing), the ciphertext is read from IndexedDB and decrypted with the wrapping key. The plaintext exists only briefly in WASM memory during signing.

The wrapping key is held in a JavaScript closure as a non-extractable `CryptoKey` — raw key bytes are never exposed to JavaScript.

## Feature Detection

Always check for browser support before offering passkey encryption to users:

```typescript
import { isPasskeyPrfSupported } from "@miden-sdk/miden-sdk";
// or
import { isPasskeyPrfSupported } from "@miden-sdk/react";

const supported = await isPasskeyPrfSupported();
if (supported) {
  // Safe to enable passkey encryption
}
```

## Configuration Options

### Simple (Register or Reuse)

```typescript
const client = await MidenClient.create({
  passkeyEncryption: true,
});
```

When `true`, the SDK checks `localStorage` for an existing credential. If found, it authenticates with the existing passkey. If not found, it registers a new passkey. This is the recommended approach for most applications.

### Explicit Credential

```typescript
const client = await MidenClient.create({
  passkeyEncryption: {
    credentialId: "base64url-encoded-credential-id",
  },
});
```

Pass an explicit credential ID to skip the `localStorage` lookup and authenticate with a specific passkey. Useful for multi-account scenarios or credential management UIs.

### Full Options

```typescript
const client = await MidenClient.create({
  passkeyEncryption: {
    credentialId: "...",         // Existing credential (optional)
    rpId: "example.com",        // Relying party ID (default: hostname)
    rpName: "My Wallet",        // Display name during registration
    userName: "user@example.com" // User name during registration
  },
  storeName: "my-wallet",       // Store isolation key
});
```

## Credential Persistence

- **Credential ID** is stored in `localStorage` under the key `miden_passkey_credential_{storeName}`. This allows the SDK to automatically reuse the passkey in subsequent sessions without re-registration.
- **Encrypted keys** are stored in a separate IndexedDB database (`MidenKeystore_{storeName}`), isolated from the main client database.
- **Wrapping key** exists only in memory (non-extractable `CryptoKey`). It is derived fresh on each `MidenClient.create()` call from the authenticator's PRF output.

## Migration from Plaintext

When `getKey` is called and no encrypted entry exists in the keystore database, the SDK attempts to read the plaintext key from the main client database. If found, the key is transparently re-encrypted and the plaintext entry is removed.

Migration is only available when `storeName` is explicitly provided, since the auto-generated database name (`MidenClientDB_{network_id}`) is not known to JavaScript before WASM initialization.

## Export/Import Limitations

The `exportStore()`/`importStore()` flow exports the main WASM store but does **not** include the separate `MidenKeystore_*` database. This means:

- Exported stores will not include encrypted secret keys.
- Users should perform exports while the passkey-enabled client is active (keys are decrypted in-session).

## Cross-Device Behavior

Passkeys sync within the same ecosystem:

- **Apple** (iCloud Keychain): MacBook, iPhone, iPad share the same PRF output.
- **Google** (Password Manager): Android devices + Chrome share synced passkeys.
- **Windows Hello**: Currently device-bound (no sync).

Cross-ecosystem (e.g., MacBook to Android) is **not supported**. Users migrating between ecosystems should use the store export/import flow while the client is active.

## Credential Loss

If a user loses their passkey (device factory reset without cloud sync, ecosystem switch), encrypted keys are **permanently unrecoverable**. The wrapping key exists only inside the authenticator's secure enclave.

Mitigations:
1. Recommend enabling cloud sync (iCloud Keychain, Google Password Manager).
2. Users should export their store while the client is active before switching ecosystems.

## Security Properties

| Property | Detail |
|----------|--------|
| Wrapping key | Non-extractable `CryptoKey` — raw bytes never exposed to JS |
| IV (nonce) | Fresh random 12-byte IV per encryption |
| Authentication | AES-GCM 16-byte auth tag detects tampering |
| AAD binding | Ciphertext is bound to its public key commitment, preventing ciphertext swapping |
| Authenticator | Platform-bound (hardware), not roaming/USB keys |
| User verification | Biometric/PIN required on every session |
| Key derivation | HKDF-SHA256 with application-specific salt and info strings |

## Encrypted Format

Keys are stored as hex strings in IndexedDB with the `MWEB` envelope format:

```
[4B: "MWEB"] [1B: version=0x01] [12B: IV] [NB: AES-GCM ciphertext + 16B auth tag]
```

This format is distinct from the native CLI's `MENC` format (which uses Argon2id + ChaCha20-Poly1305). The version byte enables forward-compatible format changes.
