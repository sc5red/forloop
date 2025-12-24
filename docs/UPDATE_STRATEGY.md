# forloop Update Strategy

## Principles

1. **No automatic updates** - User must explicitly initiate
2. **No identity leakage** - Update checks cannot identify users
3. **Cryptographic verification** - All updates signed
4. **Offline support** - Can update without network
5. **Reproducible verification** - Users can verify binaries match source

---

## Update Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         UPDATE SYSTEM                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │   Update     │    │   Update     │    │   Signature          │   │
│  │   Server     │    │   Manifest   │    │   Verification       │   │
│  │   (.onion)   │───▶│   (signed)   │───▶│   (Ed25519)          │   │
│  └──────────────┘    └──────────────┘    └──────────────────────┘   │
│                                                 │                    │
│                                                 ▼                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────┐   │
│  │   Binary     │    │   Hash       │    │   Install            │   │
│  │   Download   │◀───│   Verify     │◀───│   (user-initiated)   │   │
│  │   (via Tor)  │    │   (SHA256)   │    │                      │   │
│  └──────────────┘    └──────────────┘    └──────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Update Server

### Location

- Primary: `forloopxxxxxx.onion` (Tor hidden service)
- Mirror: Multiple .onion addresses for resilience
- No clearnet servers (would leak timing/IP)

### Why .onion Only

1. Request comes through Tor anyway (browser's network layer)
2. No exit node sees update traffic
3. Server location is hidden
4. No CDN/third-party involvement

### Request Privacy

Each update check is indistinguishable from any other:
- No user ID
- No hardware ID
- No install ID
- No version string (fetches same manifest)
- Random timing jitter before check

---

## Update Manifest

### Format

```json
{
    "version": "1.2.3",
    "release_date": "2024-01-15",
    "min_required_version": "1.0.0",
    "security_update": true,
    "artifacts": {
        "linux-x86_64": {
            "url": "http://forloopxxxxxx.onion/releases/1.2.3/forloop-1.2.3-linux-x86_64.tar.xz",
            "sha256": "abc123...",
            "size": 85000000,
            "sig_url": "http://forloopxxxxxx.onion/releases/1.2.3/forloop-1.2.3-linux-x86_64.tar.xz.sig"
        },
        "linux-aarch64": {
            "url": "...",
            "sha256": "...",
            "size": 82000000,
            "sig_url": "..."
        }
    },
    "changelog_url": "http://forloopxxxxxx.onion/releases/1.2.3/CHANGELOG.md",
    "manifest_signature": "..."
}
```

### Manifest Signature

The entire manifest (minus the signature field) is signed with Ed25519.
Public key is compiled into the browser binary.

---

## Signing Keys

### Key Hierarchy

```
Master Key (offline, air-gapped)
    │
    ├── Release Signing Key (rotated yearly)
    │       │
    │       └── Signs release binaries
    │
    └── Manifest Signing Key (rotated yearly)
            │
            └── Signs update manifests
```

### Key Storage

- Master key: Hardware security module (HSM), offline
- Release keys: Stored on dedicated signing machine
- Never touch networked computers

### Key Rotation

- Annual rotation of signing keys
- Old keys remain valid for 1 year transition
- Announced via signed manifest with old key

---

## Update Flow

### User-Initiated Check

```rust
// User clicks "Check for Updates" or runs:
// forloop --check-updates

pub async fn check_for_updates() -> Result<Option<UpdateInfo>, UpdateError> {
    // 1. Apply random delay (0-60 seconds)
    let jitter = rand::thread_rng().gen_range(0..60);
    tokio::time::sleep(Duration::from_secs(jitter)).await;
    
    // 2. Fetch manifest via Tor
    let manifest_url = "http://forloopxxxxxx.onion/manifest.json";
    let manifest_bytes = fetch_via_tor(manifest_url).await?;
    
    // 3. Verify manifest signature
    let manifest: UpdateManifest = parse_manifest(&manifest_bytes)?;
    if !verify_manifest_signature(&manifest, MANIFEST_PUBLIC_KEY) {
        return Err(UpdateError::InvalidSignature);
    }
    
    // 4. Compare versions
    let current_version = env!("CARGO_PKG_VERSION");
    if manifest.version > current_version {
        return Ok(Some(UpdateInfo {
            version: manifest.version,
            is_security: manifest.security_update,
            size: manifest.artifacts[CURRENT_PLATFORM].size,
        }));
    }
    
    Ok(None)
}
```

### Download and Install

```rust
pub async fn download_update(manifest: &UpdateManifest) -> Result<PathBuf, UpdateError> {
    let artifact = &manifest.artifacts[CURRENT_PLATFORM];
    
    // 1. Download binary via Tor
    let binary_bytes = fetch_via_tor(&artifact.url).await?;
    
    // 2. Verify SHA256 hash
    let actual_hash = sha256(&binary_bytes);
    if actual_hash != artifact.sha256 {
        return Err(UpdateError::HashMismatch);
    }
    
    // 3. Download and verify signature
    let sig_bytes = fetch_via_tor(&artifact.sig_url).await?;
    if !verify_binary_signature(&binary_bytes, &sig_bytes, RELEASE_PUBLIC_KEY) {
        return Err(UpdateError::InvalidSignature);
    }
    
    // 4. Save to temp location (in RAM)
    let temp_path = PathBuf::from("/dev/shm/forloop-update.tar.xz");
    std::fs::write(&temp_path, binary_bytes)?;
    
    Ok(temp_path)
}

pub fn install_update(temp_path: &PathBuf) -> Result<(), UpdateError> {
    // 1. Show user what will happen
    println!("Installing update. Browser will restart.");
    
    // 2. Extract to installation directory
    // (Requires appropriate permissions)
    
    // 3. Verify installed binary
    
    // 4. Restart browser
    
    Ok(())
}
```

---

## Offline Updates

For air-gapped or censored environments:

### Export Update Package

```bash
# On a connected machine
forloop --export-update /path/to/forloop-update.tar.xz.signed
```

### Import Update Package

```bash
# On the target machine
forloop --import-update /path/to/forloop-update.tar.xz.signed
```

### Package Format

```
forloop-update.tar.xz.signed:
├── forloop-1.2.3-linux-x86_64.tar.xz
├── forloop-1.2.3-linux-x86_64.tar.xz.sha256
├── forloop-1.2.3-linux-x86_64.tar.xz.sig
└── manifest.json.sig
```

---

## Reproducible Verification

### User Verification Steps

1. Download source for the release version
2. Build using reproducible build system
3. Compare hash with official binary

```bash
# Verify a release
git clone https://github.com/forloop-browser/forloop
cd forloop
git checkout v1.2.3

# Build reproducibly
./build/build.sh --release --reproducible

# Compare hashes
sha256sum dist/forloop-*.tar.xz
# Should match official release hash
```

### Third-Party Verification

- Multiple independent parties build and verify
- Signed attestations published
- Discrepancies trigger security review

---

## Security Considerations

### What Could Go Wrong

| Threat | Mitigation |
|--------|------------|
| Compromised update server | Signatures required, key not on server |
| Man-in-the-middle | Traffic over Tor, hash verification |
| Compromised signing key | Key rotation, HSM storage, multiple signers |
| Malicious update | Reproducible builds allow verification |
| Update timing correlation | Random jitter, user-initiated only |

### Key Compromise Response

If a signing key is compromised:

1. Revoke key immediately via signed revocation message
2. Generate new key pair
3. Sign revocation with master key
4. Publish emergency update with old key + revocation
5. New releases signed with new key only

---

## No Auto-Update

forloop does NOT auto-update because:

1. **Network request reveals activity** - Even with Tor, timing correlates
2. **User should control execution** - Don't run code without consent
3. **Verification should be possible** - User should have time to verify
4. **Availability concerns** - Updates shouldn't break the browser silently

### Notification Only

The browser may display a notification when an update is available,
but will NEVER download or install without explicit user action.

---

## Implementation

### Public Keys (Compiled In)

```rust
// core/config/src/update_keys.rs

/// Ed25519 public key for manifest verification.
/// This key is compiled into the binary and cannot be changed.
pub const MANIFEST_PUBLIC_KEY: &[u8; 32] = &[
    // Key bytes here (example, not real)
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
];

/// Ed25519 public key for binary verification.
pub const RELEASE_PUBLIC_KEY: &[u8; 32] = &[
    // Different key for defense in depth
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
    0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10,
];

/// .onion addresses for update servers.
pub const UPDATE_SERVERS: &[&str] = &[
    "forloopxxxxxx.onion",
    "forloopyyyyyy.onion",  // Mirror
];
```

### CLI Commands

```
forloop --check-updates          # Check for updates
forloop --download-update        # Download update (doesn't install)
forloop --install-update PATH    # Install downloaded update
forloop --export-update PATH     # Export for offline transfer
forloop --import-update PATH     # Import offline update
forloop --verify-update PATH     # Verify update signature
```
