# Installation

## Requirements

ParkHub is a single binary. It needs basically nothing:

- Any Linux (x86_64 or aarch64), macOS, or Windows machine
- ~50 MB disk (binary + database file)
- 128 MB RAM is plenty

## Interactive Installer

The installer provides two modes: **Quick Start** and **Custom Installation**.

### Linux / macOS

```bash
curl -fsSL https://github.com/nash87/parkhub-docker-docker/releases/latest/download/install.sh | bash
```

#### Quick Start Mode (default)

Downloads the right binary for your architecture, creates a default config, starts the server, and shows the access URL with your detected IP address. Ready in about 2 minutes.

#### Custom Installation Mode

Lets you configure before first start:

| Setting | Default |
|---------|---------|
| Port | 7878 |
| Data directory | ~/.local/share/parkhubserver |
| TLS | Disabled |
| Admin username | admin |
| Admin password | Auto-generated |
| Use-case type | Corporate |
| Organization name | My Parking |
| Self-registration | Enabled |
| Demo data | Disabled |

All settings are written to `config.toml` in the data directory.

If you prefer to do it manually:

```bash
# Linux x86_64
wget https://github.com/nash87/parkhub-docker-docker/releases/latest/download/parkhub-server-linux-amd64
chmod +x parkhub-server-linux-amd64
sudo mv parkhub-server-linux-amd64 /usr/local/bin/parkhub-server
```

### Windows

```powershell
irm https://github.com/nash87/parkhub-docker-docker/releases/latest/download/install.ps1 | iex
```

Or grab `parkhub-server-windows-amd64.exe` from the [releases page](https://github.com/nash87/parkhub-docker-docker/releases).

### Running

```bash
parkhub-server                     # GUI mode (if compiled with gui feature)
parkhub-server --headless          # Console only
parkhub-server -p 8080             # Custom port
parkhub-server --data-dir /data    # Custom data directory
parkhub-server --unattended        # Skip setup wizard, use defaults
parkhub-server --debug             # Verbose logging
```

The first user to register gets the `admin` role.

## Docker

```bash
docker run -d \
  --name parkhub \
  -p 7878:7878 \
  -v parkhub-data:/data \
  -e PARKHUB_DATA_DIR=/data \
  ghcr.io/nash87/parkhub-docker-docker:latest
```

The image is built with a multi-stage Dockerfile: Node 22 Alpine builds the frontend, Rust 1.83 Alpine builds the server with musl for a static binary. Final image is scratch-like, ~20 MB.

### Docker Compose

```yaml
services:
  parkhub:
    image: ghcr.io/nash87/parkhub-docker-docker:latest
    ports:
      - "7878:7878"
    volumes:
      - parkhub-data:/data
    environment:
      PARKHUB_DATA_DIR: /data
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

You need:
- Rust 1.83+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 22+ and npm
- On Linux: `musl-dev openssl-dev pkg-config` for static linking

```bash
git clone https://github.com/nash87/parkhub-docker-docker.git
cd parkhub

# Build the React frontend with Vite
cd parkhub-web
npm ci
npm run build    # outputs to parkhub-web/dist/
cd ..

# Build the Rust server
# build.rs embeds parkhub-web/dist/ into the binary at compile time
# Full build (includes desktop GUI)
cargo build --release --package parkhub-server

# Server-only build (no GUI dependencies, for headless servers)
# cargo build --release --package parkhub-server --no-default-features --features headless

# Binary at ./target/release/parkhub-server
./target/release/parkhub-server
```

The workspace has three crates:
- `parkhub-common` — shared types (User, Booking, ParkingLot, etc.)
- `parkhub-server` — the Axum server, database layer, and API
- `parkhub-client` — desktop client (optional, uses Slint GUI toolkit)

## Kubernetes

<details>
<summary>Deployment + Service + PVC</summary>

Important: run exactly **1 replica**. redb is a single-writer database — multiple writers will corrupt the data file.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: parkhub
spec:
  replicas: 1    # DO NOT increase. redb is single-writer.
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
          image: ghcr.io/nash87/parkhub-docker-docker:latest
          args: ["--headless"]
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
          resources:
            requests:
              memory: "64Mi"
              cpu: "50m"
            limits:
              memory: "256Mi"
              cpu: "500m"
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
  accessModes: [ReadWriteOnce]
  resources:
    requests:
      storage: 1Gi
```

</details>

## Reverse Proxy

ParkHub serves everything on a single port. Put any reverse proxy in front for TLS.

### Caddy (easiest)

```caddyfile
parking.example.com {
    reverse_proxy localhost:7878
}
```

That's it. Caddy handles Let's Encrypt automatically.

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

## Built-in TLS

ParkHub can terminate TLS itself if you don't want a reverse proxy:

```bash
parkhub-server --tls-cert /etc/parkhub/cert.pem --tls-key /etc/parkhub/key.pem
```

For production, a reverse proxy with auto-renewing certs is usually easier.

> **Note:** When behind a reverse proxy, edit `config.toml` in the data directory and set `enable_tls = false` and `encryption_enabled = false` to avoid double encryption and certificate conflicts.

## systemd Service

```ini
[Unit]
Description=ParkHub Parking Management
After=network.target

[Service]
Type=simple
User=parkhub
Group=parkhub
ExecStart=/usr/local/bin/parkhub-server --headless --data-dir /var/lib/parkhub
Restart=always
RestartSec=5
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/parkhub
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

```bash
sudo useradd -r -s /usr/sbin/nologin parkhub
sudo mkdir -p /var/lib/parkhub
sudo chown parkhub:parkhub /var/lib/parkhub
sudo cp parkhub.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now parkhub
```

---

Back to [README](../README.md) · Next: [Configuration](CONFIGURATION.md)
