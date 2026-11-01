#!/usr/bin/env bash
set -Eeuo pipefail

SERVICE_NAME="rustcraft-relay"
SERVICE_USER="rustcraft-relay"
INSTALL_DIR="/usr/local/bin"
INSTALL_BINARY="$INSTALL_DIR/rustcraft-server"
DATA_DIR="/var/lib/rustcraft-relay"
DEFAULT_PORT="26657"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_BINARY="$SCRIPT_DIR/rustcraft-server"

usage() {
    cat <<EOF
Usage: sudo $0 [port]

Install rustcraft-server from the same directory as this script and run it
as a relay service. The default relay port is $DEFAULT_PORT.

Supported service managers: systemd and OpenRC.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if [[ "$#" -gt 1 ]]; then
    usage >&2
    exit 2
fi

PORT="${1:-$DEFAULT_PORT}"
if [[ ! "$PORT" =~ ^[0-9]{1,5}$ ]] || ((10#$PORT < 1 || 10#$PORT > 65535)); then
    echo "Error: port must be a number from 1 to 65535." >&2
    exit 2
fi

if ((EUID != 0)); then
    echo "Error: run this installer as root (for example: sudo $0 $PORT)." >&2
    exit 1
fi

if [[ ! -f "$SOURCE_BINARY" || ! -x "$SOURCE_BINARY" ]]; then
    echo "Error: executable rustcraft-server must be in the same directory as this script:" >&2
    echo "  $SOURCE_BINARY" >&2
    exit 1
fi

if ! command -v install >/dev/null 2>&1; then
    echo "Error: the 'install' utility is required." >&2
    exit 1
fi

NOLOGIN="/usr/sbin/nologin"
if [[ ! -x "$NOLOGIN" ]]; then
    NOLOGIN="/sbin/nologin"
fi
if [[ ! -x "$NOLOGIN" ]]; then
    NOLOGIN="/bin/false"
fi

SERVICE_MANAGER=""
if command -v systemctl >/dev/null 2>&1 && [[ -d /run/systemd/system ]]; then
    SERVICE_MANAGER="systemd"
elif command -v rc-update >/dev/null 2>&1 && command -v rc-service >/dev/null 2>&1; then
    SERVICE_MANAGER="openrc"
else
    echo "Error: no active systemd or OpenRC service manager was found." >&2
    echo "Run this inside a systemd/OpenRC LXC container, or start its init system first." >&2
    exit 1
fi

if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
    if command -v useradd >/dev/null 2>&1; then
        useradd --system --no-create-home --shell "$NOLOGIN" "$SERVICE_USER"
    elif command -v adduser >/dev/null 2>&1; then
        adduser -S -D -H -s "$NOLOGIN" "$SERVICE_USER"
    else
        echo "Error: cannot create the restricted '$SERVICE_USER' service account." >&2
        exit 1
    fi
fi

SERVICE_GROUP="$(id -gn "$SERVICE_USER")"
install -d -o "$SERVICE_USER" -g "$SERVICE_GROUP" -m 0750 "$DATA_DIR"
install -o root -g root -m 0755 "$SOURCE_BINARY" "$INSTALL_BINARY"

if [[ "$SERVICE_MANAGER" == "systemd" ]]; then
    UNIT_PATH="/etc/systemd/system/$SERVICE_NAME.service"
    install -d -m 0755 /etc/systemd/system
    cat > "$UNIT_PATH" <<EOF
[Unit]
Description=RustCraft TCP relay
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
User=$SERVICE_USER
WorkingDirectory=$DATA_DIR
ExecStart=$INSTALL_BINARY --relay-server --relay-port $PORT
Restart=on-failure
RestartSec=3
NoNewPrivileges=true
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
    chmod 0644 "$UNIT_PATH"
    systemctl daemon-reload
    systemctl enable "$SERVICE_NAME.service"
    systemctl restart "$SERVICE_NAME.service"
    echo "Installed and started $SERVICE_NAME.service on TCP port $PORT."
    echo "Status: systemctl status $SERVICE_NAME.service"
    echo "Logs:   journalctl -u $SERVICE_NAME.service -f"
elif [[ "$SERVICE_MANAGER" == "openrc" ]]; then
    INIT_PATH="/etc/init.d/$SERVICE_NAME"
    install -d -m 0755 /etc/init.d
    cat > "$INIT_PATH" <<EOF
#!/sbin/openrc-run
description="RustCraft TCP relay"
command="$INSTALL_BINARY"
command_args="--relay-server --relay-port $PORT"
command_user="$SERVICE_USER"
command_background=true
pidfile="/run/\${RC_SVCNAME}.pid"
directory="$DATA_DIR"

depend() {
    need net
}
EOF
    chmod 0755 "$INIT_PATH"
    rc-update add "$SERVICE_NAME" default
    if rc-service "$SERVICE_NAME" status >/dev/null 2>&1; then
        rc-service "$SERVICE_NAME" restart
    else
        rc-service "$SERVICE_NAME" start
    fi
    echo "Installed and started the OpenRC service $SERVICE_NAME on TCP port $PORT."
    echo "Status: rc-service $SERVICE_NAME status"
fi

echo "Allow inbound TCP port $PORT in the host firewall and, for LXC, its network policy."
