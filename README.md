# BBB Live Library

Record, archive, and rewatch BigBlueButton sessions on your own terms.

A self-hosted Rust + React application that captures live BigBlueButton (BBB) meetings — including ones the host never enabled server-side recording for — and organises them into a searchable, categorised video library.

---

## Motivation

A growing share of university courses are delivered over BigBlueButton. In theory, missing a lecture is fine — you can just watch the recording later. In practice, the **recording flag is set by the lecturer**, and many of them never turn it on. When that happens and you can't attend live, the material is simply gone.

This project exists to close that gap. It joins a BBB room as an ordinary participant, captures whatever is on screen and whatever audio the room produces, and stores it as a normal MP4 file in a personal library you control. It also imports the recordings that *do* exist (from a public BBB server, or any direct video URL) so everything you care about lives in one place — searchable, filterable by course, and playable from a browser without leaving the app.

---

## Features

- **Two capture modes**
  - *Direct stream*: when the room exposes an RTMP feed, ffmpeg ingests it directly — lightweight and resource-efficient.
  - *Browser-based*: a headless Chromium (driven by Playwright) joins the room inside a virtual X display; ffmpeg grabs the screen via `x11grab` and records the audio from a virtual PulseAudio sink. Works for rooms that don't expose a stream at all.
- **Scheduled recording** — define cron-style schedules with start and end offsets; a background scheduler handles the rest, with concurrent captures isolated on separate X displays.
- **Imports** — pull existing recordings from public BBB servers or any direct video URL; SHA-256 file hashing deduplicates across all import paths.
- **Library browser** — dashboard with stats and recent items, paginated grid/list views, full-text search, category filtering.
- **Built-in player** — HTTP range-request streaming (`206 Partial Content`) for instant seeks; auto-generated JPEG thumbnails.
- **Categories** — tag recordings into user-defined groups (one per course, for example).
- **Single-image deploy** — one Docker image bundles backend, frontend assets, recorder script, and every runtime dependency.

---

## Architecture

```
                  ┌──────────────────────┐
   Browser ──────►│  React SPA (Vite)    │
                  │  Tailwind + Shadcn   │
                  └──────────┬───────────┘
                             │ HTTPS / fetch
                  ┌──────────▼───────────┐
                  │  Axum REST API       │
                  │  (Rust, Tokio)       │
                  └────┬─────────┬───────┘
                       │         │
              ┌────────▼──┐  ┌───▼──────────────┐
              │  SQLite   │  │  Capture engine  │
              │  + files  │  │  (background     │
              │  on disk  │  │   tokio tasks)   │
              └───────────┘  └───┬──────────────┘
                                 │
                ┌────────────────┴─────────────────┐
                │                                  │
        ┌───────▼────────┐              ┌──────────▼──────────┐
        │ Direct ffmpeg  │              │ Browser pipeline    │
        │ (RTMP ingest)  │              │ Xvfb + PulseAudio + │
        │                │              │ Playwright + ffmpeg │
        └────────────────┘              └─────────────────────┘
```

The backend (`backend/src/main.rs`) wires up the Axum router, applies SQLx migrations on startup, and spawns the capture scheduler (`backend/src/capture/scheduler.rs`) as a background task. API handlers live under `backend/src/api/`. The two capture pipelines are implemented in `backend/src/capture/recorder.rs` (direct) and `backend/src/capture/browser_recorder.rs` (browser); the latter spawns `recorder/record.js` as a Node subprocess.

The frontend is a standard SPA: pages in `frontend/src/pages/`, server state managed by TanStack Query, components built on Shadcn/ui + Base UI, all styled with Tailwind. In production, the backend serves the built `frontend/dist` directly.

---

## Tech stack

| Layer       | Stack                                                                                 |
|-------------|---------------------------------------------------------------------------------------|
| Backend     | Rust 2021, Axum 0.8, Tokio, SQLx 0.8 (compile-time-checked queries), tracing, anyhow, reqwest, quick-xml, cron |
| Frontend    | React 19, Vite 8, TypeScript 5.9, TanStack Query 5, React Router 7, Tailwind CSS 4, Shadcn/ui, Base UI, Lucide, Sonner |
| Capture     | ffmpeg, Xvfb, PulseAudio, Node 22 + Playwright 1.52                                   |
| Storage     | SQLite (single file), filesystem for media                                            |
| Packaging   | Multi-stage Docker build (Node 22 → Rust 1.85 → Debian bookworm-slim)                |

---

## Quick start (Docker)

The fastest way to run the whole stack:

```bash
docker compose up --build
```

Then open <http://localhost:8080>.

What this gets you:

- A running server on port `8080` with the SPA, API, and capture engine.
- A persistent `./data` directory holding the SQLite database and all recordings.
- Configuration loaded from `config.docker.toml` (storage at `/data`, timezone `Europe/Berlin`).

To start capturing:

1. Open the app, go to **Settings**, and add a BBB import source (server URL + shared secret). BBB connection info lives in the database, not in the config file.
2. Either trigger an import from the **Settings** / **Recordings** views, or create a **Schedule** for a live capture (room URL, start time, optional cron recurrence, capture offsets).

---

## Local development

You'll want three terminals: backend, frontend, and (the first time only) recorder setup.

**Backend** — needs `ffmpeg` on `PATH` and a `config.toml` in the working directory (use `config.docker.toml` as a template, adjusted for local paths):

```bash
cd backend
cargo run                # migrations apply automatically on startup
cargo clippy             # lint
cargo fmt                # format
cargo test               # unit tests
```

**Frontend** — Node 22+ required:

```bash
cd frontend
npm install
npm run dev              # Vite dev server
npm run build            # production build into frontend/dist
npm run lint
```

**Recorder** — install Playwright + Chromium once:

```bash
cd recorder
npm install
```

The backend spawns `recorder/record.js` as a subprocess when a browser-based capture is needed; the path is configured via `[capture] recorder_script_path`.

---

## Configuration

The backend reads `config.toml` from the working directory by default. The Docker image uses `config.docker.toml`. Sections:

- `[server]` — `host`, `port`, `frontend_dir` (path to built frontend assets), `timezone` (IANA name, e.g. `Europe/Berlin`).
- `[database]` — SQLite connection URL (e.g. `sqlite:/data/bbb-library.db`).
- `[capture]` — `storage_dir`, `ffmpeg_path`, `output_format`, `retry_interval_secs`, `recorder_script_path`.

Example: see [`config.docker.toml`](./config.docker.toml).

BBB servers and credentials are **not** in the config file — they're managed at runtime from the Settings page and stored in the database.

---

## API surface

<details>
<summary>HTTP endpoints (click to expand)</summary>

All endpoints are JSON unless noted. Pagination uses `?page=&per_page=`.

**Health**
- `GET  /api/health`

**Recordings**
- `GET    /api/recordings` — paginated list (`search`, `category_id` filters)
- `GET    /api/recordings/{id}` — detail
- `POST   /api/recordings/{id}` — update title / description
- `DELETE /api/recordings/{id}`
- `POST   /api/recordings/{id}/categories` — assign categories

**Playback**
- `GET /api/recordings/{id}/stream` — video bytes (HTTP range requests supported)
- `GET /api/recordings/{id}/thumbnail` — JPEG

**Categories**
- `GET    /api/categories`
- `POST   /api/categories`
- `PUT    /api/categories/{id}`
- `DELETE /api/categories/{id}`

**Schedules**
- `GET    /api/schedules` — paginated
- `POST   /api/schedules`
- `GET    /api/schedules/{id}`
- `PUT    /api/schedules/{id}`
- `DELETE /api/schedules/{id}`

**Imports**
- `POST /api/import/url` — import from a direct video URL
- `POST /api/import/bbb-public` — import from a public BBB recording

**Stats / Settings**
- `GET /api/stats`
- `GET /api/settings/timezone`

</details>

---

## Project status & limitations

This is a personal project, not a multi-tenant product:

- Single-user; no auth layer in front of the API. Don't expose it to the public internet without putting it behind something.
- Tests cover only the BBB XML parsing module today; no CI workflows yet.
- Browser-based capture relies on Xvfb + PulseAudio, so that mode is **Linux-only at runtime**. The Docker image handles all of this for you.
- ffmpeg is a hard runtime dependency.

**Legal note**: capturing a live session you don't have permission to record may violate course policy or local law. Whether and how you use this is your responsibility — make sure you have the right to record before you do.

---

## License

GPL-3.0. See `LICENSE` for the full text.

---

*Built with the help of [Claude Code](https://claude.com/claude-code) — the architecture, code, and this document were produced through iterative collaboration with an AI coding agent.*
