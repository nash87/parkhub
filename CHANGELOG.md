# Changelog

All notable changes to ParkHub are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed
- Repository renamed from `parkhub` to `parkhub-docker` to better reflect the Docker-first deployment model
- Docker image name updated to `ghcr.io/nash87/parkhub-docker`
- All installation URLs and clone commands updated

## [1.2.0] — 2026-02-08

### Added

- **License plate city code autocomplete**: Dropdown with 400+ German Unterscheidungszeichen
  - Filtered dropdown appears on first keystroke, max 10 items
  - Prefix highlighting, keyboard navigation (arrow keys, Enter, Escape)
  - Auto-formatting: city code → dash → letters → space → numbers
  - Selected city name shown below input (e.g. "Göttingen" for GÖ)
  - Dark mode compatible dropdown styling
- **Interactive install script** (): Modern two-mode installer
  - Quick Start mode: Default settings, ready in 2 minutes
  - Custom Installation mode: Configure port, TLS, admin credentials, use-case, etc.
  - Auto-detects host IP and shows access URL
  - Color-coded output with progress indicators
  - OS and architecture detection (Linux/macOS)
- **Windows installer overhaul** (): Same two-mode flow for Windows

### Changed

- License plate input now uses structured city code selection instead of free-form typing
- Install scripts default to port 7878 (was 8080)
- Install scripts show onboarding wizard URL after installation

## [1.1.0] — 2026-02-08

### Added

- Vehicle autocomplete with 50+ car brands for quick vehicle registration
- Vehicle photo upload with automatic resize (max 800px, Lanczos3)
- German license plate format validation (XX-YY 1234)
- Booking date/time picker for selecting specific date and time ranges
- Slot favorites for quick booking of preferred spots
- Demo vehicles with colored placeholder photos in dummy data generation
- Comprehensive documentation updates (API, Configuration, Development, Accessibility)

### Changed

- Dummy data now includes 8 realistic German demo vehicles with photos
- `--features headless` documented for server-only builds
- TLS and encryption settings documented for reverse proxy setups

### Fixed

- Various API endpoint documentation accuracy improvements
- Configuration docs now cover `enable_tls` and `encryption_enabled` settings

## [1.0.0] — 2026-02-08

### Added

- Real-time parking slot management with interactive visual parking map
- Booking system: one-time, multi-day, and permanent reservations
- Check-in with QR code scanning
- 10 color themes: Default Blue, Solarized, Dracula, Nord, Gruvbox, Catppuccin, Tokyo Night, One Dark, Rose Pine, Everforest
- Dark / Light mode with system preference detection
- Internationalization: 10 languages (English, German, Spanish, French, Portuguese, Turkish, Arabic, Hindi, Japanese, Chinese)
- Accessibility: colorblind modes (protanopia, deuteranopia, tritanopia), font scaling, reduced motion, high contrast
- Organization branding customization (logo, name, colors) — works for any use case
- Homeoffice integration with recurring patterns and auto-release
- Vehicle management with photo upload
- Waitlist system with automatic notifications
- iCal export for calendar subscriptions
- Admin dashboard with statistics, reports, and CSV export
- User management with roles and departments
- GDPR compliance: data export, account deletion, privacy policy
- PWA support for mobile installation
- REST API with 40+ endpoints
- Prometheus metrics endpoint
- Health check endpoints (liveness, readiness)
- Rate limiting per IP
- Welcome screen with language selection on first start
- Auto-onboarding wizard (password, use-case, organization, dummy data, registration mode)
- 5 use-case modes: Corporate, Residential, Family, Rental, Public
- Layout editor improvements with auto-numbering and auto-labels
- JWT authentication with refresh tokens
- Built-in TLS support
- Push notification subscriptions
- Single binary deployment with embedded redb database
- Docker support with multi-stage build (~20 MB image)
- Comprehensive documentation suite
