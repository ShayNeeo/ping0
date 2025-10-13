# ping0 — Fast file & link sharing with QR

This README is written for users and operators who want to run the ping0 service. It explains how to deploy the backend on a Debian VPS, how to host the frontend on Cloudflare Pages, and the minimal operational steps: start, stop, check status, and upgrade.

If you are a developer looking for build internals, the repository contains a developer README and build scripts — this file focuses on deploying and running the service in production.

Contents
- Quick start (VPS)
- Cloudflare Pages (frontend)
- Systemd service (backend)
- Nginx and TLS
- Configuration and runtime files
- Backups, logs, and monitoring
- Troubleshooting
- Next steps and resources

## Quick start (one VPS, Debian / Debian Sid)

This section assumes you have a Debian-based VPS and SSH access with sudo.

1) Clone the repository under `/opt/ping0/repo` (recommended location):

```bash
sudo mkdir -p /opt/ping0
sudo chown $USER:$USER /opt/ping0
cd /opt/ping0
git clone https://github.com/ShayNeeo/ping0.git repo
cd repo
```

2) Run the automated installer. It will:
- install system packages (Debian/apt)
- create a system user `ping0`
- install rustup (if needed) and the Rust toolchain for building
- build and install the backend binary to `/opt/ping0/ping0-server`
- create upload directories and set permissions
- write a basic nginx site and enable it (optional)

Run:
```bash
# set NGINX_SERVER_NAME if you want nginx configured; optional.
sudo NGINX_SERVER_NAME=api.0.id.vn ./deploy/install.sh
```

Notes:
- The script supports installing Cloudflare Origin certificates during the run using these env vars or file paths:
  - `CF_ORIGIN_CERT` (PEM text) or `CF_ORIGIN_CERT_FILE` (path)
  - `CF_ORIGIN_KEY` (PEM text) or `CF_ORIGIN_KEY_FILE` (path)
- It's recommended to upload certs via `scp` and use `CF_ORIGIN_CERT_FILE` / `CF_ORIGIN_KEY_FILE` rather than embedding keys in env vars.
```
sudo scp cloudflare-origin.crt root@server:/root/
sudo scp cloudflare-origin.key root@server:/root/
sudo CF_ORIGIN_CERT_FILE=/root/cloudflare-origin.crt CF_ORIGIN_KEY_FILE=/root/cloudflare-origin.key NGINX_SERVER_NAME=api.0.id.vn ./deploy/install.sh
```

## Cloudflare Pages (frontend)

The frontend in this repo is a static site under `static/`. We recommend hosting it on Cloudflare Pages and the backend on your VPS.

- The frontend reads `/config.json` at runtime to know the API base URL. Create `static/config.json` with:

```json
{
  "apiBase": "https://api.0.id.vn"
}
```

- Deploy options:
  - Manual: drag & drop the `static/` folder in the Pages UI.
  - Git-based: connect your repository and set the build output directory to `static`.
  - Wrangler CLI: `wrangler pages publish static --project-name=ping0-frontend`.

- If you prefer to have frontend + backend on the same origin, configure nginx on your VPS to serve the static files and proxy API routes (example in the Nginx section).

## Systemd service (backend)

The installer creates a systemd unit at `/etc/systemd/system/ping0.service`. Useful commands:

```bash
sudo systemctl start ping0
sudo systemctl stop ping0
sudo systemctl restart ping0
sudo systemctl status ping0 -l
sudo journalctl -u ping0 -f
```

Health endpoint: `GET /health` returns a small JSON payload. Example:
```bash
curl -v https://api.0.id.vn/health
```

If the service fails to start, check permissions on `/opt/ping0` and `/var/lib/ping0/uploads` and ensure the `ping0` user can traverse and write to required directories.

## Nginx and TLS

The installer writes a simple site to `/etc/nginx/sites-available/ping0` and symlinks it to `sites-enabled`. The default config:
- redirects HTTP->HTTPS
- proxies `/` to `http://127.0.0.1:8080`
- enforces `client_max_body_size 12M` (the server enforces 10MB as well)

TLS options:
- Recommended: Use Cloudflare in proxy mode and install a Cloudflare Origin Certificate on the VPS. Place cert and key at `/etc/ssl/cf_origin/<domain>.crt` and `.key` (the installer can write them if you provide env vars or file paths).
- Alternate: Use certbot/Let's Encrypt. If Cloudflare is proxied, set DNS to gray cloud while obtaining certs.

After editing nginx config, test and reload:

```bash
sudo nginx -t
sudo systemctl reload nginx
```

## Configuration and runtime files

- `/etc/default/ping0`: environment file used by systemd. Example variables:
  - PORT=8080
  - HOST=0.0.0.0
  - BASE_URL=
  - UPLOAD_DIR=/var/lib/ping0/uploads
  - MAX_BODY=10485760

- `/var/lib/ping0/uploads`: file uploads (make sure this directory is owned by `ping0` and backed up as needed).

## Backups, logs, and monitoring

- Backups: copy `/var/lib/ping0/uploads` to a backup location regularly (rsync, cron, or object storage).
- Logs: use `journalctl -u ping0` or forward logs to a central service.
- Health checks: point your monitoring (UptimeRobot, healthchecks.io) at `https://api.0.id.vn/health`.

## Troubleshooting

- CHDIR / permission errors: ensure `/opt/ping0` is `root:ping0` with `750` permissions and binary is `root:ping0` `750`.
- 405 on `/link` when using Pages: ensure `static/config.json` exists and `apiBase` points to `https://api.0.id.vn` so the frontend posts to the backend origin.
- CORS errors: if frontend and backend are different origins, enable CORS on the backend or proxy API through the same origin.
- Docker builds: the repo has Dockerfile(s) for containerized builds — see the top-level Dockerfile if you prefer building images.

## Upgrading

To update the backend binary on the VPS:

1. In the repo on the VPS: pull latest changes and run installer again (it will build and install):
```bash
cd /opt/ping0/repo
git pull origin main
sudo ./deploy/install.sh
```

2. Alternatively, build locally and copy the release binary to `/opt/ping0/ping0-server`, then restart systemd.

## Security notes

- Never commit private keys to the repo. Use the installer to copy keys via secure channel.
- Restrict origin access to Cloudflare IP ranges if you use Cloudflare proxying.
- Run regular OS updates and monitor open ports.

## Need help?

If something fails, collect and share these outputs when asking for help:

```bash
sudo systemctl status ping0 -l
sudo journalctl -u ping0 -n 200 --no-pager -o cat
sudo nginx -t
sudo tail -n 200 /var/log/nginx/error.log
sudo ls -la /opt/ping0 /var/lib/ping0
```

Happy running!
