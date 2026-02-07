# Contributing to ParkHub

Thanks for your interest in contributing! 🎉

## Development Setup

### Prerequisites
- Rust 1.75+ (`rustup` recommended)
- Node.js 20+
- npm

### Getting Started
```bash
git clone https://github.com/nash87/parkhub.git
cd parkhub

# Frontend
cd parkhub-web
npm install
npm run dev          # Dev server on :5173
cd ..

# Backend
cargo run            # Dev server on :8080
```

The backend proxies to the frontend dev server during development.

## Project Structure
```
parkhub/
├── src/             # Rust backend
│   ├── api/         # REST API handlers
│   ├── auth/        # Authentication & JWT
│   ├── db/          # redb database layer
│   └── models/      # Data models
├── parkhub-web/     # React frontend
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   └── i18n/
│   └── public/
├── Dockerfile
└── docker-compose.yml
```

## Coding Standards

### Rust
- Follow standard `rustfmt` formatting (`cargo fmt`)
- Run `cargo clippy` before committing
- Write doc comments for public APIs

### TypeScript/React
- ESLint + Prettier (configured in `parkhub-web/`)
- Functional components with hooks
- Use Tailwind CSS for styling

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add multi-day booking support
fix: correct slot availability calculation
docs: update API documentation
style: format code with prettier
refactor: extract booking validation logic
test: add unit tests for auth module
chore: update dependencies
```

### Scope (optional)
```
feat(api): add vehicle photo upload endpoint
fix(web): fix dark mode toggle persistence
```

## Pull Request Process

1. **Fork & branch** — Create a feature branch from `main`
2. **Small PRs** — Keep changes focused and reviewable
3. **Test** — Make sure `cargo test` and `npm run build` pass
4. **Describe** — Write a clear PR description explaining what and why
5. **Review** — Address feedback promptly

### PR Title
Use conventional commit format for the PR title:
```
feat: add parking lot capacity warnings
```

## Reporting Issues

- Use GitHub Issues
- Include steps to reproduce
- Include browser/OS info for frontend bugs
- Include `RUST_LOG=debug` output for backend bugs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
