# CLAUDE.md — BBB Live Library

## Project Overview

A platform for storing, categorizing, and browsing BigBlueButton (BBB) live recordings. Rust backend (Axum) + React frontend (Vite + TypeScript).

## Project Structure

```
recorder/               -- Node.js Playwright script for browser-based capture
  record.js             -- Joins BBB rooms and enables screen/audio capture
  package.json          -- Playwright dependency
backend/                -- Rust backend (Axum + SQLite)
  src/
    api/                -- API route handlers (categories, import, playback, recordings, schedules, stats, tags)
    bbb/                -- BBB API client
    capture/            -- Recording capture (ffmpeg direct + browser-based via Playwright)
      browser_recorder.rs -- Browser capture pipeline (Xvfb + PulseAudio + ffmpeg)
      common.rs         -- Shared capture utilities
      recorder.rs       -- Direct ffmpeg stream capture
      scheduler.rs      -- Schedule-based capture orchestration
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
    assets/             -- Static assets (fonts, images)
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
- `[capture]` — storage directory, ffmpeg path, output format, retry interval, `recorder_script_path` (path to Node.js browser recorder script)

BBB server connections are managed via import sources in the database (added through the Settings page), not the config file.

For local development, create a `config.toml` based on `config.docker.toml` with local paths.

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
- **ffmpeg via subprocess** — more reliable and flexible than Rust ffmpeg bindings; used for direct stream capture, browser screen/audio capture, and thumbnail generation
- **Browser-based capture** — for rooms without direct RTMP streams, a Playwright script (`recorder/record.js`) joins the BBB room in a headless browser (Xvfb), captures screen via ffmpeg x11grab and audio via PulseAudio virtual sink
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

Current state: unit tests exist in `backend/src/bbb/public.rs` only.

Conventions to follow when adding tests:
- Backend: integration tests against an in-memory SQLite database
- No mocking the database — use a real SQLite instance in tests for accuracy
- Frontend: component tests with Vitest + Testing Library for critical flows

## Playwright Frontend Testing (Required)

After any changes to files in `frontend/src/` or `backend/src/api/`, you **must** run a Playwright smoke test using the Playwright MCP tools before considering the task complete. This ensures no regressions in the UI or API integration.

### Prerequisites
1. Build the frontend: `cd frontend && npm run build`
2. Start the backend: `cd <project-root> && cargo run --manifest-path backend/Cargo.toml`
3. Wait for `http://localhost:8080/api/health` to return `{"status":"ok"}`

### Test checklist
Run these checks using the Playwright MCP browser tools:

1. **Dashboard** (`http://localhost:8080/`)
   - Page loads, stats cards render, recent recordings appear
   - Zero console errors (`browser_console_messages` with level `error`)

2. **Recordings page** (`http://localhost:8080/recordings`)
   - Grid view shows recording cards with thumbnails
   - List view toggle works (`?view=list`)
   - Search filters results
   - Zero console errors

3. **Video playback** (`http://localhost:8080/recordings/{id}`) — **most critical**
   - Video element has `readyState >= 3` (enough data to play) and `error: null`
   - `video.play()` advances `currentTime` (verify after ~1s delay)
   - Seeking (`video.currentTime = N`) works without errors
   - Stream endpoint returns `206 Partial Content` for range requests
   - Zero console errors

4. **Other pages** (Schedules, Categories, Settings)
   - Pages load without console errors

### How to check video state
```js
// Use browser_evaluate with this function:
() => {
  const video = document.querySelector('video');
  if (!video) return 'No video element found';
  return {
    readyState: video.readyState,
    error: video.error ? { code: video.error.code, message: video.error.message } : null,
    duration: video.duration,
    paused: video.paused,
    currentTime: video.currentTime,
  };
}
```

### Failure handling
If any check fails, fix the issue before completing the task. Common issues:
- Font 404s → check `frontend/dist/assets/files/` for woff2 files
- SPA route 404 in console → backend must use `ServeDir::fallback()` not `.not_found_service()`
- Video won't load → check `/api/recordings/{id}/stream` returns 200/206

## Gotchas

- **ffmpeg** is a required runtime dependency — must be installed on the host or available in the container
- **Frontend in production** is served as static files by the backend from `frontend/dist` — run `npm run build` before the backend can serve it
- **Migrations** run automatically on startup — no manual migration step needed in normal operation
- **Config file** is loaded from `config.toml` in the working directory by default
- **Browser capture dependencies** — browser-based recording requires Xvfb, PulseAudio, and Node.js with Playwright installed at runtime
- **Recorder setup** — run `cd recorder && npm install` to install Playwright; the script is spawned as a subprocess by the backend
