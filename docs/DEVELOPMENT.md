# Development

## Project Structure

```
parkhub/
├── parkhub-common/          Shared types — User, Booking, ParkingLot, Vehicle, etc.
│   └── src/lib.rs           Models + protocol version constant
├── parkhub-server/          The actual server
│   └── src/
│       ├── main.rs          CLI parsing, Tokio runtime, server startup
│       ├── api.rs           All route handlers (~2500 lines, the core of the app)
│       ├── db.rs            redb database layer — all table definitions, CRUD ops
│       ├── jwt.rs           JWT module (exists but unused — sessions use UUID tokens)
│       ├── config.rs        Configuration parsing
│       ├── error.rs         Error types (AppError enum)
│       └── openapi.rs       Swagger/OpenAPI spec generation
├── parkhub-client/          Desktop GUI client (Slint toolkit, optional)
├── parkhub-web/             React frontend
│   ├── src/
│   │   ├── pages/           Route-level components (Login, Dashboard, Bookings, Admin, etc.)
│   │   ├── components/      Reusable UI (ThemeSelector, Layout shell, etc.)
│   │   ├── stores/          Zustand stores
│   │   │   ├── theme.ts     Dark/light mode toggle, persisted to localStorage
│   │   │   └── palette.ts   10 color palettes, applied via CSS custom properties
│   │   ├── i18n/            Translation files (10 languages: ar, de, en, es, fr, hi, ja, pt, tr, zh)
│   │   └── App.tsx          Router (react-router-dom), auth guard, theme init
│   ├── package.json
│   ├── vite.config.ts
│   └── tailwind.config.js
├── build.rs                 Embeds parkhub-web/dist/ into the server binary
├── Cargo.toml               Workspace: parkhub-common, parkhub-server, parkhub-client
├── Dockerfile               Multi-stage: node:22-alpine → rust:1.83-alpine → scratch
└── docker-compose.yml
```

## How the Build Works

1. `npm run build` in `parkhub-web/` produces static files in `dist/`
2. `cargo build` triggers `build.rs`, which uses `include_dir` or similar to embed `parkhub-web/dist/` into the binary
3. At runtime, the Axum server serves these embedded files for any non-API route
4. Result: one binary, no separate frontend deployment

## Backend Dev

```bash
# Run the server with auto-reload on code changes
cargo install cargo-watch    # one-time
cargo watch -x "run --package parkhub-server -- --headless --debug"

# Or just run it directly
cargo run --package parkhub-server -- --headless -p 7878
```

The server starts on port 7878. API is at `/api/v1/*`. The database file gets created in `./data/` by default.

### Adding a New Endpoint

1. Write the handler in `parkhub-server/src/api.rs`:

```rust
async fn my_handler(
    State(state): State<SharedState>,
    auth: AuthUser,               // extracted from Bearer token via middleware
    Json(body): Json<MyRequest>,  // deserialize request body
) -> (StatusCode, Json<ApiResponse<MyResponse>>) {
    let state_guard = state.read().await;
    // use state_guard.db to access the database
    // ...
    (StatusCode::OK, Json(ApiResponse::success(response)))
}
```

2. Register it in `create_router()`:

```rust
// For authenticated routes — add inside the auth-protected router
.route("/api/v1/my-endpoint", post(my_handler))

// For public routes — add to the public router
.route("/api/v1/my-public-endpoint", get(my_public_handler))
```

The auth middleware (`auth_middleware`) runs on all routes in the protected group. It reads the `Authorization: Bearer <uuid>` header, looks up the session in redb, and populates `AuthUser` with the user's ID, username, and role.

### Adding a New Database Table

In `parkhub-server/src/db.rs`:

```rust
// Define the table
const MY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("my_table");

// Add CRUD methods to impl Database
pub async fn save_my_thing(&self, id: &str, thing: &MyThing) -> Result<()> {
    let db = self.db.read().await;
    let db = db.as_ref().ok_or_else(|| anyhow!("DB not open"))?;
    let json = serde_json::to_vec(thing)?;
    let encoded = self.maybe_encrypt(&json)?;  // handles encryption if enabled
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(MY_TABLE)?;
        table.insert(id, encoded.as_slice())?;
    }
    write_txn.commit()?;
    Ok(())
}
```

All values are stored as JSON-serialized bytes. If encryption is enabled (`PARKHUB_DB_PASSPHRASE`), the bytes go through AES-256-GCM before writing.

## Frontend Dev

```bash
cd parkhub-web
npm ci
npm run dev    # Vite dev server on http://localhost:5173
```

Vite proxies `/api/*` to `http://localhost:7878` (configured in `vite.config.ts`), so run the backend separately.

### State Management

Zustand stores in `src/stores/`:

- **theme.ts** — `isDark` boolean, `toggle()` function. Persisted to localStorage as `parkhub-theme`.
- **palette.ts** — `paletteId` string, the 10 palette definitions, `applyPalette()` which sets CSS custom properties on `:root`.

Usage in components:

```tsx
const { isDark, toggle } = useTheme();
const { paletteId, setPalette } = usePalette();
```

### Translations (i18n)

Uses react-i18next. Translation files are TypeScript objects in `src/i18n/de.ts` and `en.ts`.

To add a new string:

```typescript
// de.ts
"myPage.title": "Mein Titel",

// en.ts
"myPage.title": "My Title",
```

```tsx
const { t } = useTranslation();
return <h1>{t("myPage.title")}</h1>;
```

To add a new language, create a new file (e.g., `fr.ts`) and register it in the i18n config.

### Icons

Using [Phosphor Icons](https://phosphoricons.com/) (`@phosphor-icons/react`). They're SVG-based and tree-shakeable.

```tsx
import { Car, Calendar, House } from "@phosphor-icons/react";
<Car weight="fill" className="w-5 h-5" />
```

### Animations

Framer Motion for page transitions, list animations, and interactive elements.

```tsx
import { motion } from "framer-motion";
<motion.div variants={itemVariants} className="card p-6">
  ...
</motion.div>
```

Respects `prefers-reduced-motion` — all animations are disabled when the user or OS requests it.

## Testing

```bash
# Backend
cargo test

# Frontend
cd parkhub-web && npm test
```

## Code Style

- **Rust:** `cargo fmt` (standard rustfmt). No clippy warnings.
- **TypeScript:** ESLint + Prettier (config in `parkhub-web/`).
- **Commits:** [Conventional Commits](https://www.conventionalcommits.org/) — `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`.

---

Back to [README](../README.md) · Previous: [Deployment](DEPLOYMENT.md) · Next: [Themes](THEMES.md)
