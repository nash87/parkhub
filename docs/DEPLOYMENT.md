# Deployment

## Docker Compose

This is the simplest production setup. ParkHub + Caddy for automatic HTTPS.

```yaml
services:
  parkhub:
    image: ghcr.io/nash87/parkhub:latest
    container_name: parkhub
    command: ["--headless", "--data-dir", "/data"]
    volumes:
      - parkhub-data:/data
    environment:
      RUST_LOG: "info"
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
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy-data:/data
    depends_on:
      parkhub:
        condition: service_healthy
    restart: unless-stopped

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

Start: `docker compose up -d`. Caddy gets a Let's Encrypt cert automatically.

If you don't need HTTPS (e.g., behind a corporate VPN), skip Caddy:

```yaml
services:
  parkhub:
    image: ghcr.io/nash87/parkhub:latest
    ports:
      - "7878:7878"
    volumes:
      - parkhub-data:/data
    environment:
      PARKHUB_DATA_DIR: /data
    restart: unless-stopped

volumes:
  parkhub-data:
```

## Health Checks

Three endpoints for monitoring and orchestrators:

| Endpoint | What it checks | Use for |
|----------|---------------|---------|
| `GET /health` | Returns `"ok"` if the process is running | Simple uptime check |
| `GET /health/live` | Always returns `200` if the server is alive | K8s `livenessProbe` |
| `GET /health/ready` | Returns `200` if the database is accessible, `503` otherwise | K8s `readinessProbe` |

## Prometheus Metrics

`GET /metrics` returns Prometheus-format metrics. Scrape it:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: parkhub
    static_configs:
      - targets: ["parkhub:7878"]
    metrics_path: /metrics
    scrape_interval: 30s
```

## Backups

All data lives in a single redb file (default: `./data/parkhub.redb`). Back it up by copying the file.

For a consistent backup, either stop the server briefly or copy during low-traffic hours. redb uses MVCC, so a file copy during operation is generally safe, but stopping the server guarantees consistency.

```bash
# Simple cron backup
0 3 * * * cp /var/lib/parkhub/parkhub.redb /backups/parkhub-$(date +\%Y\%m\%d).redb
```

Docker volume backup:

```bash
docker run --rm \
  -v parkhub-data:/data:ro \
  -v ./backups:/backups \
  alpine cp /data/parkhub.redb /backups/parkhub-$(date +%Y%m%d).redb
```

To restore: stop ParkHub, replace the `.redb` file, restart.

## Logging

Structured logs go to stdout. Control verbosity with `RUST_LOG`:

```bash
RUST_LOG=debug parkhub-server          # verbose
RUST_LOG=warn parkhub-server           # quiet
RUST_LOG=info,parkhub_server=trace     # trace only parkhub code
```

Docker Compose logging config to avoid filling disks:

```yaml
services:
  parkhub:
    logging:
      driver: json-file
      options:
        max-size: "10m"
        max-file: "3"
```

## Scaling

You can't. redb is a single-writer embedded database. Run exactly one instance. If you need horizontal scaling, ParkHub isn't the right tool — but for its intended use case (a company with dozens to hundreds of employees), a single instance handles it easily.

---

Back to [README](../README.md) · Previous: [API](API.md) · Next: [Development](DEVELOPMENT.md)
