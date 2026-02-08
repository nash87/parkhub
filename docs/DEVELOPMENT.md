# Development Guide

## Project Structure

```
parkhub/
├── src/                        # Desktop client (Slint UI)
│   ├── main.rs                 # Desktop entry point
│   ├── api/                    # API client, endpoints, models
│   ├── auth/                   # Auth module
│   ├── config.rs               # Config parsing
│   ├── database/               # Local database (schema, repository)
│   └── layout_storage.rs       # Parking layout persistence
├── parkhub-server/             # HTTP server (the main binary)
│   └── src/
│       ├── main.rs             # Entry point, CLI args, server startup
│       ├── api.rs              # All API route handlers
│       ├── db.rs               # redb database layer
│       ├── jwt.rs              # JWT module (available, currently unused)
│       ├── config.rs           # Configuration parsing
│       ├── error.rs            # Error types and response mapping
│       ├── health.rs           # Health/liveness/readiness endpoints
│       ├── metrics.rs          # Prometheus metrics
│       ├── rate_limit.rs       # Per-IP and per-user rate limiting
│       ├── static_files.rs     # Embedded frontend serving (rust_embed)
│       ├── tls.rs              # Built-in TLS support
│       ├── email.rs            # SMTP notifications
│       ├── audit.rs            # Audit logging
│       ├── background_jobs.rs  # Periodic tasks (cleanup, notifications)
│       ├── discovery.rs        # mDNS service discovery
│       ├── openapi.rs          # Swagger/OpenAPI spec generation
│       ├── requests.rs         # Request/response types
│       └── validation.rs       # Input validation
├── parkhub-web/                # React frontend
│   ├── src/
│   │   ├── pages/              # Page components
│   │   ├── components/         # Reusable UI components
│   │   ├── stores/             # Zustand state stores
│   │   ├── i18n/               # Translations (de, en)
│   │   └── App.tsx             # Router and app shell
│   ├── package.json
│   └── vite.config.ts
├── parkhub-client/             # Desktop client connection logic
│   └── src/
│       ├── main.rs
│       ├── discovery.rs        # mDNS server discovery
│       └── server_connection.rs
├── parkhub-common/             # Shared types across crates
│   └── src/
│       ├── lib.rs
│       ├── models.rs           # Shared data models
│       ├── error.rs            # Common error types
│       └── protocol.rs         # Client-server protocol types
├── ui/                         # Slint UI definitions (desktop)
├── config/                     # Default config files
├── docs/                       # Documentation
├── build.rs                    # Build script (Slint compilation)
├── Cargo.toml                  # Workspace root
├── Dockerfile                  # Multi-stage Docker build
└── docker-compose.yml
```

## How the Database Works

ParkHub uses [redb](https://github.com/cberner/redb), an embedded key-value store written in pure Rust.

Key properties:
- **Single file** — all data lives in one `parkhub.redb` file
- **ACID transactions** — reads and writes are transactional
- **No migrations** — data is stored as serialized structs (via serde/bincode). Schema changes happen in application code.
- **No external process** — no database server to install or manage
- **Copy-on-write B-tree** — crash-safe without a WAL

The database layer is in `parkhub-server/src/db.rs`. Tables are defined as redb `TableDefinition` constants. Each table maps a key (usually a UUID) to a serialized value.

To back up the database, copy the `.redb` file while the server is stopped — or use file-system snapshots.

## How the Frontend Gets Embedded

The build process:

1. `cd parkhub-web && npm run build` — Vite builds the React app to `parkhub-web/dist/`
2. `cargo build --package parkhub-server` — The server compiles with [rust_embed](https://github.com/pyrossh/rust-embed)
3. `rust_embed` includes every file from `parkhub-web/dist/` into the binary at compile time
4. At runtime, `static_files.rs` serves these embedded files. Non-file paths fall back to `index.html` for SPA routing.

This means the final binary contains the entire frontend. No separate web server needed.

## Authentication Flow

ParkHub uses **UUID session tokens** stored in redb (not stateless JWTs):

1. User sends `POST /api/v1/auth/login` with username + password
2. Server verifies password against Argon2id hash in the database
3. Server generates a UUID `access_token` and a `refresh_token`, stores both in redb with an expiry timestamp
4. Client receives both tokens. The `access_token` expires after 24 hours (configurable). The `refresh_token` lasts 30 days.
5. Client sends `Authorization: Bearer <access_token>` on every request
6. Server looks up the token in redb. If expired or missing → 401.
7. When the access token expires, client sends the refresh token to `POST /api/v1/auth/refresh` to get a new pair.

A `jwt.rs` module exists in the codebase but is currently unused — the server uses opaque UUID tokens instead.

## Setting Up the Dev Environment

### Prerequisites

- Rust 1.83+ (via [rustup](https://rustup.rs))
- Node.js 22+ and npm
- Git

### Backend

```bash
# Start the server in dev mode with auto-reload
cargo watch -x "run --package parkhub-server"

# Or without cargo-watch:
cargo run --package parkhub-server
```

The server runs on `http://localhost:7878` by default.

### Frontend

```bash
cd parkhub-web
npm ci
npm run dev
```

Vite dev server runs on `http://localhost:5173` with hot module replacement. API requests proxy to the backend.

## Adding New API Endpoints

1. Define the handler in `parkhub-server/src/api.rs`:

```rust
async fn my_endpoint(
    State(db): State<Arc<Database>>,
    auth: AuthUser,
) -> Result<Json<MyResponse>, AppError> {
    // ...
}
```

2. Register the route:

```rust
// In the router builder
.route("/api/v1/my-endpoint", get(my_endpoint))
```

3. Add to authenticated or public routes as appropriate.

## Adding Translations

Edit the translation files in `parkhub-web/src/i18n/`:

```typescript
// de.ts
export default {
  "myFeature.title": "Mein Feature",
  "myFeature.description": "Beschreibung",
};

// en.ts
export default {
  "myFeature.title": "My Feature",
  "myFeature.description": "Description",
};
```

Use in components:

```tsx
const { t } = useTranslation();
return <h1>{t("myFeature.title")}</h1>;
```

## Testing

```bash
# Backend tests
cargo test

# Frontend tests
cd parkhub-web && npm test
```

## Code Style

- **Rust:** Follow standard `rustfmt` formatting (`cargo fmt`)
- **TypeScript:** ESLint + Prettier (configured in `parkhub-web/`)
- **Commits:** Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`)

---

Back to [README](../README.md) · Previous: [Deployment](DEPLOYMENT.md) · Next: [Themes](THEMES.md)
