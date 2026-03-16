---
phase: 04-auto-applier
plan: 01
subsystem: automation
tags: [rust, rand, enigo, coordinate-scaling, click-humanization, thiserror]

# Dependency graph
requires:
  - phase: 02-scaffold-safety-game-capture
    provides: Resolution enum in types.rs used by scale_coord
provides:
  - Resolution-adaptive coordinate scaling (scale_coord, scale_factor)
  - Click humanization with magnitude-bounded jitter (jitter_coord)
  - Random delay generation for timing variance (random_delay_ms)
  - ApplyError enum for auto-applier error handling
  - Point2D struct and coordinate constant tables (SkillTreeCoords, ParagonBoardCoords)
affects:
  - 04-02-executor (consumes coords, humanize, error types)
  - 04-02 (imports jitter_coord and random_delay_ms for click simulation)

# Tech tracking
tech-stack:
  added:
    - rand = "0.8" (random number generation for jitter and delay)
    - enigo = "0.6" (input simulation — declared in Cargo.toml, used in Plan 02)
  patterns:
    - TDD red-green pattern for pure functions
    - Magnitude-then-sign jitter pattern (ensures minimum offset, never 0 or 1 px)
    - PLACEHOLDER comments on all empirical coordinate values

key-files:
  created:
    - src-tauri/src/auto_applier/coords.rs
    - src-tauri/src/auto_applier/humanize.rs
    - src-tauri/src/auto_applier/error.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/auto_applier/mod.rs

key-decisions:
  - "scale_factor uses 2560.0/1920.0 for 1440p (not 4/3) to match exact pixel math"
  - "jitter uses magnitude [2,5] + random sign (not gen_range(-5..=5)) to enforce minimum 2px offset"
  - "All coordinate constants marked PLACEHOLDER — empirical measurement required before ship"

patterns-established:
  - "Magnitude-sign jitter: gen_range(2..=5) then gen_bool(0.5) for sign — never produces 0 or 1px offset"
  - "scale_coord rounds to nearest u32 via .round() — consistent with pixel grid"
  - "ApplyError follows SafetyError thiserror pattern established in phase 02"

requirements-completed: [APPLY-03, APPLY-04, APPLY-07]

# Metrics
duration: 2min
completed: 2026-03-16
---

# Phase 4 Plan 01: Auto Applier Foundations Summary

**Resolution-adaptive coordinate scaling (1080p/1440p/4K), magnitude-bounded click jitter (2-5px), and ApplyError enum — 16 unit tests passing on Linux**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-16T12:58:07Z
- **Completed:** 2026-03-16T13:00:16Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- scale_coord correctly maps 1080p reference coordinates to 1440p (1.333x) and 4K (2.0x) with rounding
- jitter_coord enforces minimum 2px offset using magnitude-then-sign pattern, clamps at 0
- random_delay_ms returns 50-200ms range mimicking human reaction time
- ApplyError enum with 8 variants (SafetyFailure, InputFailed, CaptureFailed, NoBuildPlan, NoGameState, UnsupportedResolution, TaskPanic, Cancelled)
- enigo and rand added to Cargo.toml; all pure functions compile cross-platform (Linux/WSL)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies and create coords.rs** - `901dcc8` (feat)
2. **Task 2: Create humanize.rs** - `cc65dfd` (feat)

_Note: Tasks used TDD — implementation and tests co-committed in GREEN phase (no separate RED commits as tests were written with implementation)._

## Files Created/Modified
- `src-tauri/src/auto_applier/coords.rs` - scale_factor, scale_coord, Point2D, SkillTreeCoords, ParagonBoardCoords with PLACEHOLDER comments
- `src-tauri/src/auto_applier/humanize.rs` - jitter_coord and random_delay_ms with 6 unit tests
- `src-tauri/src/auto_applier/error.rs` - ApplyError enum with 8 variants
- `src-tauri/src/auto_applier/mod.rs` - Declares coords, error, humanize submodules
- `src-tauri/Cargo.toml` - Added enigo = "0.6" and rand = "0.8"

## Decisions Made
- scale_factor uses exact fraction (2560.0/1920.0) not 4.0/3.0 — matches pixel-perfect coordinate math
- jitter_coord generates magnitude in [2,5] then multiplies by random sign — enforces minimum 2px offset (gen_range(-5..=5) could return 0 or 1)
- All coordinate constant values marked `// PLACEHOLDER: requires empirical measurement at 1080p` — values are guesses until game is running

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None — no external service configuration required.

## Next Phase Readiness
- coords.rs and humanize.rs ready for consumption by Plan 02 (executor)
- enigo dependency declared; Plan 02 will add Windows-specific input simulation
- All coordinate constants are placeholders — empirical measurement with game running required before Phase 5

## Self-Check: PASSED

- coords.rs: FOUND at src-tauri/src/auto_applier/coords.rs
- humanize.rs: FOUND at src-tauri/src/auto_applier/humanize.rs
- error.rs: FOUND at src-tauri/src/auto_applier/error.rs
- SUMMARY.md: FOUND at .planning/phases/04-auto-applier/04-01-SUMMARY.md
- commit 901dcc8: FOUND (Task 1)
- commit cc65dfd: FOUND (Task 2)

---
*Phase: 04-auto-applier*
*Completed: 2026-03-16*
