# Configuration Reference

ParkHub uses a `config.toml` file and environment variables for configuration. All settings have sensible defaults.

## Configuration File

By default, ParkHub looks for `config.toml` in the working directory. Override with:

```bash
parkhub-server --config /path/to/config.toml
```

### Full Reference

```toml
[server]
# Port to listen on (default: 7878)
port = 7878

# Data directory for the embedded database
data_dir = "./data"

# Bind address (default: 0.0.0.0)
bind = "0.0.0.0"

# Log level: trace, debug, info, warn, error
log_level = "info"

[tls]
# Enable built-in TLS (default: false)
enabled = false

# Path to TLS certificate (PEM format)
cert = "/etc/parkhub/cert.pem"

# Path to TLS private key (PEM format)
key = "/etc/parkhub/key.pem"

[smtp]
# Enable email notifications (default: false)
enabled = false

# SMTP server hostname
host = "smtp.example.com"

# SMTP port (default: 587)
port = 587

# SMTP username
username = "parking@example.com"

# SMTP password
password = "your-password"

# Sender address
from = "parking@example.com"

# Use STARTTLS (default: true)
starttls = true

[rate_limit]
# Requests per minute per IP (default: 60)
requests_per_minute = 60

# Login attempts per minute per IP (default: 10)
login_attempts_per_minute = 10

# Enable rate limiting (default: true)
enabled = true

[branding]
# Company name displayed in the UI
company_name = "Your Company"

# Custom logo path (uploaded via admin panel)
# logo = "/data/branding/logo.png"

[i18n]
# Default locale: "de" or "en"
default_locale = "de"

# Available locales
available = ["de", "en"]

[security]
# JWT token expiry in hours (default: 24)
token_expiry_hours = 24

# Refresh token expiry in days (default: 30)
refresh_token_expiry_days = 30

# Minimum password length (default: 8)
min_password_length = 8
```

## Environment Variables

All config options can be set via environment variables with the `PARKHUB_` prefix:

| Variable | Config Equivalent | Default |
|----------|-------------------|---------|
| `PARKHUB_PORT` | `server.port` | `7878` |
| `PARKHUB_DATA_DIR` | `server.data_dir` | `./data` |
| `PARKHUB_BIND` | `server.bind` | `0.0.0.0` |
| `PARKHUB_LOG_LEVEL` | `server.log_level` | `info` |
| `PARKHUB_TLS_ENABLED` | `tls.enabled` | `false` |
| `PARKHUB_TLS_CERT` | `tls.cert` | — |
| `PARKHUB_TLS_KEY` | `tls.key` | — |
| `PARKHUB_SMTP_HOST` | `smtp.host` | — |
| `PARKHUB_SMTP_PORT` | `smtp.port` | `587` |
| `PARKHUB_SMTP_USER` | `smtp.username` | — |
| `PARKHUB_SMTP_PASS` | `smtp.password` | — |
| `PARKHUB_SMTP_FROM` | `smtp.from` | — |
| `PARKHUB_RATE_LIMIT` | `rate_limit.requests_per_minute` | `60` |

Environment variables take precedence over `config.toml`.

## Branding Customization

Customize the look via the Admin Panel → Branding section:

- **Company name** — displayed in the header and login page
- **Logo** — upload a custom logo (PNG/SVG, max 2 MB)
- **Primary color** — override the theme's primary color

Or configure via API:

```bash
# Update branding
curl -X PUT http://localhost:7878/api/v1/admin/branding \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"company_name": "Acme Corp"}'

# Upload logo
curl -X POST http://localhost:7878/api/v1/admin/branding/logo \
  -H "Authorization: Bearer $TOKEN" \
  -F "logo=@logo.png"
```

---

Back to [README](../README.md) · Previous: [Installation](INSTALLATION.md) · Next: [API](API.md)
