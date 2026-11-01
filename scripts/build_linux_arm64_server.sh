#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_PACKAGE_NAME="rustcraft-server-linux-arm64" \
    "$SCRIPT_DIR/build_server.sh" aarch64-unknown-linux-gnu
