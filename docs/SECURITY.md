# Security

## Authentication and Authorization

- **JWT tokens** with configurable expiry (default: 24 hours)
- **Refresh tokens** for seamless re-authentication (default: 30 days)
- **Password hashing** with Argon2id (memory-hard, side-channel resistant)
- **Role-based access control** — `user` and `admin` roles
- Admin endpoints are protected by role middleware

## Input Validation

- All API inputs are validated and sanitized server-side
- Request body size limits prevent abuse
- Type-safe deserialization via Serde rejects malformed data
- Path parameters are validated before database queries

## XSS Prevention

- React's built-in JSX escaping prevents DOM-based XSS
- API responses use `Content-Type: application/json` — no HTML injection
- User-generated content is sanitized before storage and rendering
- CSP headers restrict inline scripts

## Rate Limiting

Built-in Tower-based rate limiting:

| Scope | Default Limit |
|-------|---------------|
| General API | 60 requests/min per IP |
| Login attempts | 10 attempts/min per IP |

Configure in `config.toml`:

```toml
[rate_limit]
enabled = true
requests_per_minute = 60
login_attempts_per_minute = 10
```

## HTTPS / HSTS

- Built-in TLS support via `config.toml`
- Recommended: TLS termination at reverse proxy (Nginx/Caddy)
- HSTS headers sent when TLS is enabled
- Automatic HTTP → HTTPS redirect

## Security Headers

ParkHub sets the following headers by default:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 0
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; ...
```

## GDPR Compliance

- **Data export** — Users can download all their data as JSON (`GET /api/v1/users/me/export`)
- **Account deletion** — Full account and data removal (`DELETE /api/v1/users/me/delete`)
- **Privacy policy** — Configurable privacy policy page (`GET /api/v1/privacy`)
- **Minimal data collection** — Only essential data is stored
- **No third-party tracking** — Zero external analytics or tracking scripts
- **Data stays on-premise** — Self-hosted, no cloud dependencies

## Responsible Disclosure

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email: **security@parkhub.dev** (or use GitHub Security Advisories)
3. Include: description, reproduction steps, impact assessment
4. We aim to respond within 48 hours and patch within 7 days

---

Back to [README](../README.md) · Previous: [Accessibility](ACCESSIBILITY.md)
