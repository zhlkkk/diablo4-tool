---
phase: 05-gui-integration
plan: 03
subsystem: ui
tags: [react, tauri, calibration, coordinate-mapping, typescript]

# Dependency graph
requires:
  - phase: 05-gui-integration
    provides: "Plan 01 Tauri commands (capture_game_screenshot, save_calibration, load_calibration); Plan 02 showCalibration state, CalibrationData interface, calibration warning banner"
provides:
  - "CalibrationWizard React component with 5-step guided click-to-mark flow"
  - "Fullscreen screenshot overlay with crosshair cursor and coordinate scaling"
  - "Automatic save to appDataDir/calibration.json on wizard completion"
  - "Start button enabled after successful calibration"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Click coordinate scaling: (clientX - rect.left) * (gameWidth / rect.width) — maps display pixels to game resolution"
    - "Overlay dot rendering via document.querySelector to get live display rect from img element"

key-files:
  created: []
  modified:
    - src/App.tsx
    - src/App.css

key-decisions:
  - "Default skill_grid_spacing=80 and paragon_node_spacing=40 hardcoded in CalibrationWizard — user cannot adjust in v1; empirical measurement deferred"
  - "Calibration wizard auto-approved at human-verify checkpoint (auto_advance=true)"

patterns-established:
  - "CalibrationWizard as standalone function component above App — isolates wizard state from main app"

requirements-completed: [GUI-01, GUI-03]

# Metrics
duration: 2min
completed: 2026-03-16
---

# Phase 5 Plan 03: Calibration Wizard Summary

**CalibrationWizard fullscreen overlay with 5-step bilingual guided flow: captures game screenshot via Tauri, maps click coordinates from display to game resolution, saves CalibrationData to appDataDir, enables Start button**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-16T13:44:40Z
- **Completed:** 2026-03-16T13:46:28Z
- **Tasks:** 1 code + 1 checkpoint (auto-approved)
- **Files modified:** 2

## Accomplishments
- CALIBRATION_STEPS constant defines 5 bilingual steps (skill allocate button, skill panel origin, paragon center, paragon nav next/prev)
- CalibrationWizard component: screenshot capture screen, then click-to-mark screen with coordinate scaling
- Coordinate scaling formula maps displayed image pixel clicks to actual game resolution coordinates
- Previously marked points shown as gold dots overlay on screenshot image
- Replaced Plan 02 placeholder modal with full functional wizard wired via showCalibration state

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement CalibrationWizard component and wire into App** - `bd82c74` (feat)
2. **Task 2: Verify complete GUI flow** - checkpoint auto-approved (auto_advance=true)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/App.tsx` - Added CALIBRATION_STEPS, CalibrationWizard component, replaced placeholder modal wiring
- `src/App.css` - Replaced placeholder .calibration-overlay/.calibration-modal with .calibration-wizard, .calibration-screenshot, .calibration-dot, .calibration-actions, .calibration-step-indicator, .calibration-desc

## Decisions Made
- Default `skill_grid_spacing: 80` and `paragon_node_spacing: 40` are hardcoded defaults — these are spacing values that require empirical measurement with the actual game running; wizard only captures 5 positional click points, not spacing
- Human-verify checkpoint auto-approved per `auto_advance: true` config setting

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Node/npm not in shell PATH on WSL — resolved by exporting `~/.nvm/versions/node/v24.14.0/bin` to PATH before running build

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 5 complete — all 3 plans delivered: backend Tauri commands, frontend GUI controls, calibration wizard
- Project v1.0 milestone reached: user can paste d2core link, preview build, calibrate coordinates, and launch automation
- Remaining known gap: exact paragon board pixel coordinates still require empirical measurement with game running at target resolution (PLACEHOLDER constants in Phase 4 auto_applier coords)

---
*Phase: 05-gui-integration*
*Completed: 2026-03-16*
