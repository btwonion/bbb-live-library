# CLAUDE.md — BBB Live Library

## Project Overview

A platform for storing, categorizing, and browsing BigBlueButton (BBB) live recordings. Rust backend (Axum) + React frontend (Vite + TypeScript).

## Project Structure

```
backend/                -- Rust backend (Axum + SQLite)
  src/
    api/                -- API route handlers (categories, import, playback, recordings, schedules, stats, tags)
    bbb/                -- BBB API client
    capture/            -- Recording capture via ffmpeg
    config.rs           -- Config file parsing
    db.rs               -- Database pool and migrations
    error.rs            -- AppError type
    models.rs           -- Shared data models
    main.rs             -- Entrypoint, router setup, background tasks
  migrations/           -- SQLx migrations (run automatically on startup)
frontend/               -- React + TypeScript frontend (Vite)
  src/
    api/                -- API client and types
    components/         -- Reusable UI components
    hooks/              -- Custom React hooks
    lib/                -- Utility functions
    pages/              -- Page-level components
config.docker.toml      -- Config for containerized deployments
docker-compose.yml      -- Docker Compose setup
Dockerfile              -- Multi-stage build (frontend + backend)
```

## Commands

### Backend
```bash
cd backend && cargo build            # Build
cd backend && cargo run              # Run dev server
cd backend && cargo test             # Run tests
cd backend && cargo clippy           # Lint
cd backend && cargo fmt              # Format
cd backend && cargo sqlx migrate run # Run migrations manually
```

### Frontend
```bash
cd frontend && npm install           # Install dependencies
cd frontend && npm run dev           # Dev server
cd frontend && npm run build         # Production build
cd frontend && npm run lint          # Lint
```

### Docker
```bash
docker compose up --build            # Build and run
docker compose up -d                 # Run in background
```

## Configuration

The backend loads `config.toml` from the working directory by default. For Docker, `config.docker.toml` is used instead.

Config sections:
- `[server]` — host, port, `frontend_dir` (path to built frontend assets)
- `[database]` — SQLite connection URL
- `[bbb]` — BBB server URL, shared secret, import interval
- `[capture]` — storage directory, ffmpeg path, output format, retry interval

For local development, create a `config.toml` based on `config.docker.toml` with local paths and your BBB server credentials.

## Code Conventions

### Rust (Backend)
- Use `anyhow` for application errors, custom error types only for API responses
- All API handlers return `Result<Json<T>, AppError>` where `AppError` implements `IntoResponse`
- Use `sqlx::query_as!` for type-checked queries wherever possible
- Prefer `&str` over `String` in function parameters
- All public functions need doc comments; internal functions don't
- Group imports: std → external crates → local modules
- No `unwrap()` in production code — use `?` or explicit error handling
- Use `tracing` for logging, not `println!`
- Keep handlers thin: extract business logic into service functions
- Async all the way — never block the tokio runtime with sync I/O

### TypeScript (Frontend)
- Functional components only, no class components
- Use TanStack Query for all server state — no manual `useEffect` + `fetch`
- Colocate component-specific types in the same file
- Shared API types go in `src/api/types.ts`
- Use Shadcn/ui components as the base — don't reinvent inputs, dialogs, etc.
- Tailwind for styling — no CSS files, no inline style objects
- Name files in PascalCase for components (`RecordingCard.tsx`), camelCase for utilities (`formatDuration.ts`)
- Prefer `interface` over `type` for object shapes
- Destructure props in function signature

### General
- No dead code — delete unused functions, imports, and variables
- No TODO comments in committed code
- Keep functions short: if a function exceeds ~40 lines, split it
- Naming: be descriptive, avoid abbreviations (except well-known ones like `id`, `url`, `db`)

## Architecture Decisions

- **SQLite** — single-file database, no external DB server needed, sufficient for this workload
- **ffmpeg via subprocess** — more reliable and flexible than Rust ffmpeg bindings; capture and thumbnail generation both use it
- **Axum** — lightweight, tower-based, good ecosystem fit with sqlx and tokio
- **Shadcn/ui** — copy-paste components (not a dependency), full control over styling
- **TanStack Query** — handles caching, refetching, and loading/error states so we don't
- **Background tasks** — BBB recording import loop and capture scheduler run automatically on startup as spawned tokio tasks
- **Multi-stage Docker build** — frontend built with Node, backend built with Rust, final image is minimal with only the binary and static assets

## BBB API Notes

- BBB uses a shared-secret checksum auth scheme: `SHA256(apiCall + queryString + sharedSecret)`
- Key endpoints: `getMeetings` (active meetings), `getRecordings` (published recordings)
- API returns XML by default — parse with `quick-xml` or convert to JSON
- Meeting streams for live capture are typically RTMP — ffmpeg can ingest these directly

## File Storage

- Recordings stored in `{storage_dir}/{recording_id}.{format}`
- Thumbnails stored in `{storage_dir}/thumbs/{recording_id}.jpg`
- Paths in DB are relative to `storage_dir` — never store absolute paths
- When deleting a recording, always delete both the DB row and the files

## Testing

Current state: unit tests exist in `backend/src/bbb/client.rs` only.

Conventions to follow when adding tests:
- Backend: integration tests against an in-memory SQLite database
- No mocking the database — use a real SQLite instance in tests for accuracy
- Frontend: component tests with Vitest + Testing Library for critical flows

## Gotchas

- **ffmpeg** is a required runtime dependency — must be installed on the host or available in the container
- **Frontend in production** is served as static files by the backend from `frontend/dist` — run `npm run build` before the backend can serve it
- **Migrations** run automatically on startup — no manual migration step needed in normal operation
- **Config file** is loaded from `config.toml` in the working directory by default
