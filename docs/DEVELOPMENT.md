# Development Guide

## Project Structure

```
parkhub/
├── parkhub-common/       # Shared types and utilities
│   └── src/
├── parkhub-server/       # Rust backend (Axum)
│   └── src/
│       ├── main.rs       # Entry point, CLI args
│       ├── api.rs        # All API route handlers
│       ├── db.rs         # redb database layer
│       ├── auth/         # JWT authentication
│       └── config.rs     # Configuration parsing
├── parkhub-web/          # React frontend
│   ├── src/
│   │   ├── pages/        # Page components
│   │   ├── components/   # Reusable UI components
│   │   ├── stores/       # Zustand state stores
│   │   ├── i18n/         # Translations (de, en)
│   │   └── App.tsx       # Router and app shell
│   ├── package.json
│   └── vite.config.ts
├── config/               # Default config files
├── docs/                 # Documentation
├── Cargo.toml            # Workspace root
├── Dockerfile            # Multi-stage Docker build
└── docker-compose.yml
```

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
