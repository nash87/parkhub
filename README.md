<p align="center">
  <img src="docs/screenshots/logo-banner.png" alt="ParkHub" width="600">
</p>

<h1 align="center">ParkHub</h1>

<p align="center">
  <strong>Open-source corporate parking management. One binary. Zero dependencies. Beautiful UI.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge" alt="MIT License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.83+-orange.svg?style=for-the-badge&logo=rust" alt="Rust"></a>
  <a href="https://www.typescriptlang.org/"><img src="https://img.shields.io/badge/TypeScript-5.x-3178C6.svg?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript"></a>
  <a href="https://hub.docker.com/r/nash87/parkhub"><img src="https://img.shields.io/badge/Docker-Ready-2496ED.svg?style=for-the-badge&logo=docker&logoColor=white" alt="Docker"></a>
</p>

<p align="center">
  <a href="docs/INSTALLATION.md">Installation</a> · <a href="docs/API.md">API Docs</a> · <a href="docs/CONFIGURATION.md">Configuration</a> · <a href="docs/DEPLOYMENT.md">Deployment</a> · <a href="docs/DEVELOPMENT.md">Development</a> · <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

ParkHub is a self-hosted parking management system for companies. It ships as a single binary with an embedded database — no PostgreSQL, no Redis, no external services. Drop it on a server and it runs.

## Key Features

| Feature | Description |
|---|---|
| **Real-time Slot Management** | Interactive visual parking map with live availability |
| **Smart Booking System** | One-time, multi-day, and permanent reservations with check-in & QR codes |
| **10 Color Themes** | Solarized, Dracula, Nord, Gruvbox, Catppuccin, Tokyo Night, One Dark, Rose Pine, Everforest, Default Blue |
| **Dark / Light Mode** | Full dark theme with automatic system detection |
| **Internationalization** | German & English with extensible i18n framework |
| **Accessibility** | Colorblind modes (protanopia, deuteranopia, tritanopia), font scaling, reduced motion, high contrast |
| **Corporate Branding** | Custom logo, colors, and company name via admin panel |
| **Homeoffice Integration** | Set WFH patterns, auto-release parking spots for colleagues |
| **GDPR / DSGVO** | Data export, account deletion, privacy policy — fully compliant |
| **PWA-Ready** | Install as native app on any device |
| **Single Binary** | ~30 MB, embedded redb database, zero external dependencies |
| **REST API** | Full API with Swagger/OpenAPI documentation |
| **iCal Export** | Subscribe to your bookings in any calendar app |
| **Admin Dashboard** | Reports, statistics, CSV export, user management |
| **Waitlist System** | Automatic notification when a spot becomes available |
| **Rate Limiting** | Built-in request throttling per IP and per user |
| **Security Hardened** | XSS prevention, input validation, HSTS, security headers |

## Quick Start

```bash
# Download the latest release
curl -fsSL https://github.com/nash87/parkhub/releases/latest/download/install.sh | bash

# Start the server
parkhub-server

# Open your browser
open http://localhost:7878
```

Default port is **7878**. An admin account (`admin` / `admin`) is created on first start. Change the password immediately.

On first login, an onboarding wizard guides you through initial setup: company branding, creating your first parking lot, and configuring slots.

## Screenshots

<p align="center">
  <img src="docs/screenshots/login.png" alt="Login" width="45%">
  &nbsp;&nbsp;
  <img src="docs/screenshots/dashboard.png" alt="Dashboard" width="45%">
</p>

<p align="center">
  <img src="docs/screenshots/booking.png" alt="Booking" width="45%">
  &nbsp;&nbsp;
  <img src="docs/screenshots/admin.png" alt="Admin Panel" width="45%">
</p>

<details>
<summary>More screenshots</summary>
<br>
<p align="center">
  <img src="docs/screenshots/vehicles.png" alt="Vehicles" width="45%">
  &nbsp;&nbsp;
  <img src="docs/screenshots/themes.png" alt="Themes" width="45%">
</p>
<p align="center">
  <img src="docs/screenshots/dark-mode.png" alt="Dark Mode" width="45%">
  &nbsp;&nbsp;
  <img src="docs/screenshots/profile.png" alt="Profile" width="45%">
</p>
</details>

## Installation

### Binary

```bash
# Linux / macOS
curl -fsSL https://github.com/nash87/parkhub/releases/latest/download/install.sh | bash

# Windows (PowerShell)
irm https://github.com/nash87/parkhub/releases/latest/download/install.ps1 | iex
```

### Docker

```bash
docker run -d \
  --name parkhub \
  -p 7878:7878 \
  -v parkhub-data:/data \
  ghcr.io/nash87/parkhub:latest
```

### From Source

```bash
git clone https://github.com/nash87/parkhub.git
cd parkhub

# Build frontend
cd parkhub-web && npm ci && npm run build && cd ..

# Build server
cargo build --release --package parkhub-server

# Run
./target/release/parkhub-server
```

See the full [Installation Guide](docs/INSTALLATION.md) for Docker Compose, Kubernetes, reverse proxy, systemd, and TLS setup.

## Configuration

ParkHub works out of the box with sensible defaults. For customization, create a `config.toml`:

```toml
[server]
port = 7878
data_dir = "/var/lib/parkhub"

[tls]
enabled = false
cert = "/etc/parkhub/cert.pem"
key = "/etc/parkhub/key.pem"

[smtp]
enabled = false
host = "smtp.example.com"
from = "parking@example.com"

[rate_limit]
requests_per_minute = 60
```

See [Configuration Reference](docs/CONFIGURATION.md) for all options.

## API Overview

All endpoints are under `/api/v1/`. Authentication uses Bearer tokens.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/auth/login` | Authenticate and receive tokens |
| `POST` | `/auth/register` | Create new account |
| `GET` | `/lots` | List parking lots |
| `GET` | `/lots/:id/slots` | Get slots for a lot |
| `POST` | `/bookings` | Create a booking |
| `GET` | `/bookings/ical` | Export bookings as iCal |
| `POST` | `/bookings/:id/checkin` | Check in to booking |
| `GET` | `/vehicles` | List user vehicles |
| `GET/PUT` | `/homeoffice` | Manage homeoffice settings |
| `GET` | `/admin/stats` | Dashboard statistics |
| `GET` | `/admin/reports` | Generate reports |
| `GET` | `/users/me/export` | GDPR data export |
| `DELETE` | `/users/me/delete` | GDPR account deletion |

See the full [API Documentation](docs/API.md) for all 40+ endpoints with examples.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                        Browser                           │
│              React · TypeScript · TailwindCSS             │
└────────────────────────┬─────────────────────────────────┘
                         │ HTTPS
┌────────────────────────┴─────────────────────────────────┐
│              Reverse Proxy (Nginx / Caddy)                │
│                    TLS termination                        │
└────────────────────────┬─────────────────────────────────┘
                         │ HTTP
┌────────────────────────┴─────────────────────────────────┐
│                    ParkHub Server                         │
│         Rust · Axum · Tower (rate limiting)               │
│                                                          │
│  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌───────────┐  │
│  │  Auth   │  │ Bookings │  │  Admin  │  │  Metrics  │  │
│  │  Token  │  │  iCal    │  │ Reports │  │ Prometheus│  │
│  └─────────┘  └──────────┘  └─────────┘  └───────────┘  │
│                                                          │
│  ┌─────────────────────────────────────────────────────┐  │
│  │                  redb (embedded)                    │  │
│  │            Zero-config · ACID · Fast                │  │
│  └─────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust 1.83+, Axum, Tower, Tokio |
| Database | redb (embedded key-value store) |
| Frontend | React 18, TypeScript 5, TailwindCSS 3 |
| Animations | Framer Motion |
| Icons | Phosphor Icons |
| Build | Vite, Cargo |
| Container | Docker (multi-stage, ~20 MB image) |
| CI/CD | GitHub Actions |

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and the PR process.

## License

ParkHub is licensed under the [MIT License](LICENSE).

---

<p align="center">
  <a href="docs/INSTALLATION.md">Installation</a> · <a href="docs/API.md">API</a> · <a href="docs/CONFIGURATION.md">Config</a> · <a href="docs/DEPLOYMENT.md">Deploy</a> · <a href="docs/DEVELOPMENT.md">Develop</a> · <a href="docs/THEMES.md">Themes</a> · <a href="docs/SECURITY.md">Security</a> · <a href="docs/ACCESSIBILITY.md">Accessibility</a>
</p>
