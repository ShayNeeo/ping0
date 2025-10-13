#!/usr/bin/env bash
set -euo pipefail

# Small deploy helper: build release, copy binary to /opt/ping0, set ownership, restart systemd
# Usage: sudo ./deploy/install.sh   (script will use sudo for operations when needed)

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
echo "Repo root: $ROOT_DIR"

# Build as the non-root repo owner when possible
if [ -f "$ROOT_DIR/.git" ]; then
  REPO_OWNER=$(stat -c '%U' "$ROOT_DIR")
else
  REPO_OWNER=$(whoami)
fi

echo "Building release (as user: $REPO_OWNER)"
if [ "$(id -u)" -eq 0 ]; then
  # if running as root, drop to repo owner if possible
  if [ "$REPO_OWNER" != "root" ]; then
    sudo -u "$REPO_OWNER" -H bash -c "cd '$ROOT_DIR' && cargo build --release"
  else
    (cd "$ROOT_DIR" && cargo build --release)
  fi
else
  (cd "$ROOT_DIR" && cargo build --release)
fi

BIN_PATH="$ROOT_DIR/target/release/ping0-server"
if [ ! -f "$BIN_PATH" ]; then
  echo "ERROR: build failed, binary not found at $BIN_PATH" >&2
  exit 2
fi

echo "Installing binary to /opt/ping0/ping0-server"
sudo mkdir -p /opt/ping0
sudo cp -f "$BIN_PATH" /opt/ping0/ping0-server
sudo chown root:ping0 /opt/ping0/ping0-server || true
sudo chmod 750 /opt/ping0/ping0-server

echo "Ensuring upload dir ownership"
sudo mkdir -p /var/lib/ping0/uploads
sudo chown -R ping0:ping0 /var/lib/ping0 || true
sudo chmod 750 /var/lib/ping0 || true

echo "Reloading systemd and restarting ping0 service"
sudo systemctl daemon-reload
sudo systemctl restart ping0
sudo systemctl status ping0 --no-pager -l || true

echo "Done. To follow logs: sudo journalctl -u ping0 -f"
