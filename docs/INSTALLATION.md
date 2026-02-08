# Installation Guide

Complete guide to installing and running ParkHub.

## System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| CPU | 1 core | 2+ cores |
| RAM | 128 MB | 256 MB |
| Disk | 50 MB | 1 GB (includes data) |
| OS | Linux (x86_64, aarch64), macOS, Windows | Linux x86_64 |

## Binary Installation

### Linux / macOS

```bash
curl -fsSL https://github.com/nash87/parkhub/releases/latest/download/install.sh | bash
```

The script downloads the appropriate binary for your platform and places it in `/usr/local/bin/`.

Manual download:

```bash
# Linux x86_64
wget https://github.com/nash87/parkhub/releases/latest/download/parkhub-server-linux-amd64
chmod +x parkhub-server-linux-amd64
sudo mv parkhub-server-linux-amd64 /usr/local/bin/parkhub-server

# macOS (Apple Silicon)
wget https://github.com/nash87/parkhub/releases/latest/download/parkhub-server-darwin-arm64
chmod +x parkhub-server-darwin-arm64
sudo mv parkhub-server-darwin-arm64 /usr/local/bin/parkhub-server
```

### Windows

```powershell
irm https://github.com/nash87/parkhub/releases/latest/download/install.ps1 | iex
```

Or download `parkhub-server-windows-amd64.exe` from the [releases page](https://github.com/nash87/parkhub/releases).

## Docker

### Quick Start

```bash
docker run -d \
  --name parkhub \
  -p 7878:7878 \
  -v parkhub-data:/data \
  -e PARKHUB_DATA_DIR=/data \
  ghcr.io/nash87/parkhub:latest
```

### Docker Compose

See the included [`docker-compose.yml`](../docker-compose.yml) or the [Deployment Guide](DEPLOYMENT.md).

```yaml
services:
  parkhub:
    image: ghcr.io/nash87/parkhub:latest
    ports:
      - "7878:7878"
    volumes:
      - parkhub-data:/data
    environment:
      - PARKHUB_DATA_DIR=/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:7878/health"]
      interval: 30s
      timeout: 5s
      retries: 3

volumes:
  parkhub-data:
```

## Building from Source

### Prerequisites

- Rust 1.83+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 22+ and npm
- Git

### Build Steps

```bash
git clone https://github.com/nash87/parkhub.git
cd parkhub

# Build the frontend
cd parkhub-web
npm ci
npm run build
cd ..

# Build the server (includes the frontend via build.rs)
cargo build --release --package parkhub-server

# Binary is at ./target/release/parkhub-server
```

## Kubernetes Deployment

<details>
<summary>Kubernetes manifests</summary>

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: parkhub
  labels:
    app: parkhub
spec:
  replicas: 1
  selector:
    matchLabels:
      app: parkhub
  template:
    metadata:
      labels:
        app: parkhub
    spec:
      containers:
        - name: parkhub
          image: ghcr.io/nash87/parkhub:latest
          ports:
            - containerPort: 7878
          volumeMounts:
            - name: data
              mountPath: /data
          env:
            - name: PARKHUB_DATA_DIR
              value: /data
          livenessProbe:
            httpGet:
              path: /health/live
              port: 7878
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 7878
            initialDelaySeconds: 5
            periodSeconds: 10
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: parkhub-data
---
apiVersion: v1
kind: Service
metadata:
  name: parkhub
spec:
  selector:
    app: parkhub
  ports:
    - port: 80
      targetPort: 7878
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: parkhub-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 1Gi
```

</details>

## Reverse Proxy Setup

### Nginx

```nginx
server {
    listen 443 ssl http2;
    server_name parking.example.com;

    ssl_certificate     /etc/letsencrypt/live/parking.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/parking.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:7878;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Caddy

```caddyfile
parking.example.com {
    reverse_proxy localhost:7878
}
```

### Traefik

```yaml
http:
  routers:
    parkhub:
      rule: "Host(`parking.example.com`)"
      entryPoints: ["websecure"]
      service: parkhub
      tls:
        certResolver: letsencrypt
  services:
    parkhub:
      loadBalancer:
        servers:
          - url: "http://127.0.0.1:7878"
```

## TLS / HTTPS

ParkHub supports built-in TLS:

```toml
[tls]
enabled = true
cert = "/etc/parkhub/cert.pem"
key = "/etc/parkhub/key.pem"
```

For production, a reverse proxy with Let's Encrypt is recommended (see above).

## systemd Service

```ini
[Unit]
Description=ParkHub Parking Management
After=network.target

[Service]
Type=simple
User=parkhub
Group=parkhub
ExecStart=/usr/local/bin/parkhub-server
WorkingDirectory=/var/lib/parkhub
Environment=PARKHUB_DATA_DIR=/var/lib/parkhub
Restart=always
RestartSec=5

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/parkhub
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

```bash
sudo cp parkhub.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now parkhub
```

---

Back to [README](../README.md) · Next: [Configuration](CONFIGURATION.md)
