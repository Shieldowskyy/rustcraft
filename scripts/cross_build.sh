#!/usr/bin/env bash
# Shared helper for the cross-compilation build scripts (meant to be sourced).
#
# Cross-compiling needs *target* libraries (libudev, alsa, x11) that the host
# pkg-config can't see. This helper picks a strategy that actually works:
#   1. `cross` + Docker/Podman (uses Cross.toml, installs target libs in the image)
#   2. plain `cargo` when a target sysroot is available:
#        - PKG_CONFIG_SYSROOT_DIR is set, or
#        - Debian/Ubuntu multiarch libs exist (/usr/lib/<triplet>/pkgconfig/libudev.pc)
#   3. otherwise: stop with clear instructions instead of a cryptic pkg-config panic.
#
# Usage: cross_build <rust-target> [multiarch-triplet]

cross_build() {
    local target="$1"
    local triplet="${2:-}"

    if command -v cross &> /dev/null && { command -v docker &> /dev/null || command -v podman &> /dev/null; }; then
        echo "==> Using 'cross' (container-based cross-compilation) for $target"
        cross build --release --target "$target"
        return
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
