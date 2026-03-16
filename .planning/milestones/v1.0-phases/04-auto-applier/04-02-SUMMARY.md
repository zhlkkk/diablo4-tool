---
phase: 04-auto-applier
plan: 02
subsystem: automation
tags: [rust, tauri, enigo, tokio, click-automation, safety-gating, humanization]

# Dependency graph
requires:
  - phase: 04-01
    provides: "coords.rs, humanize.rs, error.rs — coordinate scaling, jitter, error types"
  - phase: 02-scaffold-safety-game-capture
    provides: "safety::assert_safe_state, game_capture::screenshot::capture_window — safety gate and screenshot"
provides:
  - "executor.rs: ClickStep struct, build_step_sequence pure function, click_at, bring_window_foreground, run, pause, resume"
  - "Tauri commands: start_apply, pause_apply, resume_apply registered in invoke_handler"
affects: [05-ui, integration-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Executor loop: cancel_flag checked before each click, screenshot captured, safety::assert_safe_state called, then scale_coord + jitter_coord + click_at"
    - "Pause/resume: pause() sets cancel_flag + saves step in Paused{step,total}; resume() clears flag, re-calls run() which detects Paused state"
    - "Thin Tauri command pattern: state.inner() returns &Mutex<AppState> matched by executor function signatures"

key-files:
  created:
    - "src-tauri/src/auto_applier/executor.rs"
  modified:
    - "src-tauri/src/auto_applier/mod.rs"
    - "src-tauri/src/lib.rs"

key-decisions:
  - "run() accepts &Mutex<AppState> (not Arc-wrapped) — state.inner() return type matches directly, no wrapper needed"
  - "resume() re-calls run() which reads apply_phase from state and detects Paused{step} to skip already-done steps"
  - "bring_window_foreground wrapped in cfg(windows); non-windows stub returns Ok(()) for cross-platform compilation"
  - "tauri::Emitter trait must be explicitly imported for app.emit() to resolve — Rule 1 auto-fix during Task 2"

patterns-established:
  - "Lock-extract-drop: acquire Mutex lock, extract needed values (clone Arc, read fields), drop lock immediately before async work"
  - "Cancel flag semantics: pause() sets flag + transitions Running->Paused; run() loop treats Paused phase as clean exit (Ok), not error"

requirements-completed: [APPLY-01, APPLY-02, APPLY-05, APPLY-06]

# Metrics
duration: 4min
completed: 2026-03-16
---

# Phase 4 Plan 02: Auto-Applier Executor Summary

**Safety-gated executor loop applying BuildPlan via humanized, resolution-scaled clicks with pause/resume; wired as start_apply/pause_apply/resume_apply Tauri commands**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-16T13:03:47Z
- **Completed:** 2026-03-16T13:07:47Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- `build_step_sequence` converts BuildPlan Variant into ordered ClickSteps: skills (respecting skill_order) -> equip_skills -> paragon boards
- `run()` loop gates every click on `safety::assert_safe_state`, scales coords via `scale_coord`, applies jitter via `jitter_coord`, emits progress events, supports pause/resume from saved step index
- Three Tauri commands (`start_apply`, `pause_apply`, `resume_apply`) registered and compiling; all delegate to executor via `state.inner()` — no Arc wrapping needed

## Task Commits

1. **Task 1: ClickStep struct and build_step_sequence with 4 unit tests** - `df57058` (feat)
2. **Task 2: click_at, bring_window_foreground, run, pause, resume** - `fed4704` (feat)
3. **Task 3: Wire start_apply, pause_apply, resume_apply Tauri commands** - `6b12429` (feat)

## Files Created/Modified
- `src-tauri/src/auto_applier/executor.rs` - Full executor: ClickStep, build_step_sequence, click_at, bring_window_foreground, run, pause, resume; 4 unit tests
- `src-tauri/src/auto_applier/mod.rs` - Added `pub mod executor;`
- `src-tauri/src/lib.rs` - Added start_apply, pause_apply, resume_apply commands and invoke_handler registration

## Decisions Made
- `run()` signature uses `&Mutex<AppState>` (not `Arc<Mutex<AppState>>`) — Tauri's `state.inner()` returns `&Mutex<T>` directly, eliminating any need for Arc wrapping at the command boundary
- `resume()` re-uses `run()` rather than duplicating loop logic — run() reads `apply_phase` from state and detects `Paused{step}` to skip already-completed steps
- `bring_window_foreground` non-windows stub takes `usize` (not HWND) to keep the stub compilable without the windows crate dependency

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added missing `use tauri::Emitter;` import**
- **Found during:** Task 2 (run function implementation)
- **Issue:** `app.emit()` call failed to resolve — `Emitter` trait must be explicitly in scope in Tauri v2
- **Fix:** Added `use tauri::Emitter;` to executor.rs imports
- **Files modified:** `src-tauri/src/auto_applier/executor.rs`
- **Verification:** `cargo check` clean after fix
- **Committed in:** `fed4704` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - missing trait import)
**Impact on plan:** Required for compilation. No scope changes.

## Issues Encountered
None beyond the Tauri Emitter trait import issue documented above.

## Next Phase Readiness
- Executor is complete and wired — Phase 5 (UI) can invoke start_apply/pause_apply/resume_apply from the frontend
- Coordinate constants in coords.rs are still PLACEHOLDER values — empirical measurement with game running required before shipping
- Non-windows stub for click_at returns Err — automation only functional on Windows at runtime (by design)

---
*Phase: 04-auto-applier*
*Completed: 2026-03-16*
