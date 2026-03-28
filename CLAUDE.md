# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Caby** is a full-stack distributed file storage and management web application. It consists of:
- `caby-service/` — Rust backend (Axum + Tokio), serves REST API on port 8080
- `caby-web/` — SvelteKit frontend (Svelte 5 with runes), served by Vite on port 5173

## Development Commands

### Running Both Services (Recommended)
```sh
make tmux        # Opens both services in a tmux split pane
make run         # Runs backend only (cargo watch)
```

### Backend (`caby-service/`)
```sh
cargo watch -q -c -w src/ -x run   # Dev with auto-reload (used by make run)
cargo build                         # Build
cargo build --release               # Production build
```
Requires nightly Rust toolchain for rustfmt: `rustup toolchain install nightly --allow-downgrade -c rustfmt`

### Frontend (`caby-web/`)
```sh
pnpm run dev        # Dev server with HMR
pnpm run build      # Production build
pnpm run preview    # Preview production build
pnpm run check      # TypeScript + Svelte type checking
pnpm run lint       # Prettier + ESLint
pnpm run format     # Auto-format with Prettier
```
Uses `pnpm` (enforced via preinstall hook — do not use npm or yarn).

## Architecture

### Backend Structure (`caby-service/src/`)
- `main.rs` — Entry point, sets up Axum router
- `web/` — HTTP handlers organized by resource:
  - `files_api/` — CRUD file operations (list, upload, download, delete, move, rename)
  - `spaces_api/` — Space management
- `files/` — Core file system logic (entry structs, directory overview, pretty printing)
- `space/` — Space resolution from config
- `config/` — YAML-based config loading and validation
- `auth/auth_middleware.rs` — Bearer token authentication middleware
- `jsend.rs` — JSend response format (`{ status, data|message }`)
- `error.rs` — Unified error type that maps to HTTP responses

### Frontend Structure (`caby-web/src/`)
- `routes/(app)/files/[space]/[...path]/` — Main file browser page and its dialogs (Delete, Rename, Move, NewFolder)
- `lib/api/` — API client abstractions:
  - `client.ts` — Core `ApiClient` and `ApiRequestBuilder` pattern
  - `api_files.ts`, `api_spaces.ts` — Typed wrappers per resource
- `lib/files/` — File management state and components:
  - `upload/` — Chunked upload system with web workers and progress tracking
  - `overview/` — Directory listing components (Nav and Select variants)
  - `select.ts` — Multi-selection state
  - `ContextMenu.svelte` — Right-click context menu
- `lib/stores/client.svelte.ts` — Global API client store (Svelte 5 runes)

### API Conventions
- All endpoints under `/v0/` prefix
- Responses use JSend format: `{ "status": "success|fail|error", "data": {...} | "message": "..." }`
- File paths: `/v0/files/{space}/{...path}`
- Space list: `GET /v0/spaces`

### File System Layout
The backend stores data at `CABY_HOME_PATH` (default: `~/cabynet`):
```
{space}/
  live/     — actual files
  meta/     — metadata (mirrors live/ structure)
  uploads/  — temporary chunked upload storage
  shares/   — share metadata
```
Directory metadata uses `.cabydir` files (configurable via `CABY_DIRECTORY_META_FILENAME`).

### Environment Variables (Backend)
Configured in `caby-service/.env`:
- `RUST_LOG` — Logging level (e.g. `debug`)
- `CABY_HOME_PATH` — Root storage path
- `CABY_DIRECTORY_META_FILENAME` — Metadata filename per directory

## Key Patterns

- **Svelte 5 runes**: Frontend uses `$state`, `$derived`, `$effect` — not the older Svelte 4 store/reactive syntax
- **Path traversal prevention**: Backend uses `PathClean` to validate all file paths before filesystem access
- **Upload flow**: Files are chunked on the client (via web workers + xxhash for integrity), uploaded to `uploads/`, then finalized
- **Two component variants**: Several UI components come in `*Nav` (navigation mode) and `*Select` (multi-select mode) variants
