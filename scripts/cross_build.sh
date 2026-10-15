#!/usr/bin/env bash
# Shared helper for the cross-compilation build scripts (meant to be sourced).
#
# Cross-compiling needs *target* libraries (libudev, alsa, x11) or cross-toolchains
# (MinGW for Windows, osxcross/Xcode for macOS) that host build systems may lack.
# This helper picks a strategy that actually works:
#   1. `cross` + Docker/Podman (uses Cross.toml, installs target libs in container)
#   2. plain `cargo` when a target sysroot / toolchain is available:
#        - PKG_CONFIG_SYSROOT_DIR is set, or
#        - Debian/Ubuntu multiarch libs exist (/usr/lib/<triplet>/pkgconfig/libudev.pc), or
#        - target is Windows and host has MinGW / zig / native Windows environment, or
#        - target is macOS (Apple Darwin) and host is macOS or has osxcross / zig
#   3. otherwise: stop with clear instructions instead of a cryptic crash.
#
# Usage: cross_build <rust-target> [multiarch-triplet]

# Automatically load Rust environment (if present)
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
elif [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# True when `cross`, `rustup`, and a container engine are available.
cross_available() {
    command -v cross &> /dev/null && command -v rustup &> /dev/null && { command -v docker &> /dev/null || command -v podman &> /dev/null; }
}

# Syncs binary from $BINARY_PATH to canonical target/${target}/release/${binary_name}
sync_canonical_binary() {
    local target="$1"
    local bin_name="${2:-${BINARY_NAME:-rustcraft}}"
    local canonical_path="target/${target}/release/${bin_name}"
    if [ -n "$BINARY_PATH" ] && [ -f "$BINARY_PATH" ] && [ "$BINARY_PATH" != "$canonical_path" ]; then
        mkdir -p "$(dirname "$canonical_path")"
        cp -f "$BINARY_PATH" "$canonical_path"
    fi
}

# Safely strips debug symbols from a binary (supports Linux GNU strip and macOS/BSD strip)
strip_binary() {
    local file="$1"
    local custom_strip="${2:-}"

    if [ ! -f "$file" ]; then
        return 0
    fi

    if [ -n "$custom_strip" ] && command -v "$custom_strip" &> /dev/null; then
        echo "==> Running '$custom_strip'..."
        "$custom_strip" --strip-all "$file" 2>/dev/null || "$custom_strip" "$file" 2>/dev/null || true
    elif [[ "$(uname -s)" == "Darwin" ]] || [[ "${target:-}" == *apple-darwin* ]]; then
        if command -v strip &> /dev/null; then
            echo "==> Running 'strip' (macOS)..."
            strip -u -r "$file" 2>/dev/null || strip -x "$file" 2>/dev/null || strip "$file" 2>/dev/null || true
        fi
    elif command -v strip &> /dev/null; then
        echo "==> Running 'strip'..."
        strip --strip-all "$file" 2>/dev/null || strip "$file" 2>/dev/null || true
    fi
}

cross_build() {
    local target="$1"
    local triplet="${2:-}"

    # macOS (Apple Darwin) targets
    # Note: `cross` does not provide built-in Docker images for Apple Darwin targets,
    # and falls back to host cargo which fails on non-macOS hosts.
    # Therefore Darwin targets must be handled explicitly:
    if [[ "$target" == *apple-darwin* || "$target" == *darwin* ]]; then
        # Native build or cross-architecture build (x86_64 <-> aarch64) on macOS host
        if [[ "$(uname -s)" == "Darwin" ]]; then
            echo "==> Using plain cargo on macOS for $target"
            cargo build --release --target "$target"
            return
        fi

        # Cross-compiling for macOS on non-Darwin host:
        # Check for osxcross or zig
        local osxcross_linker=""
        if [[ "$target" == aarch64* ]] && command -v aarch64-apple-darwin-clang &> /dev/null; then
            osxcross_linker="aarch64-apple-darwin-clang"
        elif [[ "$target" == aarch64* ]] && command -v oa64-clang &> /dev/null; then
            osxcross_linker="oa64-clang"
        elif [[ "$target" == x86_64* ]] && command -v x86_64-apple-darwin-clang &> /dev/null; then
            osxcross_linker="x86_64-apple-darwin-clang"
        elif [[ "$target" == x86_64* ]] && command -v o64-clang &> /dev/null; then
            osxcross_linker="o64-clang"
        elif command -v zig &> /dev/null; then
            osxcross_linker="zig"
        fi

        if [ -n "$osxcross_linker" ]; then
            echo "==> Using cross-compiler ($osxcross_linker) for $target"
            cargo build --release --target "$target"
            return
        fi

        echo "==> ERROR: unable to build for macOS target $target on this Linux host."
        echo "    Reason: Official macOS SDK & toolchain (Objective-C/Cocoa) are not available on this host."
        echo "    Fix (recommended): build natively on macOS with Xcode Command Line Tools:"
        echo "      xcode-select --install"
        echo "      rustup target add $target"
        echo "      ./build_macos.sh"
        echo "    Alternative (Linux cross-compilation):"
        echo "      - Install osxcross (https://github.com/tpoechtrager/osxcross) and set target linker"
        echo "      - Or configure a custom macOS cross-image in Cross.toml"
        return 1
    fi

    if cross_available; then
        # Container images have their own glibc and cannot run host-built build scripts
        # cached in target/release, so use a dedicated target dir for container builds.
        export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target/cross-${target}}"
        BINARY_PATH="${CARGO_TARGET_DIR}/${target}/release/${BINARY_NAME}"
        echo "==> Using 'cross' (container-based cross-compilation) for $target"
        cross build --release --target "$target"
        sync_canonical_binary "$target"
        return
    fi

    # Windows targets cross-compilation without container
    if [[ "$target" == *windows* ]]; then
        if [[ "$(uname -s)" =~ (MINGW|MSYS|CYGWIN|Windows_NT) ]] \
            || command -v x86_64-w64-mingw32-gcc &> /dev/null \
            || command -v i686-w64-mingw32-gcc &> /dev/null \
            || command -v zig &> /dev/null; then
            echo "==> Using plain cargo for $target"
            cargo build --release --target "$target"
            return
        fi

        echo "==> ERROR: no cross-compiler found for Windows target $target."
        echo "    Fix (recommended): install Docker or Podman, then:  cargo install cross"
        echo "      and re-run this script - it will cross-compile for Windows via container."
        echo "    Alternative (Linux/macOS): install MinGW (e.g. gcc-mingw-w64, mingw64-gcc, or brew install mingw-w64)."
        return 1
    fi

    if [ -n "${PKG_CONFIG_SYSROOT_DIR:-}" ] \
        || { [ -n "$triplet" ] && [ -f "/usr/lib/$triplet/pkgconfig/libudev.pc" ]; }; then
        if [ -n "$triplet" ] && [ -z "${PKG_CONFIG_SYSROOT_DIR:-}" ]; then
            export PKG_CONFIG_PATH="/usr/lib/$triplet/pkgconfig:${PKG_CONFIG_PATH:-}"
        fi
        export PKG_CONFIG_ALLOW_CROSS=1
        echo "==> Using plain cargo with the target sysroot / multiarch libraries"
        cargo build --release --target "$target"
        return
    fi

    echo "==> ERROR: no way to obtain target libraries (libudev, alsa, ...) for $target."
    echo "    pkg-config on this host only sees host libraries, so the build would fail."
    echo "    Fix (recommended): install Docker or Podman, then:  cargo install cross"
    echo "      and re-run this script - it will use 'cross' + Cross.toml automatically."
    if [ -n "$triplet" ]; then
        echo "    Alternative (Debian/Ubuntu multiarch): install the target dev libraries, e.g."
        echo "      sudo dpkg --add-architecture <arch> && sudo apt update"
        echo "      sudo apt install libudev-dev:<arch> libasound2-dev:<arch> libx11-dev:<arch>"
    fi
    echo "    Alternative: set PKG_CONFIG_SYSROOT_DIR / PKG_CONFIG_PATH to a target sysroot."
    return 1
}
