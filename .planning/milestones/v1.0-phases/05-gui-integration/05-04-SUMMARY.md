---
phase: 05-gui-integration
plan: 04
subsystem: auto-applier
tags: [calibration, coordinate-scaling, rust, tauri]

# Dependency graph
requires:
  - phase: 04-auto-applier
    provides: "executor.rs with build_step_sequence and run() automation loop"
  - phase: 05-gui-integration (plan 01)
    provides: "CalibrationData type and save/load calibration Tauri commands"
  - phase: 05-gui-integration (plan 03)
    provides: "CalibrationWizard UI that saves calibration.json"
provides:
  - "executor.rs loads CalibrationData at runtime from calibration.json"
  - "build_step_sequence uses CalibrationData fields instead of PLACEHOLDER constants"
  - "scale_from_calibration() function for cross-resolution coordinate mapping"
  - "NoCalibration error variant for missing calibration file"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: ["runtime calibration loading in executor", "cross-resolution coordinate normalization via 1080p baseline"]

key-files:
  created: []
  modified:
    - "src-tauri/src/auto_applier/executor.rs"
    - "src-tauri/src/auto_applier/coords.rs"
    - "src-tauri/src/auto_applier/error.rs"

key-decisions:
  - "scale_from_calibration normalizes to 1080p first, then scales to target resolution -- supports calibration at any resolution"
  - "PLACEHOLDER constants kept in coords.rs as documentation/fallback reference, not deleted"
  - "CalibrationPoint not imported in executor (only CalibrationData needed since fields accessed via cal.field.x)"

patterns-established:
  - "Runtime calibration: executor loads calibration.json on each run() call, not cached in AppState"
  - "Coordinate pipeline: calibration coords -> normalize to 1080p -> scale to target resolution"

requirements-completed: [GUI-01, GUI-02, GUI-03, GUI-04, GUI-05, GUI-06]

# Metrics
duration: 3min
completed: 2026-03-16
---

# Phase 5 Plan 4: Calibration Data Wiring Summary

**Executor loads CalibrationData from disk at runtime and uses calibrated coordinates for all clicks, replacing PLACEHOLDER constants with scale_from_calibration() cross-resolution mapping**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-16T14:06:46Z
- **Completed:** 2026-03-16T14:09:35Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Wired CalibrationData into executor: load_calibration_from_disk() reads calibration.json at runtime
- Replaced all PLACEHOLDER constant references (SkillTreeCoords, ParagonBoardCoords) with CalibrationData fields in build_step_sequence
- Added scale_from_calibration() to coords.rs for normalizing coordinates across resolutions
- All 64 tests pass including 3 new scale_from_calibration tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire CalibrationData into executor and coords** - `6563f29` (feat)
2. **Task 2: Update tests for new build_step_sequence signature** - `81a3967` (test)

## Files Created/Modified
- `src-tauri/src/auto_applier/executor.rs` - Added load_calibration_from_disk(), updated build_step_sequence to accept CalibrationData, replaced PLACEHOLDER constants, switched to scale_from_calibration in run() loop
- `src-tauri/src/auto_applier/coords.rs` - Added scale_from_calibration() function, added 3 cross-resolution scaling tests
- `src-tauri/src/auto_applier/error.rs` - Added NoCalibration variant to ApplyError

## Decisions Made
- scale_from_calibration normalizes to 1080p first then scales to target -- enables calibration at any resolution
- PLACEHOLDER constants kept as documentation/fallback reference, not deleted from coords.rs
- CalibrationPoint import removed from executor (unused -- accessed through CalibrationData fields)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused CalibrationPoint import**
- **Found during:** Task 1
- **Issue:** Plan specified importing CalibrationPoint in executor.rs but it was unused (fields accessed via CalibrationData)
- **Fix:** Removed CalibrationPoint from import line to eliminate compiler warning
- **Files modified:** src-tauri/src/auto_applier/executor.rs
- **Verification:** cargo check passes without unused import warning
- **Committed in:** 6563f29

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor cleanup, no scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full calibration pipeline now complete: CalibrationWizard saves coordinates, executor loads and uses them
- All v1.0 milestone functionality is wired end-to-end
- Ready for empirical coordinate measurement with actual game running

---
*Phase: 05-gui-integration*
*Completed: 2026-03-16*
