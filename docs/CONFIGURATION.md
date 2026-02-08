# Configuration

ParkHub works with zero configuration. Everything has defaults. Customize via CLI flags, environment variables, or a `config.toml` file.

## CLI Flags

```
parkhub-server [OPTIONS]

  -p, --port PORT      Server port (default: 7878)
  --data-dir PATH      Where to store the database file (default: ./data)
  --headless           No GUI, console only
  --unattended         Skip setup wizard, auto-configure with defaults
  -d, --debug          Enable debug logging
  -h, --help           Show help
  -v, --version        Show version
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PARKHUB_DATA_DIR` | `./data` | Directory for the redb database file and uploads |
| `PARKHUB_DB_PASSPHRASE` | — | If set, enables AES-256-GCM encryption for all database values. Key derived via PBKDF2-SHA256 with a random salt (stored in the database itself). |
| `RUST_LOG` | `info,parkhub_server=debug` | Standard Rust log filter. Set to `debug` or `trace` for troubleshooting. |

Environment variables override CLI flags, which override `config.toml`.

## config.toml

Place a `config.toml` in the working directory or specify `--config /path/to/config.toml`.

```toml
[app]
name = "Your Company Parking"

[server]
# active = "local"  or  "production"
local_url = "http://localhost:7878"
production_url = "https://parking.example.com"
active = "local"

[i18n]
# "de" or "en"
default_locale = "de"
available = ["de", "en"]

[development]
# Shows a dev panel with server switcher in the UI
enabled = false
show_dev_panel = false
```

The config file is primarily used by the desktop client (`parkhub-client`). The server reads most settings from CLI flags and environment variables.

## Database Encryption

When `PARKHUB_DB_PASSPHRASE` is set:

1. On first run, a random 32-byte salt is generated and stored in the `settings` table (unencrypted)
2. The passphrase + salt are fed through PBKDF2-SHA256 (600,000 iterations) to derive a 256-bit key
3. Every value written to redb is encrypted with AES-256-GCM using a random 96-bit nonce per write
4. Nonce is prepended to the ciphertext

The encryption is transparent — the API and application logic don't know about it. It protects against someone copying the database file off disk.

Caution: if you lose the passphrase, the data is gone. There's no recovery mechanism.

## Branding

Customizable via the admin panel (Settings → Branding):

- **Company name** — shown in the header, login page, and browser tab
- **Logo** — PNG or SVG, uploaded via the admin UI or API
- **Primary color** — overrides the theme's primary color globally

API endpoints for branding:

```bash
# Get current branding (public, no auth needed)
curl http://localhost:7878/api/v1/branding

# Update branding (admin only)
curl -X PUT http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"company_name": "Acme Corp"}'

# Upload logo (admin only)
curl -X POST http://localhost:7878/api/v1/admin/branding/logo \
  -H "Authorization: Bearer $TOKEN" \
  -F "logo=@company-logo.png"
```

## Rate Limiting

Built into the Tower middleware stack. Defaults:

| Scope | Limit |
|-------|-------|
| General API requests | 60/min per IP |
| Login attempts | 10/min per IP |

When exceeded, the server returns `429 Too Many Requests`.

---

Back to [README](../README.md) · Previous: [Installation](INSTALLATION.md) · Next: [API](API.md)
