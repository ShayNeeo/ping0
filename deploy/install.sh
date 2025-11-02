#!/usr/bin/env bash
set -euo pipefail

# Idempotent installer for ping0. It will:
# - Install required packages (Debian/Ubuntu)
# - Create system user and runtime dirs
# - Ensure Rust toolchain for the repo owner
# - Build release binary
# - Install to /opt/ping0/ping0
# - Write systemd unit and enable service
# - Optionally configure nginx and Cloudflare Origin certs
# Usage: sudo ./deploy/install.sh

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
echo "Repo root: $ROOT_DIR"

# Configuration (override via environment variables)
SERVICE_NAME=${SERVICE_NAME:-ping0}
BIN_NAME=${BIN_NAME:-ping0}
INSTALL_DIR=${INSTALL_DIR:-/opt/ping0}
SERVICE_USER=${SERVICE_USER:-ping0}
SERVICE_GROUP=${SERVICE_GROUP:-$SERVICE_USER}
DATA_DIR=${DATA_DIR:-$INSTALL_DIR/data}
UPLOADS_DIR=${UPLOADS_DIR:-$INSTALL_DIR/uploads}
ENV_FILE=${ENV_FILE:-/etc/default/$SERVICE_NAME}
SYSTEMD_UNIT=${SYSTEMD_UNIT:-/etc/systemd/system/$SERVICE_NAME.service}
CF_SSL_DIR=${CF_SSL_DIR:-/etc/ssl/cf_origin}
NGINX_SERVER_NAME=${NGINX_SERVER_NAME:-0.id.vn}
NGINX_SITE_PATH=${NGINX_SITE_PATH:-/etc/nginx/sites-available/$SERVICE_NAME}
APP_PORT=${APP_PORT:-10105}
ENV_OVERWRITE=${ENV_OVERWRITE:-1}

# API/TLS settings
API_DOMAIN=${API_DOMAIN:-$NGINX_SERVER_NAME}
# The public domain for short links (can differ from API_DOMAIN if behind reverse proxy)
BASE_URL=${BASE_URL:-https://$API_DOMAIN}
# Ensure BASE_URL has a protocol
if [[ ! "$BASE_URL" =~ ^https?:// ]]; then
  BASE_URL="https://$BASE_URL"
fi
# Whether the API DNS is proxied behind Cloudflare (orange cloud)
# If enabled and CERTBOT is enabled, we will use DNS-01 with Cloudflare.
PROXIED_API=${PROXIED_API:-0}
CERTBOT_ENABLE=${CERTBOT_ENABLE:-1}
CERTBOT_EMAIL=${CERTBOT_EMAIL:-}
# Cloudflare API token for DNS-01 (scoped to zone DNS edit)
CF_API_TOKEN=${CF_API_TOKEN:-}
CERTBOT_DNS_PROPAGATION=${CERTBOT_DNS_PROPAGATION:-60}

# Feature flags (1/true/yes to enable)
APT_INSTALL=${APT_INSTALL:-1}
SYSTEMD_ENABLE=${SYSTEMD_ENABLE:-1}
NGINX_ENABLE=${NGINX_ENABLE:-1}

# Build selection: auto | root | server, or provide BINARY_PATH
BUILD_SOURCE=${BUILD_SOURCE:-auto}
BINARY_PATH=${BINARY_PATH:-}

is_enabled() {
  case "${1:-}" in
    1|true|TRUE|yes|YES|y|Y|on|ON|enable|enabled) return 0 ;;
    *) return 1 ;;
  esac
}

# Choose a non-root user for building when possible
if [ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != "root" ]; then
  BUILD_USER="$SUDO_USER"
elif [ -f "$ROOT_DIR/.git" ]; then
  BUILD_USER=$(stat -c '%U' "$ROOT_DIR")
else
  BUILD_USER=$(whoami)
fi

echo "Building release (as user: $BUILD_USER)"
## Auto-detect OS and install packages (Debian/apt)
if [ -f /etc/debian_version ]; then
  if is_enabled "$APT_INSTALL"; then
    echo "Detected Debian-based OS. Installing system packages via apt..."
    APT_PKGS="build-essential pkg-config libsqlite3-dev ca-certificates curl git"
    if is_enabled "$NGINX_ENABLE"; then
      APT_PKGS="$APT_PKGS nginx ufw certbot python3-certbot-nginx python3-certbot-dns-cloudflare"
    fi
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends $APT_PKGS || true
  else
    echo "APT_INSTALL disabled; skipping apt package installation."
  fi
else
  echo "Non-Debian OS detected (or /etc/debian_version missing). Skipping package install step."
fi

# Ensure a system user exists (service user)
if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
  echo "Creating system user '$SERVICE_USER'"
  sudo useradd --system --create-home --home-dir "$INSTALL_DIR" --shell /usr/sbin/nologin "$SERVICE_USER" || true
fi

# Ensure rustup/cargo is available for the build user; install rustup if missing
if [ "$BUILD_USER" = "root" ]; then
  if ! bash -lc 'source "$HOME/.cargo/env" 2>/dev/null || true; command -v cargo >/dev/null 2>&1'; then
    echo "Installing rustup for user root"
    bash -lc 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
  fi
else
  if ! sudo -u "$BUILD_USER" -H bash -lc 'source "$HOME/.cargo/env" 2>/dev/null || true; command -v cargo >/dev/null 2>&1'; then
    echo "Installing rustup for user $BUILD_USER"
    sudo -u "$BUILD_USER" -H bash -lc 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
  fi
fi

echo "Building release (as user: $BUILD_USER)"
if [ "$(id -u)" -eq 0 ] && [ "$BUILD_USER" != "root" ]; then
  sudo -u "$BUILD_USER" -H bash -lc "source \"\$HOME/.cargo/env\" 2>/dev/null || true; export PATH=\"\$HOME/.cargo/bin:\$PATH\"; cd '$ROOT_DIR' && cargo build --release"
else
  bash -lc "source \"\$HOME/.cargo/env\" 2>/dev/null || true; export PATH=\"\$HOME/.cargo/bin:\$PATH\"; cd '$ROOT_DIR' && cargo build --release"
fi

# Determine which binary to install
BIN_TARGET="$INSTALL_DIR/$BIN_NAME"
if [ -n "$BINARY_PATH" ]; then
  BIN_PATH="$BINARY_PATH"
  echo "Using provided BINARY_PATH: $BIN_PATH"
else
  case "$BUILD_SOURCE" in
    server)
      CANDIDATES=("$ROOT_DIR/server/target/release/$BIN_NAME")
      ;;
    root)
      CANDIDATES=("$ROOT_DIR/target/release/$BIN_NAME")
      ;;
    *)
      CANDIDATES=(
        "$ROOT_DIR/target/release/$BIN_NAME"
        "$ROOT_DIR/server/target/release/$BIN_NAME"
      )
      ;;
  esac
  BIN_PATH=""
  for c in "${CANDIDATES[@]}"; do
    if [ -f "$c" ]; then
      BIN_PATH="$c"
      break
    fi
  done
fi

if [ -z "${BIN_PATH:-}" ] || [ ! -f "$BIN_PATH" ]; then
  echo "ERROR: build failed, binary not found at expected locations:" >&2
  echo "  - $ROOT_DIR/target/release/$BIN_NAME" >&2
  echo "  - $ROOT_DIR/server/target/release/$BIN_NAME" >&2
  exit 2
fi

echo "Installing binary to $BIN_TARGET"
sudo mkdir -p "$INSTALL_DIR"
sudo cp -f "$BIN_PATH" "$BIN_TARGET"
# Ensure the directory and binary are owned so the service user can execute
sudo chown root:"$SERVICE_GROUP" "$INSTALL_DIR"
sudo chmod 750 "$INSTALL_DIR"
sudo chown root:"$SERVICE_GROUP" "$BIN_TARGET" || true
sudo chmod 750 "$BIN_TARGET"
sudo mkdir -p "$UPLOADS_DIR" "$DATA_DIR"
sudo chown -R "$SERVICE_USER":"$SERVICE_GROUP" "$UPLOADS_DIR" "$DATA_DIR"
sudo chmod 750 "$UPLOADS_DIR" "$DATA_DIR"

# Ensure SSL dir exists if nginx is enabled
if is_enabled "$NGINX_ENABLE"; then
  echo "Ensuring SSL dir exists at $CF_SSL_DIR"
  sudo mkdir -p "$CF_SSL_DIR"
  sudo chmod 700 "$CF_SSL_DIR"
fi

# Obtain Let's Encrypt certificate (optional, before nginx site is active)
obtain_certbot_cert() {
  local domain="$1"
  local email="$2"
  local proxied="$3"
  local have_cert=0
  if [ -f "/etc/letsencrypt/live/$domain/fullchain.pem" ] && [ -f "/etc/letsencrypt/live/$domain/privkey.pem" ]; then
    have_cert=1
  fi
  if [ $have_cert -eq 1 ]; then
    echo "Let's Encrypt cert already present for $domain"
    return 0
  fi
  if ! is_enabled "$CERTBOT_ENABLE"; then
    echo "CERTBOT_ENABLE=0; skipping Let's Encrypt issuance"
    return 0
  fi
  if [ -z "$email" ]; then
    echo "WARN: CERTBOT_EMAIL is empty; skipping Let's Encrypt issuance"
    return 0
  fi
  echo "Attempting Let's Encrypt issuance for $domain"
  if is_enabled "$proxied"; then
    if [ -z "$CF_API_TOKEN" ]; then
      echo "WARN: PROXIED_API=1 but CF_API_TOKEN not set; cannot perform DNS-01. Skipping LE issuance."
      return 0
    fi
    echo "Using DNS-01 with Cloudflare for $domain"
    sudo mkdir -p /root/.secrets/certbot
    local cf_ini="/root/.secrets/certbot/cloudflare.ini"
    sudo bash -c "umask 077 && echo 'dns_cloudflare_api_token=$CF_API_TOKEN' > '$cf_ini'"
    sudo certbot certonly \
      --non-interactive --agree-tos -m "$email" \
      --dns-cloudflare --dns-cloudflare-credentials "$cf_ini" \
      --dns-cloudflare-propagation-seconds "$CERTBOT_DNS_PROPAGATION" \
      -d "$domain" || true
  else
    echo "Using standalone HTTP-01 for $domain (temporarily binding :80)"
    # Try to stop nginx if it is running
    if command -v nginx >/dev/null 2>&1; then
      sudo systemctl stop nginx || true
    fi
    sudo certbot certonly --standalone \
      --non-interactive --agree-tos -m "$email" \
      --preferred-challenges http \
      -d "$domain" || true
  fi
}

# Write (or update) env file for systemd
write_env_file() {
  sudo tee "$ENV_FILE" > /dev/null <<ENVV
HOST=0.0.0.0
PORT=$APP_PORT
BASE_URL=$BASE_URL
DATABASE_PATH=$DATA_DIR/ping0.db
ENVV
}

if [ ! -f "$ENV_FILE" ]; then
  echo "Writing $ENV_FILE"
  write_env_file
elif is_enabled "$ENV_OVERWRITE"; then
  echo "Updating $ENV_FILE (ENV_OVERWRITE enabled)"
  sudo cp "$ENV_FILE" "${ENV_FILE}.bak.$(date +%s)" || true
  write_env_file
else
  echo "Existing $ENV_FILE detected; ENV_OVERWRITE disabled, skipping update"
fi

# Systemd unit
if is_enabled "$SYSTEMD_ENABLE"; then
  echo "Writing systemd unit $SYSTEMD_UNIT"
  sudo tee "$SYSTEMD_UNIT" > /dev/null <<UNIT
[Unit]
Description=$SERVICE_NAME - Rust link & file sharer
After=network.target

[Service]
Type=simple
EnvironmentFile=-$ENV_FILE
WorkingDirectory=$INSTALL_DIR
ExecStart=$BIN_TARGET
User=$SERVICE_USER
Group=$SERVICE_GROUP
Restart=always
RestartSec=2
NoNewPrivileges=yes

[Install]
WantedBy=multi-user.target
UNIT

  echo "Reloading systemd and enabling service"
  sudo systemctl daemon-reload
  sudo systemctl enable "$SERVICE_NAME" || true
fi

### Nginx site setup (optional)
if is_enabled "$NGINX_ENABLE"; then
  # Try to obtain LE cert upfront (no nginx port conflicts if using standalone)
  obtain_certbot_cert "$API_DOMAIN" "$CERTBOT_EMAIL" "$PROXIED_API"

  # Choose TLS cert/key paths
  TLS_CERT_PATH="/etc/letsencrypt/live/${API_DOMAIN}/fullchain.pem"
  TLS_KEY_PATH="/etc/letsencrypt/live/${API_DOMAIN}/privkey.pem"
  if [ ! -f "$TLS_CERT_PATH" ] || [ ! -f "$TLS_KEY_PATH" ]; then
    # Fallback to Cloudflare Origin certs if LE is not available
    TLS_CERT_PATH="$CF_SSL_DIR/${NGINX_SERVER_NAME}.crt"
    TLS_KEY_PATH="$CF_SSL_DIR/${NGINX_SERVER_NAME}.key"
    echo "Using TLS from $TLS_CERT_PATH (LE not found)"
  else
    echo "Using Let's Encrypt cert at $TLS_CERT_PATH"
  fi

  echo "Generating nginx site config for $API_DOMAIN"
  sudo tee "$NGINX_SITE_PATH" > /dev/null <<NGX
server {
    listen 80;
    server_name ${API_DOMAIN};
    return 301 https://\$host\$request_uri;
}

server {
    listen 443 ssl;
    http2 on;
    server_name ${API_DOMAIN};

    ssl_certificate $TLS_CERT_PATH;
    ssl_certificate_key $TLS_KEY_PATH;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers off;

    client_max_body_size 1024M;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;

    location / {
        proxy_pass http://127.0.0.1:$APP_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 90s;
    }
}
NGX

  echo "Enabling nginx site"
  sudo ln -sf "$NGINX_SITE_PATH" "/etc/nginx/sites-enabled/$SERVICE_NAME"

  if [[ "$TLS_CERT_PATH" == *"letsencrypt"* ]]; then
    echo "Let's Encrypt cert configured for ${API_DOMAIN}"
  else
    echo "Warning: Using fallback TLS at $TLS_CERT_PATH. Consider enabling CERTBOT or placing Cloudflare Origin certs." >&2
  fi

  echo "Testing nginx configuration"
  sudo nginx -t || true

  if command -v ufw >/dev/null 2>&1; then
    sudo ufw allow 'Nginx Full' || true
  fi
fi

if is_enabled "$SYSTEMD_ENABLE"; then
  echo "Starting (or restarting) $SERVICE_NAME service"
  sudo systemctl restart "$SERVICE_NAME" || sudo systemctl start "$SERVICE_NAME"
  sudo systemctl status "$SERVICE_NAME" --no-pager -l || true
fi

if is_enabled "$NGINX_ENABLE"; then
  echo "Reloading nginx (if installed)"
  if command -v nginx >/dev/null 2>&1; then
    sudo systemctl reload nginx || sudo systemctl restart nginx || true
  fi
fi

echo "Done. To follow logs: sudo journalctl -u $SERVICE_NAME -f"

### Optional: write Cloudflare Origin cert/key if provided via environment or file
# WARNING: storing private keys in environment or repo is sensitive. Prefer uploading
# cert/key to the server via scp or using a secrets manager. This helper exists for
# automation convenience.
if is_enabled "$NGINX_ENABLE"; then
  echo "Checking for Cloudflare Origin certificate inputs..."
  sudo mkdir -p "$CF_SSL_DIR"
  sudo chmod 700 "$CF_SSL_DIR"

  CRT_TARGET="$CF_SSL_DIR/${NGINX_SERVER_NAME}.crt"
  KEY_TARGET="$CF_SSL_DIR/${NGINX_SERVER_NAME}.key"

  if [ -n "${CF_ORIGIN_CERT_FILE:-}" ] && [ -f "$CF_ORIGIN_CERT_FILE" ]; then
    echo "Copying origin cert from file $CF_ORIGIN_CERT_FILE to $CRT_TARGET"
    sudo cp -f "$CF_ORIGIN_CERT_FILE" "$CRT_TARGET"
  fi
  if [ -n "${CF_ORIGIN_KEY_FILE:-}" ] && [ -f "$CF_ORIGIN_KEY_FILE" ]; then
    echo "Copying origin key from file $CF_ORIGIN_KEY_FILE to $KEY_TARGET"
    sudo cp -f "$CF_ORIGIN_KEY_FILE" "$KEY_TARGET"
  fi

  if [ -n "${CF_ORIGIN_CERT:-}" ] && [ ! -f "$CRT_TARGET" ]; then
    echo "Writing origin cert from CF_ORIGIN_CERT env var to $CRT_TARGET"
    printf '%s' "$CF_ORIGIN_CERT" | sudo tee "$CRT_TARGET" > /dev/null
  fi
  if [ -n "${CF_ORIGIN_KEY:-}" ] && [ ! -f "$KEY_TARGET" ]; then
    echo "Writing origin key from CF_ORIGIN_KEY env var to $KEY_TARGET"
    printf '%s' "$CF_ORIGIN_KEY" | sudo tee "$KEY_TARGET" > /dev/null
  fi

  if [ -f "$KEY_TARGET" ] || [ -f "$CRT_TARGET" ]; then
    sudo chown root:root "$CF_SSL_DIR"/*
    sudo chmod 600 "$KEY_TARGET" || true
    sudo chmod 644 "$CRT_TARGET" || true
    echo "Origin cert/key present at $CF_SSL_DIR/"
  fi
fi

# Final summary
echo
echo "========== $SERVICE_NAME installation summary =========="
echo "Install dir:       $INSTALL_DIR"
echo "Binary installed:  $BIN_TARGET"
echo "Data dir:          $DATA_DIR"
echo "Uploads dir:       $UPLOADS_DIR"
echo "Env file:          $ENV_FILE"
if is_enabled "$SYSTEMD_ENABLE"; then
  echo "Systemd unit:      $SYSTEMD_UNIT (service: $SERVICE_NAME)"
else
  echo "Systemd unit:      disabled (set SYSTEMD_ENABLE=1 to enable)"
fi
if is_enabled "$NGINX_ENABLE"; then
  echo "Nginx site:        $NGINX_SITE_PATH (server_name: $API_DOMAIN)"
  echo "TLS cert path:     $TLS_CERT_PATH"
  echo "TLS key path:      $TLS_KEY_PATH"
else
  echo "Nginx:             disabled (set NGINX_ENABLE=1 to enable)"
fi
echo "========================================================"
