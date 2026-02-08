# Security

## Authentication

ParkHub uses **UUID session tokens**, not JWTs. There's a JWT module in the codebase (`jwt.rs`), but it's unused — kept for a potential future migration to stateless auth.

Here's how login works:

1. Client sends `POST /api/v1/auth/login` with `{username, password}`
2. Server looks up the user in redb via the `users_by_username` index. If not found, tries `users_by_email`.
3. Password is verified against the stored Argon2id hash (using the `argon2` crate with `OsRng`-generated salt)
4. On success: server generates a UUID v4 as the access token, creates a `Session` struct with the user's ID, role, a refresh token (`rt_<uuid>`), and a 24-hour expiry
5. Session is stored in redb's `sessions` table, keyed by the access token
6. Client stores the token and sends `Authorization: Bearer <uuid>` on every request

The auth middleware (`auth_middleware` in `api.rs`) runs on every protected route. It extracts the Bearer token, reads the session from the `sessions` table, checks expiry, and populates an `AuthUser` struct with `id`, `username`, and `role`.

## Password Hashing

Argon2id with default parameters from the `argon2` crate:

- Algorithm: Argon2id (hybrid — resistant to both side-channel and GPU attacks)
- Salt: 16 bytes from `OsRng`
- Memory: 19 MiB (default)
- Iterations: 2 (default)
- Parallelism: 1 (default)

The hash is stored in PHC string format: `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`

## Database Encryption

Optional AES-256-GCM encryption at rest, enabled by setting `PARKHUB_DB_PASSPHRASE`:

1. On first run, a 32-byte random salt is generated and stored in the `settings` table (this one value is unencrypted)
2. Passphrase + salt → PBKDF2-SHA256 (600K iterations) → 256-bit encryption key
3. Every value written to any redb table is encrypted: random 96-bit nonce + AES-256-GCM ciphertext
4. Nonce is prepended to the stored bytes

Table keys (UUIDs, usernames) are **not** encrypted — only values. This means someone with disk access can see user IDs and usernames but not the actual user data, bookings, etc.

## Input Validation

- All request bodies are deserialized through Serde with typed structs. Unexpected fields are ignored, missing required fields return `400`.
- Path parameters (UUIDs) are parsed and validated before any database lookup.
- Request body size is limited by Axum defaults (2 MB).
- String fields are not blindly trusted — the frontend sanitizes display, and the API returns `Content-Type: application/json` so there's no HTML injection vector.

## XSS Prevention

- React auto-escapes all JSX interpolation. No `dangerouslySetInnerHTML` usage.
- API responses are always JSON with `Content-Type: application/json`. Even if someone stored malicious HTML, it'd be sent as a JSON string, not rendered.
- CSP headers restrict script sources to `'self'`.

## Rate Limiting

Tower-based middleware, applied per IP address:

| Scope | Default |
|-------|---------|
| All API requests | 60/min per IP |
| Login endpoint | 10/min per IP |

Returns `429 Too Many Requests` when exceeded with a `Retry-After` header.

## Security Headers

Set on all responses:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 0          (modern browsers, CSP is preferred)
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'
```

## GDPR / DSGVO

ParkHub is designed for German/EU companies. GDPR compliance features:

- **Data export** (`GET /api/v1/users/me/export`): Returns all user data as JSON — profile, bookings, vehicles, homeoffice settings
- **Account deletion** (`DELETE /api/v1/users/me/delete`): Permanently removes the user and all associated data from all redb tables
- **Privacy policy** (`GET /api/v1/privacy`): Configurable text served to users
- **No external calls**: Zero analytics, no CDNs, no tracking pixels, no third-party anything. The binary serves everything locally.
- **Data stays on your server**: Self-hosted only. No cloud, no telemetry, no phoning home.

## Responsible Disclosure

Found a vulnerability? Don't open a public issue.

1. Email **security@parkhub.dev** or use [GitHub Security Advisories](https://github.com/nash87/parkhub/security/advisories)
2. Include: what you found, how to reproduce it, what the impact is
3. We'll respond within 48 hours and aim to patch within 7 days

---

Back to [README](../README.md) · Previous: [Accessibility](ACCESSIBILITY.md)
