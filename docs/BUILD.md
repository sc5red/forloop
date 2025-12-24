# forloop Build System

## Overview

forloop uses a deterministic, reproducible build system to ensure that:
1. Anyone can verify the binary matches the source
2. No malicious code is introduced during build
3. The build environment is consistent

## Toolchain Requirements

### Linux (Primary Target)

```bash
# Debian/Ubuntu
sudo apt-get install \
    build-essential \
    clang-16 \
    lld-16 \
    llvm-16 \
    rustup \
    python3 \
    python3-pip \
    nodejs \
    npm \
    yasm \
    nasm \
    pkg-config \
    libgtk-4-dev \
    libdbus-1-dev \
    libpulse-dev \
    libasound2-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libx11-dev \
    libxext-dev \
    libxrandr-dev \
    libxcursor-dev \
    libxi-dev \
    libxss-dev \
    libxtst-dev

# Install Rust (specific version)
rustup install 1.75.0
rustup default 1.75.0

# Firefox build dependencies
./mach bootstrap
```

### Version Pinning

All tools are version-pinned for reproducibility:

| Tool | Version | SHA256 |
|------|---------|--------|
| Clang | 16.0.6 | `abc123...` |
| Rust | 1.75.0 | `def456...` |
| Node.js | 20.10.0 | `ghi789...` |
| Python | 3.11.x | (system) |

## Repository Structure

```
forloop/
├── build/
│   ├── build.sh              # Main build script
│   ├── build_config.py       # Build configuration
│   ├── mozconfig             # Firefox build config
│   ├── verify_reproducible.sh # Reproducibility check
│   └── toolchain/
│       ├── fetch_toolchain.sh
│       └── checksums.txt
├── core/
│   ├── fingerprint/          # Rust fingerprint defense
│   └── js_shims/             # JavaScript privacy shims
├── network/                   # Rust network layer
├── sandbox/                   # Rust sandbox implementation
├── patches/                   # Engine patches
├── ui/                        # UI components
└── reproducible/
    ├── Dockerfile            # Reproducible build container
    └── compare_builds.py     # Binary comparison tool
```

## Build Commands

### Quick Build (Development)

```bash
cd build
./build.sh --debug
```

### Release Build (Reproducible)

```bash
cd build
./build.sh --release --reproducible
```

### Build Options

```bash
./build.sh [OPTIONS]

Options:
  --debug           Debug build (faster, not for release)
  --release         Release build (optimized)
  --reproducible    Enable reproducible build mode
  --verify          Verify build reproducibility
  --clean           Clean build directory
  --jobs N          Number of parallel jobs (default: nproc)
  --target TARGET   Build target (linux-x86_64, linux-aarch64)
  --tor-version V   Tor version to embed
```

## Build Configuration

### mozconfig (Firefox Build Config)

```bash
# forloop mozconfig
# Reproducible, privacy-focused build configuration

# Compiler
export CC="clang-16"
export CXX="clang++-16"
export AR="llvm-ar-16"
export NM="llvm-nm-16"
export RANLIB="llvm-ranlib-16"

# Reproducibility
export SOURCE_DATE_EPOCH=1700000000
export PYTHONDONTWRITEBYTECODE=1

# Build type
ac_add_options --enable-release
ac_add_options --enable-optimize="-O2"
ac_add_options --disable-debug
ac_add_options --disable-debug-symbols

# Disable unwanted features
ac_add_options --disable-crashreporter
ac_add_options --disable-telemetry
ac_add_options --disable-updater
ac_add_options --disable-webrtc
ac_add_options --disable-eme
ac_add_options --disable-parental-controls
ac_add_options --disable-accessibility
ac_add_options --disable-necko-wifi
ac_add_options --disable-webspeech
ac_add_options --disable-webspeechtestbackend
ac_add_options --disable-synth-speechd
ac_add_options --disable-dbus
ac_add_options --disable-pulse-audio
ac_add_options --disable-geckodriver

# Security
ac_add_options --enable-hardening
ac_add_options --enable-sandbox

# Application name
ac_add_options --with-app-name=forloop
ac_add_options --with-app-basename=forloop
ac_add_options --with-branding=browser/branding/forloop

# Distribution
ac_add_options --enable-official-branding
ac_add_options --with-distribution-id=org.forloop

# Linking
ac_add_options --enable-lto=thin
ac_add_options --enable-linker=lld

# Strip binaries
ac_add_options --enable-strip
ac_add_options --enable-install-strip
```

## Build Steps

### 1. Fetch and Verify Toolchain

```bash
#!/bin/bash
# build/toolchain/fetch_toolchain.sh

set -euo pipefail

TOOLCHAIN_DIR="$HOME/.forloop-toolchain"
CHECKSUMS_FILE="$(dirname "$0")/checksums.txt"

fetch_and_verify() {
    local name="$1"
    local url="$2"
    local expected_sha256="$3"
    local dest="$TOOLCHAIN_DIR/$name"

    if [[ -f "$dest" ]]; then
        local actual_sha256=$(sha256sum "$dest" | cut -d' ' -f1)
        if [[ "$actual_sha256" == "$expected_sha256" ]]; then
            echo "[OK] $name already downloaded and verified"
            return 0
        fi
    fi

    echo "[FETCH] Downloading $name..."
    curl -L -o "$dest" "$url"

    local actual_sha256=$(sha256sum "$dest" | cut -d' ' -f1)
    if [[ "$actual_sha256" != "$expected_sha256" ]]; then
        echo "[ERROR] SHA256 mismatch for $name"
        echo "  Expected: $expected_sha256"
        echo "  Actual:   $actual_sha256"
        rm -f "$dest"
        exit 1
    fi

    echo "[OK] $name verified"
}

mkdir -p "$TOOLCHAIN_DIR"

# Fetch components
fetch_and_verify "rust-1.75.0.tar.xz" \
    "https://static.rust-lang.org/dist/rust-1.75.0-x86_64-unknown-linux-gnu.tar.xz" \
    "$(grep rust-1.75.0 "$CHECKSUMS_FILE" | cut -d' ' -f1)"
```

### 2. Fetch Firefox Source

```bash
#!/bin/bash
# Part of build.sh

FIREFOX_VERSION="128.0esr"
FIREFOX_SOURCE="https://archive.mozilla.org/pub/firefox/releases/${FIREFOX_VERSION}/source/firefox-${FIREFOX_VERSION}.source.tar.xz"
FIREFOX_SHA256="<known_hash>"

# Download
curl -L -o firefox-source.tar.xz "$FIREFOX_SOURCE"

# Verify
echo "$FIREFOX_SHA256  firefox-source.tar.xz" | sha256sum -c -

# Extract
tar -xf firefox-source.tar.xz
```

### 3. Apply Patches

```bash
#!/bin/bash
# Part of build.sh

PATCHES_DIR="../patches"
SOURCE_DIR="./firefox-source"

cd "$SOURCE_DIR"

for patch in "$PATCHES_DIR"/*.patch; do
    echo "[PATCH] Applying $(basename "$patch")..."
    patch -p1 < "$patch"
done
```

### 4. Build Rust Components

```bash
#!/bin/bash
# Part of build.sh

echo "[BUILD] Building Rust components..."

# Build fingerprint defense
cd ../core/fingerprint
cargo build --release

# Build network layer
cd ../../network
cargo build --release

# Build sandbox
cd ../sandbox
cargo build --release
```

### 5. Build Firefox

```bash
#!/bin/bash
# Part of build.sh

cd firefox-source

# Set reproducible environment
export SOURCE_DATE_EPOCH=$(date -d "2024-01-01" +%s)
export PYTHONDONTWRITEBYTECODE=1
export LC_ALL=C
export TZ=UTC

# Build
./mach build

# Package
./mach package
```

## Reproducible Build Verification

```bash
#!/bin/bash
# build/verify_reproducible.sh

set -euo pipefail

echo "=== Reproducible Build Verification ==="

# Build twice
./build.sh --release --reproducible
mv dist/forloop-*.tar.xz /tmp/forloop-build1.tar.xz

./build.sh --clean
./build.sh --release --reproducible
mv dist/forloop-*.tar.xz /tmp/forloop-build2.tar.xz

# Compare
HASH1=$(sha256sum /tmp/forloop-build1.tar.xz | cut -d' ' -f1)
HASH2=$(sha256sum /tmp/forloop-build2.tar.xz | cut -d' ' -f1)

if [[ "$HASH1" == "$HASH2" ]]; then
    echo "[SUCCESS] Builds are reproducible"
    echo "SHA256: $HASH1"
    exit 0
else
    echo "[FAILURE] Builds differ"
    echo "Build 1: $HASH1"
    echo "Build 2: $HASH2"

    # Detailed diff
    mkdir -p /tmp/build1 /tmp/build2
    tar -xf /tmp/forloop-build1.tar.xz -C /tmp/build1
    tar -xf /tmp/forloop-build2.tar.xz -C /tmp/build2
    diffoscope /tmp/build1 /tmp/build2 || true

    exit 1
fi
```

## Docker Reproducible Build

```dockerfile
# reproducible/Dockerfile

FROM debian:bookworm-slim

# Pin package versions
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential=12.9 \
    clang-16=1:16.0.6-* \
    lld-16=1:16.0.6-* \
    python3=3.11.* \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.75.0
ENV PATH="/root/.cargo/bin:${PATH}"

# Reproducibility settings
ENV SOURCE_DATE_EPOCH=1700000000
ENV PYTHONDONTWRITEBYTECODE=1
ENV LC_ALL=C
ENV TZ=UTC

WORKDIR /build
COPY . /build/

RUN ./build/build.sh --release --reproducible

# Output is in /build/dist/
```

## Main Build Script

```bash
#!/bin/bash
# build/build.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-output"
DIST_DIR="$PROJECT_DIR/dist"

# Defaults
BUILD_TYPE="debug"
REPRODUCIBLE="false"
VERIFY="false"
CLEAN="false"
JOBS=$(nproc)
TARGET="linux-x86_64"
TOR_VERSION="0.4.8.10"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug) BUILD_TYPE="debug" ;;
        --release) BUILD_TYPE="release" ;;
        --reproducible) REPRODUCIBLE="true" ;;
        --verify) VERIFY="true" ;;
        --clean) CLEAN="true" ;;
        --jobs) JOBS="$2"; shift ;;
        --target) TARGET="$2"; shift ;;
        --tor-version) TOR_VERSION="$2"; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
    shift
done

# Clean if requested
if [[ "$CLEAN" == "true" ]]; then
    echo "[CLEAN] Removing build directory..."
    rm -rf "$BUILD_DIR"
fi

mkdir -p "$BUILD_DIR" "$DIST_DIR"

# Set reproducible environment
if [[ "$REPRODUCIBLE" == "true" ]]; then
    export SOURCE_DATE_EPOCH=1700000000
    export PYTHONDONTWRITEBYTECODE=1
    export LC_ALL=C
    export TZ=UTC
fi

echo "=== forloop Build ==="
echo "Type: $BUILD_TYPE"
echo "Target: $TARGET"
echo "Jobs: $JOBS"
echo "Reproducible: $REPRODUCIBLE"
echo ""

# Step 1: Fetch toolchain
echo "[1/6] Fetching toolchain..."
"$SCRIPT_DIR/toolchain/fetch_toolchain.sh"

# Step 2: Fetch Firefox source
echo "[2/6] Fetching Firefox source..."
# ... (implementation above)

# Step 3: Apply patches
echo "[3/6] Applying patches..."
# ... (implementation above)

# Step 4: Build Rust components
echo "[4/6] Building Rust components..."
# ... (implementation above)

# Step 5: Build Firefox
echo "[5/6] Building Firefox..."
cd "$BUILD_DIR/firefox-source"
cp "$SCRIPT_DIR/mozconfig" .mozconfig
./mach build -j"$JOBS"

# Step 6: Package
echo "[6/6] Packaging..."
./mach package
cp dist/*.tar.xz "$DIST_DIR/"

echo ""
echo "=== Build Complete ==="
echo "Output: $DIST_DIR/"

if [[ "$VERIFY" == "true" ]]; then
    "$SCRIPT_DIR/verify_reproducible.sh"
fi
```

## Windows/macOS Notes

### Windows

- Use MSYS2/MinGW-w64 build environment
- Visual Studio 2022 Build Tools required
- Cross-compile from Linux recommended for reproducibility
- AppContainer sandboxing instead of namespaces

### macOS

- Xcode Command Line Tools required
- Cross-compile from Linux for reproducibility
- App Sandbox and Hardened Runtime
- Code signing required for distribution

## Continuous Integration

```yaml
# .github/workflows/build.yml

name: Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build-linux:
    runs-on: ubuntu-22.04
    container:
      image: ghcr.io/forloop-browser/build-env:latest

    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: |
          ./build/build.sh --release --reproducible

      - name: Verify Reproducibility
        run: |
          ./build/verify_reproducible.sh

      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: forloop-linux-x86_64
          path: dist/forloop-*.tar.xz
```
