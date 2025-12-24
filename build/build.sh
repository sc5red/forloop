#!/bin/bash
# forloop Browser Build Script
# Deterministic, reproducible build for maximum security

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-output"
DIST_DIR="$PROJECT_DIR/dist"

# Versions (pinned for reproducibility)
FIREFOX_VERSION="128.0esr"
RUST_VERSION="1.75.0"
TOR_VERSION="0.4.8.10"
ARTI_VERSION="1.1.12"

# Defaults
BUILD_TYPE="debug"
REPRODUCIBLE="false"
VERIFY="false"
CLEAN="false"
JOBS=$(nproc 2>/dev/null || echo 4)
TARGET="linux-x86_64"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Build the forloop privacy browser.

Options:
  --debug           Debug build (faster, not for release)
  --release         Release build (optimized, for distribution)
  --reproducible    Enable reproducible build mode
  --verify          Verify build reproducibility after completion
  --clean           Clean build directory before building
  --jobs N          Number of parallel jobs (default: $JOBS)
  --target TARGET   Build target (linux-x86_64, linux-aarch64)
  --help            Show this help message

Examples:
  $0 --debug                    # Quick development build
  $0 --release --reproducible   # Production build
  $0 --clean --release          # Clean release build
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            BUILD_TYPE="debug"
            ;;
        --release)
            BUILD_TYPE="release"
            ;;
        --reproducible)
            REPRODUCIBLE="true"
            ;;
        --verify)
            VERIFY="true"
            ;;
        --clean)
            CLEAN="true"
            ;;
        --jobs)
            JOBS="$2"
            shift
            ;;
        --target)
            TARGET="$2"
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
    shift
done

# Set reproducible environment
setup_reproducible_env() {
    if [[ "$REPRODUCIBLE" == "true" ]]; then
        log_info "Setting up reproducible build environment"
        export SOURCE_DATE_EPOCH=1700000000
        export PYTHONDONTWRITEBYTECODE=1
        export LC_ALL=C
        export TZ=UTC
        export LANG=C
        
        # Disable ccache for reproducibility
        export CCACHE_DISABLE=1
    fi
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    for cmd in clang-16 lld-16 python3 cargo rustc node npm; do
        if ! command -v "$cmd" &> /dev/null; then
            missing+=("$cmd")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing[*]}"
        log_error "Please install them before building"
        exit 1
    fi
    
    # Check Rust version
    local rust_ver=$(rustc --version | cut -d' ' -f2)
    if [[ "$rust_ver" != "$RUST_VERSION" ]]; then
        log_warn "Rust version mismatch: expected $RUST_VERSION, got $rust_ver"
        if [[ "$REPRODUCIBLE" == "true" ]]; then
            log_error "Reproducible build requires exact Rust version"
            exit 1
        fi
    fi
    
    log_info "All dependencies satisfied"
}

# Clean build directory
clean_build() {
    if [[ "$CLEAN" == "true" ]]; then
        log_info "Cleaning build directory..."
        rm -rf "$BUILD_DIR"
    fi
    
    mkdir -p "$BUILD_DIR" "$DIST_DIR"
}

# Build Rust components
build_rust_components() {
    log_info "Building Rust components..."
    
    local cargo_flags=""
    if [[ "$BUILD_TYPE" == "release" ]]; then
        cargo_flags="--release"
    fi
    
    # Build fingerprint defense
    log_info "  - forloop-fingerprint"
    (cd "$PROJECT_DIR/core/fingerprint" && cargo build $cargo_flags)
    
    # Build network layer
    log_info "  - forloop-network"
    (cd "$PROJECT_DIR/network" && cargo build $cargo_flags)
    
    # Build sandbox
    log_info "  - forloop-sandbox"
    (cd "$PROJECT_DIR/sandbox" && cargo build $cargo_flags)
}

# Fetch Firefox source
fetch_firefox_source() {
    local source_dir="$BUILD_DIR/firefox-$FIREFOX_VERSION"
    
    if [[ -d "$source_dir" ]]; then
        log_info "Firefox source already present"
        return 0
    fi
    
    log_info "Fetching Firefox $FIREFOX_VERSION source..."
    
    local url="https://archive.mozilla.org/pub/firefox/releases/${FIREFOX_VERSION}/source/firefox-${FIREFOX_VERSION}.source.tar.xz"
    local tarball="$BUILD_DIR/firefox-source.tar.xz"
    
    curl -L -o "$tarball" "$url"
    
    log_info "Extracting Firefox source..."
    tar -xf "$tarball" -C "$BUILD_DIR"
    rm "$tarball"
}

# Apply patches
apply_patches() {
    log_info "Applying forloop patches..."
    
    local source_dir="$BUILD_DIR/firefox-$FIREFOX_VERSION"
    local patches_dir="$PROJECT_DIR/patches"
    
    if [[ ! -d "$patches_dir" ]]; then
        log_warn "No patches directory found"
        return 0
    fi
    
    cd "$source_dir"
    
    for patch in "$patches_dir"/*.patch; do
        if [[ -f "$patch" ]]; then
            log_info "  - $(basename "$patch")"
            patch -p1 < "$patch" || {
                log_error "Failed to apply patch: $(basename "$patch")"
                exit 1
            }
        fi
    done
}

# Generate mozconfig
generate_mozconfig() {
    log_info "Generating mozconfig..."
    
    local source_dir="$BUILD_DIR/firefox-$FIREFOX_VERSION"
    local mozconfig="$source_dir/.mozconfig"
    
    cat > "$mozconfig" << 'EOF'
# forloop mozconfig - Privacy-focused build configuration

# Compiler
export CC="clang-16"
export CXX="clang++-16"
export AR="llvm-ar-16"
export NM="llvm-nm-16"
export RANLIB="llvm-ranlib-16"

# Build type
ac_add_options --enable-optimize="-O2"
ac_add_options --enable-release

# Disable telemetry and crash reporting
ac_add_options --disable-crashreporter
ac_add_options --disable-updater
ac_add_options --disable-telemetry
ac_add_options --disable-normandy

# Disable tracking features
ac_add_options --disable-webrtc
ac_add_options --disable-eme
ac_add_options --disable-parental-controls

# Disable unnecessary features
ac_add_options --disable-accessibility
ac_add_options --disable-webspeech
ac_add_options --disable-webspeechtestbackend
ac_add_options --disable-synth-speechd
ac_add_options --disable-geckodriver

# Security hardening
ac_add_options --enable-hardening
ac_add_options --enable-sandbox

# Application branding
ac_add_options --with-app-name=forloop
ac_add_options --with-app-basename=forloop

# Linking
ac_add_options --enable-linker=lld
ac_add_options --enable-lto=thin

# Strip for release
ac_add_options --enable-strip
ac_add_options --enable-install-strip
EOF
    
    if [[ "$BUILD_TYPE" == "debug" ]]; then
        cat >> "$mozconfig" << 'EOF'

# Debug settings
ac_add_options --enable-debug
ac_add_options --enable-debug-symbols
ac_add_options --disable-optimize
EOF
    fi
}

# Build Firefox
build_firefox() {
    log_info "Building Firefox with forloop modifications..."
    
    local source_dir="$BUILD_DIR/firefox-$FIREFOX_VERSION"
    cd "$source_dir"
    
    ./mach build -j"$JOBS"
}

# Package
package_build() {
    log_info "Packaging forloop..."
    
    local source_dir="$BUILD_DIR/firefox-$FIREFOX_VERSION"
    cd "$source_dir"
    
    ./mach package
    
    # Copy to dist
    cp dist/*.tar.* "$DIST_DIR/" 2>/dev/null || true
    
    log_info "Package created in $DIST_DIR/"
}

# Main build
main() {
    echo "============================================"
    echo "           forloop Browser Build           "
    echo "============================================"
    echo ""
    echo "Build type:    $BUILD_TYPE"
    echo "Target:        $TARGET"
    echo "Jobs:          $JOBS"
    echo "Reproducible:  $REPRODUCIBLE"
    echo ""
    
    setup_reproducible_env
    check_dependencies
    clean_build
    
    log_info "[1/6] Building Rust components..."
    build_rust_components
    
    log_info "[2/6] Fetching Firefox source..."
    fetch_firefox_source
    
    log_info "[3/6] Applying patches..."
    apply_patches
    
    log_info "[4/6] Generating build configuration..."
    generate_mozconfig
    
    log_info "[5/6] Building Firefox..."
    build_firefox
    
    log_info "[6/6] Packaging..."
    package_build
    
    echo ""
    echo "============================================"
    log_info "Build completed successfully!"
    echo "============================================"
    
    if [[ "$VERIFY" == "true" ]]; then
        log_info "Verifying reproducibility..."
        "$SCRIPT_DIR/verify_reproducible.sh"
    fi
}

main
