---
phase: 05-gui-integration
plan: 01
subsystem: backend-rust
tags: [tauri, calibration, screenshot, variant-index, types]
dependency_graph:
  requires: []
  provides: [calibration-io-commands, screenshot-command, variant-index-support]
  affects: [05-02-frontend-calibration, 05-03-frontend-apply]
tech_stack:
  added: [base64 = "0.22"]
  patterns: [thin-command, cfg-windows-guard, appDataDir-io]
key_files:
  created: []
  modified:
    - src-tauri/src/types.rs
    - src-tauri/src/auto_applier/executor.rs
    - src-tauri/src/lib.rs
    - src-tauri/Cargo.toml
decisions:
  - CalibrationPoint is a separate serde-compatible type from auto_applier::coords::Point2D (which is Copy-only, not Serialize)
  - resume() passes variant_index=0 for v1 simplicity — user cannot change variant mid-apply
  - BGRA-to-RGBA channel swap performed inline before image encoding (BitBlt returns BGRA on Windows)
metrics:
  duration: ~8 min
  completed: 2026-03-16
  tasks_completed: 2
  files_modified: 4
---

# Phase 5 Plan 01: Backend Rust Commands for GUI Integration Summary

**One-liner:** CalibrationData type + calibration I/O + game screenshot capture + variant_index support wired into Tauri commands and executor.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add CalibrationData type and variant_index to executor | f03215d | types.rs, executor.rs, lib.rs, Cargo.toml |
| 2 | Add calibration I/O, screenshot capture, variant_index to start_apply | e1ffa0e | lib.rs |

## What Was Built

### CalibrationData Type (types.rs)
Added `CalibrationPoint` and `CalibrationData` structs with full `Serialize`/`Deserialize` derives. `CalibrationPoint` is intentionally separate from `auto_applier::coords::Point2D` (which is `Copy`-only and lacks serde support).

`CalibrationData` captures all UI reference points: skill allocate button, skill panel origin, skill grid spacing, paragon center, paragon node spacing, and paragon navigation buttons. Resolution metadata is stored to support runtime scaling via the existing `scale_factor()` function.

### variant_index in executor::run()
Changed `executor::run()` signature from using `plan.variants.first()` to `plan.variants.get(variant_index)` with a new `variant_index: usize` parameter. `resume()` passes `0` for v1 (acceptable because users cannot change variant mid-apply). `start_apply` command in lib.rs was updated to accept and forward `variant_index`.

### New Tauri Commands (lib.rs)
Three new commands registered in `generate_handler![]`:
- `load_calibration` — reads `calibration.json` from appDataDir, returns `Option<CalibrationData>` (None if not yet created)
- `save_calibration` — writes pretty-printed JSON to appDataDir, creates directory if missing
- `capture_game_screenshot` — Windows: BitBlt capture, BGRA-to-RGBA swap, encode as base64 PNG string; non-Windows: returns error stub

### base64 Dependency
Added `base64 = "0.22"` to Cargo.toml for PNG-to-base64 encoding in the screenshot command.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed start_apply call site when updating executor::run() signature**
- **Found during:** Task 1 (cargo test compilation)
- **Issue:** `start_apply` in lib.rs called `executor::run(plan, app, state.inner())` with 3 args; changing run() to 4 args broke compilation immediately
- **Fix:** Updated `start_apply` signature to accept `variant_index: usize` and forwarded it in the call — this was the intended Task 2 change, just applied one task early to keep tests passing
- **Files modified:** src-tauri/src/lib.rs
- **Commit:** f03215d

## Self-Check: PASSED

Files verified:
- FOUND: src-tauri/src/types.rs (contains CalibrationData, CalibrationPoint)
- FOUND: src-tauri/src/auto_applier/executor.rs (variant_index parameter, .get(variant_index))
- FOUND: src-tauri/src/lib.rs (load_calibration, save_calibration, capture_game_screenshot, variant_index)
- FOUND: src-tauri/Cargo.toml (base64 = "0.22")

Commits verified:
- f03215d: feat(05-01): add CalibrationData type and variant_index to executor
- e1ffa0e: feat(05-01): add calibration I/O and screenshot capture Tauri commands

Tests: 61 passed, 0 failed (cargo test --lib)
Build: cargo build succeeds (warnings are pre-existing, none from new code)
