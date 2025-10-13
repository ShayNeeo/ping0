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
# Ensure the directory and binary are owned so the `ping0` user can execute
sudo chown root:ping0 /opt/ping0
sudo chmod 750 /opt/ping0
sudo chown root:ping0 /opt/ping0/ping0-server || true
sudo chmod 750 /opt/ping0/ping0-server

echo "Ensuring upload dir ownership"
sudo mkdir -p /var/lib/ping0/uploads
sudo chown -R ping0:ping0 /var/lib/ping0 || true
sudo chmod 750 /var/lib/ping0 || true

### Nginx site setup (optional)
# You can override the server name by passing NGINX_SERVER_NAME environment variable.
NGINX_SERVER_NAME=${NGINX_SERVER_NAME:-api.0.id.vn}
NGINX_SITE_PATH="/etc/nginx/sites-available/ping0"

echo "Generating nginx site config for $NGINX_SERVER_NAME"
sudo tee "$NGINX_SITE_PATH" > /dev/null <<NGX
server {
    listen 80;
    server_name ${NGINX_SERVER_NAME};
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl;
    http2 on;
    server_name ${NGINX_SERVER_NAME};

    ssl_certificate /etc/ssl/cf_origin/${NGINX_SERVER_NAME}.crt;
    ssl_certificate_key /etc/ssl/cf_origin/${NGINX_SERVER_NAME}.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;

    client_max_body_size 12M;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 90s;
    }
}
NGX

echo "Enabling nginx site"
sudo ln -sf "$NGINX_SITE_PATH" /etc/nginx/sites-enabled/ping0

if [ -f "/etc/ssl/cf_origin/${NGINX_SERVER_NAME}.crt" ]; then
  echo "Found origin cert for ${NGINX_SERVER_NAME}"
else
  echo "Warning: origin cert /etc/ssl/cf_origin/${NGINX_SERVER_NAME}.crt not found. Place your Cloudflare Origin certificate at that path or adjust nginx config." >&2
fi

echo "Testing nginx configuration"
sudo nginx -t || true

echo "Reloading systemd and restarting ping0 service"
sudo systemctl daemon-reload
sudo systemctl restart ping0
sudo systemctl status ping0 --no-pager -l || true

echo "Reloading nginx (if installed)"
if command -v nginx >/dev/null 2>&1; then
  sudo systemctl reload nginx || sudo systemctl restart nginx || true
fi

echo "Done. To follow logs: sudo journalctl -u ping0 -f"
