# Deploying the backend on a Debian VPS

This file contains ready-to-run instructions and example files to run the `ping0` backend on a Debian-based VPS (you previously used Debian sid).

1) Users, directories, permissions

Create a dedicated system user and directories (run as sudo):

```bash
sudo useradd --system --home /opt/ping0 --shell /usr/sbin/nologin ping0
sudo mkdir -p /opt/ping0 /var/lib/ping0/uploads
sudo chown root:ping0 /opt/ping0
sudo chown ping0:ping0 /var/lib/ping0/uploads
sudo chmod 750 /opt/ping0
sudo chmod 750 /var/lib/ping0
```

Place the built binary at `/opt/ping0/ping0-server` and make it executable. If you build with `cargo build --release` on the VPS, copy `target/release/ping0-server` to `/opt/ping0/` and `chown root:ping0` and `chmod 750`.

2) Environment file
Create `/etc/default/ping0` (or `/etc/ping0.env`) and set env vars:

```
# /etc/default/ping0
PORT=8080
HOST=0.0.0.0
BASE_URL=
UPLOAD_DIR=/var/lib/ping0/uploads
MAX_BODY=10485760
```

3) systemd service
Create `/etc/systemd/system/ping0.service` with:

```
[Unit]
Description=ping0 backend
After=network.target

[Service]
Type=simple
User=ping0
Group=ping0
EnvironmentFile=/etc/default/ping0
ExecStart=/opt/ping0/ping0-server
Restart=on-failure
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

Reload systemd and enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now ping0.service
sudo journalctl -u ping0 -f
```

4) Verify health

```bash
curl -sS http://127.0.0.1:8080/health
```

5) Optional: nginx reverse-proxy + TLS (recommended)

Install nginx and certbot:

```bash
sudo apt update
sudo apt install -y nginx certbot python3-certbot-nginx
```

Example nginx site `/etc/nginx/sites-available/ping0`:

```
server {
    listen 80;
    server_name api.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Enable and test nginx:

```bash
sudo ln -s /etc/nginx/sites-available/ping0 /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl restart nginx
```

Request a certificate with certbot:

```bash
sudo certbot --nginx -d api.example.com
```

6) UFW firewall (optional)

```bash
sudo apt install -y ufw
sudo ufw allow OpenSSH
sudo ufw allow 'Nginx Full'
sudo ufw enable
```

7) Logs & rotation

systemd journalctl is available. For file-based logs, consider adding `--log-file` options or configuring an external logger. Add a logrotate rule if you create files.

8) Troubleshooting

- If you see `Permission denied` running the binary as `ping0`, check directory permissions and mount options (noexec). Use `namei -l /opt/ping0/ping0-server` to inspect.
- If the service fails on startup, check `sudo journalctl -u ping0 -b --no-pager -n 200`.
