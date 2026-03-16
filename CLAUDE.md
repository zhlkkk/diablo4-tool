# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Diablo IV build automation desktop tool. Users paste a d2core.com build link, the app parses it, and applies skills/paragon selections to their character via simulated UI clicks. Windows-only (Win32 APIs).

**Stack:** Rust + Tauri 2 backend, React 19 + TypeScript frontend, Vite 6 bundler.

## Build & Dev Commands

```bash
# Combined dev (Vite hot-reload + Rust debug build)
npm run tauri dev

# Frontend only
npm run dev              # Vite dev server on localhost:1420

# Backend only (from src-tauri/)
cargo build
cargo clippy             # Lint

# Release build
npm run tauri build
```

## Testing

All tests are offline using fixture files (no network). Run from `src-tauri/`:

```bash
cargo test                    # All tests
cargo test web_parser         # Web parser tests only
cargo test -- --ignored       # Integration tests (network-dependent, skip in CI)
```

Test fixtures live in `src-tauri/tests/fixtures/` (JSON API responses).

## Architecture

```
Frontend (React/TS in src/)
  ↕ Tauri IPC (invoke/listen)
Backend (Rust in src-tauri/src/)
  ├── web_parser    — Fetch & parse d2core.com builds via Tencent CloudBase API
  ├── game_capture  — Win32 FindWindow, DPI detection, BitBlt screenshot capture
  ├── safety        — Pixel-sampling UI state detection, F10 emergency hotkey
  ├── auto_applier  — Click sequence generation, resolution-adaptive execution
  └── types.rs      — Shared data structures (BuildPlan, CalibrationData, GameState)
```

Each backend module has its own `mod.rs`, concern-separated files, and a `thiserror`-derived `Error` enum.

## Key Patterns

- **Tauri commands** in `lib.rs` are thin wrappers that delegate to module logic and convert errors to `Result<T, String>`
- **Shared state** via `AppState` in `Mutex`, injected through Tauri's `State<>` parameter
- **CalibrationData** stores pixel coordinates at capture resolution; `coords.rs` scales at runtime via resolution factor
- **Safety gates**: `assert_safe_state()` called before each click step to detect unsafe game states
- **Frontend events**: `apply_progress`, `safety_event`, `apply_complete` emitted from backend via Tauri event system
- **Bilingual UI**: Chinese/English strings throughout the React frontend
- **Response double-parse**: d2core API returns `response_data` as a JSON string inside JSON — requires two parse passes

## Tauri IPC Commands

`parse_build_link`, `get_game_state`, `start_apply`, `pause_apply`, `resume_apply`, `capture_game_screenshot`, `load_calibration`, `save_calibration`, `check_safety`, `reset_emergency_stop`

## Platform Constraints

- **Windows-only**: Uses Win32 APIs (FindWindow, BitBlt, SendInput) with `#[cfg(target_os = "windows")]` gates
- **Anti-cheat aware**: Humanized click timing, safety checks before automation, no memory manipulation
- **DPI scaling**: Resolution-adaptive coordinate math handles different display configurations
