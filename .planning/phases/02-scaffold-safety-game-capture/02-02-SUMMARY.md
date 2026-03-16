---
phase: 02-scaffold-safety-game-capture
plan: 02
subsystem: game-capture
tags: [win32, dpi, screenshot, printwindow, findwindow, hwnd, thiserror]

# Dependency graph
requires:
  - phase: 02-01
    provides: "types.rs with Resolution, GameState, AppState; game_capture stub mod.rs"
provides:
  - "find_diablo_window() — locates D4 HWND via FindWindowW + EnumWindows fallback"
  - "is_exclusive_fullscreen() — window style heuristic detection"
  - "is_window_valid() — stale HWND check"
  - "get_game_resolution() — client rect dimensions via GetClientRect"
  - "get_game_dpi() — per-window DPI via GetDpiForWindow"
  - "normalize_coord() — DPI-aware coordinate normalization"
  - "capture_window() — BGRA screenshot via PrintWindow with PW_RENDERFULLCONTENT"
  - "CaptureError — typed error enum for all capture failures"
affects: [safety, auto_applier]

# Tech tracking
tech-stack:
  added: [windows 0.61 Win32 APIs, thiserror]
  patterns: [cfg(windows) guards for cross-platform compilation, pure function extraction for testability]

key-files:
  created:
    - src-tauri/src/game_capture/error.rs
    - src-tauri/src/game_capture/window.rs
    - src-tauri/src/game_capture/dpi.rs
    - src-tauri/src/game_capture/screenshot.rs
  modified:
    - src-tauri/src/game_capture/mod.rs

key-decisions:
  - "Extracted check_fullscreen_style() as pure function taking raw style bits and rect arrays — enables unit testing without Win32 HWND"
  - "Used cfg(windows) guards on all Win32-dependent functions — allows compilation and pure-logic testing on Linux/WSL"

patterns-established:
  - "Pure function extraction: Win32-dependent logic wraps a pure function that takes primitive inputs, enabling cross-platform unit tests"
  - "cfg(windows) module pattern: Win32 imports and functions guarded, tests for pure logic always run"

requirements-completed: [CAPT-01, CAPT-02, CAPT-04, CAPT-05, CAPT-06]

# Metrics
duration: 4min
completed: 2026-03-16
---

# Phase 2 Plan 02: Game Capture Summary

**Win32 game capture module with FindWindowW window detection, DPI normalization, fullscreen heuristics, and PrintWindow screenshot capture — 15 unit tests passing**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-16T11:04:33Z
- **Completed:** 2026-03-16T11:08:13Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Window finding with FindWindowW primary + EnumWindows title-match fallback
- Exclusive fullscreen detection via WS_POPUP/WS_THICKFRAME/WS_CAPTION style bits plus monitor rect comparison
- DPI normalization (normalize_coord) with 100%/125%/150%/200% test coverage
- Screenshot capture via PrintWindow with PW_RENDERFULLCONTENT for BGRA pixel buffer
- 15 unit tests for pure logic (6 fullscreen style, 5 DPI normalization, 4 resolution detection)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create error type and window finding module** - `08241f6` (feat)
2. **Task 2 RED: Failing tests for DPI normalization** - `544bcb4` (test)
3. **Task 2 GREEN: Implement DPI normalization and screenshot** - `ea887e5` (feat)

## Files Created/Modified
- `src-tauri/src/game_capture/error.rs` - CaptureError enum with 6 variants (WindowNotFound, GetRectFailed, PrintWindowFailed, UnsupportedResolution, ExclusiveFullscreen, Win32)
- `src-tauri/src/game_capture/window.rs` - find_diablo_window(), is_exclusive_fullscreen(), is_window_valid(), check_fullscreen_style() + 6 tests
- `src-tauri/src/game_capture/dpi.rs` - get_game_resolution(), get_game_dpi(), normalize_coord() + 9 tests
- `src-tauri/src/game_capture/screenshot.rs` - capture_window() with PrintWindow PW_RENDERFULLCONTENT
- `src-tauri/src/game_capture/mod.rs` - Updated to declare all submodules and re-export CaptureError

## Decisions Made
- Extracted `check_fullscreen_style()` as a pure function taking raw u32 style and i32 rect arrays, enabling unit testing without real Win32 HWNDs
- Used `#[cfg(windows)]` guards on all Win32-dependent code so the crate compiles and pure-logic tests run on Linux/WSL development environments

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Game capture module complete: safety module (Plan 03/04) can call capture_window() for pixel sampling
- Auto applier (Phase 4) can use find_diablo_window() + get_game_resolution() + normalize_coord() for click targeting
- All pure functions testable cross-platform; Win32-specific functions compile on Windows targets only

---
*Phase: 02-scaffold-safety-game-capture*
*Completed: 2026-03-16*
