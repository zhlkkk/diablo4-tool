---
phase: 02-scaffold-safety-game-capture
plan: 04
subsystem: integration
tags: [tauri, global-shortcut, hotkey, f10, emergency-stop, commands]

# Dependency graph
requires:
  - phase: 02-scaffold-safety-game-capture
    provides: "game_capture module (window, dpi, screenshot) and safety module (detector, error, assert_safe_state)"
provides:
  - "F10 emergency stop hotkey via tauri-plugin-global-shortcut"
  - "Tauri commands: get_game_state, check_safety, reset_emergency_stop"
  - "Fully wired Tauri application with all Phase 2 modules integrated"
affects: [03-web-parser, 04-auto-applier, 05-gui]

# Tech tracking
tech-stack:
  added: []
  patterns: [thin-command-fat-module, cfg-windows-guards-on-commands]

key-files:
  created:
    - src-tauri/src/safety/hotkey.rs
  modified:
    - src-tauri/src/safety/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "cfg(windows)/cfg(not(windows)) guards on Tauri commands for cross-platform compilation"
  - "Shortcut equality check instead of field access for F10 matching in handler"

patterns-established:
  - "Thin command pattern: Tauri commands delegate to module functions, no business logic in handlers"
  - "cfg(windows) command stubs: non-Windows builds return descriptive error strings"

requirements-completed: [SAFE-04, CAPT-03]

# Metrics
duration: 8min
completed: 2026-03-16
---

# Phase 2 Plan 04: Tauri Wiring Summary

**F10 emergency stop hotkey and three Tauri commands (get_game_state, check_safety, reset_emergency_stop) wiring game_capture and safety modules into the Tauri app**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-16T11:15:37Z
- **Completed:** 2026-03-16T11:23:32Z
- **Tasks:** 1
- **Files modified:** 3

## Accomplishments
- F10 global hotkey registered via tauri-plugin-global-shortcut, sets AtomicBool cancel flag and emits emergency_stop event
- Three Tauri commands expose game_capture and safety module functions to frontend
- Full cargo build succeeds with all modules wired together
- All 29 unit tests pass (safety, game_capture, hotkey modules)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create emergency stop hotkey module and wire Tauri commands** - `fb76b47` (feat)

**Plan metadata:** pending (docs: complete plan)

## Files Created/Modified
- `src-tauri/src/safety/hotkey.rs` - F10 emergency stop setup using tauri-plugin-global-shortcut
- `src-tauri/src/safety/mod.rs` - Added `pub mod hotkey` declaration
- `src-tauri/src/lib.rs` - Tauri builder with commands, state management, and hotkey setup in .setup()

## Decisions Made
- Used `cfg(windows)`/`cfg(not(windows))` guards on get_game_state and check_safety commands so the project compiles and tests on Linux/WSL (where Win32 APIs are unavailable), returning descriptive error strings on non-Windows
- Matched F10 shortcut via equality check (`shortcut == &Shortcut::new(None, Code::F10)`) rather than field access, as the exact field API varies between plugin versions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added cfg(not(windows)) command stubs**
- **Found during:** Task 1 (lib.rs rewrite)
- **Issue:** Plan did not account for non-Windows compilation -- get_game_state and check_safety call Win32 functions that don't exist on Linux
- **Fix:** Added `#[cfg(windows)]` on Win32 command implementations and `#[cfg(not(windows))]` stub versions returning error strings
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** cargo build and cargo test both succeed on Linux/WSL
- **Committed in:** fb76b47 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Essential for cross-platform compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 2 modules (scaffold, game_capture, safety) are wired into the Tauri application
- F10 emergency stop, game window detection, safety checking, and DPI normalization are all accessible via Tauri commands
- Ready for Phase 3 (web_parser) to add build link parsing
- Ready for Phase 4 (auto_applier) to use game_capture and safety modules for click automation

---
*Phase: 02-scaffold-safety-game-capture*
*Completed: 2026-03-16*
