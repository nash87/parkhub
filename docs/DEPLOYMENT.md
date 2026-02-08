# Production Deployment Guide

## Docker Compose (Recommended)

```yaml
services:
  parkhub:
    image: ghcr.io/nash87/parkhub:latest
    container_name: parkhub
    ports:
      - "7878:7878"
    volumes:
      - parkhub-data:/data
    environment:
      - PARKHUB_DATA_DIR=/data
      - PARKHUB_LOG_LEVEL=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://localhost:7878/health"]
      interval: 30s
      timeout: 5s
      retries: 3

  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy-data:/data
    depends_on:
      - parkhub

volumes:
  parkhub-data:
  caddy-data:
```

**Caddyfile:**

```caddyfile
parking.example.com {
    reverse_proxy parkhub:7878
}
```

## Health Checks

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | General health status |
| `GET /health/live` | Kubernetes liveness probe |
| `GET /health/ready` | Kubernetes readiness probe |
| `GET /metrics` | Prometheus-compatible metrics |

## Backup Strategy

ParkHub uses a single redb file for all data. Backup is simple:

```bash
# Stop the server (or use file-level snapshot)
cp /data/parkhub.redb /backups/parkhub-$(date +%Y%m%d).redb

# Automated daily backup via cron
0 2 * * * cp /var/lib/parkhub/parkhub.redb /backups/parkhub-$(date +\%Y\%m\%d).redb
```

For Docker volumes:

```bash
docker run --rm \
  -v parkhub-data:/data:ro \
  -v $(pwd)/backups:/backups \
  alpine cp /data/parkhub.redb /backups/parkhub-$(date +%Y%m%d).redb
```

## Monitoring

ParkHub exposes a `/metrics` endpoint compatible with Prometheus:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: parkhub
    static_configs:
      - targets: ["parkhub:7878"]
    metrics_path: /metrics
```

## Log Management

ParkHub outputs structured logs to stdout. Adjust the level via:

```bash
PARKHUB_LOG_LEVEL=debug parkhub-server
```

Levels: `trace`, `debug`, `info`, `warn`, `error`

For production, pipe to a log aggregator:

```bash
parkhub-server 2>&1 | tee /var/log/parkhub.log
```

Or use Docker logging drivers:

```yaml
services:
  parkhub:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
```

## Kubernetes

See the full manifests in the [Installation Guide](INSTALLATION.md#kubernetes-deployment).

Key considerations:
- Use a `PersistentVolumeClaim` for the data directory
- Only **1 replica** — redb is a single-writer database
- Configure liveness and readiness probes
- Use an Ingress controller for TLS termination

---

Back to [README](../README.md) · Previous: [API](API.md) · Next: [Development](DEVELOPMENT.md)
