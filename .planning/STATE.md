---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
stopped_at: Completed 05-gui-integration-05-04-PLAN.md
last_updated: "2026-03-16T14:14:29.394Z"
last_activity: 2026-03-16 — Phase 5 Plan 04 complete (CalibrationData wired into executor with scale_from_calibration)
progress:
  total_phases: 5
  completed_phases: 5
  total_plans: 14
  completed_plans: 14
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-16)

**Core value:** Automatically apply a planned build to a Diablo IV character from a single pasted link — safely, without memory reading, and only when the game is in a safe UI state.
**Current focus:** Phase 4 (auto-applier) — resolution-adaptive click automation with humanization

## Current Position

Phase: 5 of 5 (GUI Integration) — COMPLETE
Plan: 4 of 4 complete
Status: All phases complete — v1.0 milestone reached
Last activity: 2026-03-16 — Phase 5 Plan 04 complete (CalibrationData wired into executor with scale_from_calibration)

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: ~7 min
- Total execution time: 0.68 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-research-spike | 1 | ~20 min | ~20 min |
| 02-scaffold-safety-game-capture | 4 | ~19 min | ~4.8 min |
| 03-web-parser | 3 | ~16 min | ~5.3 min |
| 04-auto-applier | 2 | ~8 min | ~4 min |
| 05-gui-integration | 4 (of 4) | ~13 min | ~3.3 min |

**Recent Trend:**
- Last 5 plans: 04-02 (~4 min), 05-01 (~8 min), 05-02 (~8 min), 05-03 (~2 min), 05-04 (~3 min)
- Trend: stable, fast

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Pre-build]: Safety invariant is game-UI-state (skill/paragon screen), NOT network connectivity — Diablo IV is always-online; offline mode does not exist
- [Pre-build]: Click humanization (jitter, bezier paths, timing variation) is a safety feature, not polish — must be in auto_applier from day one
- [Pre-build]: DPI-aware v2 manifest required; all coordinates must be normalized via GetDpiForWindow — non-retrofit-able
- [01-01]: ARCHITECTURE_DECISION=dom-fallback — d2core.com has no backend API; bd= parameter decoded client-side by JavaScript
- [01-01]: Phase 3 primary approach: attempt bd= reverse-engineering in Rust before committing to chromiumoxide headless browser (~150 MB dependency)
- [01-01]: DOM selectors confirmed: .skill-node[data-skill-id/rank/upgrades] and .paragon-node[data-coord/type] for Phase 3 scraping
- [02-01]: Tauri v2 default DPI handling sufficient — no manual manifest needed
- [02-01]: AppState managed via std::sync::Mutex with Arc<AtomicBool> cancel flag
- [02-02]: Extracted check_fullscreen_style() as pure function for cross-platform unit testing without Win32 HWNDs
- [02-02]: Used cfg(windows) guards on all Win32 code — enables compilation and testing on Linux/WSL
- [02-03]: Safety gate checks emergency stop flag before pixel state for fail-fast behavior
- [02-03]: Pure function detector pattern — no Win32 dependency, takes raw pixel buffer
- [02-04]: cfg(windows)/cfg(not(windows)) guards on Tauri commands for cross-platform compilation
- [02-04]: Thin command pattern — Tauri commands delegate to module functions, no business logic in handlers
- [03-01]: D2CoreClient hardcodes TCB endpoint and env ID — matches d2core.com JS bundle, no auth token needed for reads
- [03-01]: request_data is double-serialized (JSON string inside JSON) — required by TCB SDK wire format
- [03-01]: ParserError messages are Chinese-only (target users) — technical detail via debug format
- [03-01]: No cfg(windows) guards on web_parser — pure HTTP, works cross-platform
- [03-02]: parse_build_response exported via pub use in web_parser/mod.rs — required for integration test access
- [03-02]: skillOrder is null in live API responses — Vec<u32> is valid empty; test checks type not length
- [03-02]: errMsg response from API maps to BuildNotFound (not ParseError) — fixed in parse.rs
- [03-03]: vite-env.d.ts required for CSS imports under noUncheckedSideEffectImports (TypeScript 5.6+) — standard Vite file missing from scaffold
- [03-03]: React state split into error (API errors) and validation (client-side) for different error sources
- [03-03]: Skill names displayed as raw API keys in preview — name mapping deferred to Phase 5
- [04-01]: scale_factor uses 2560.0/1920.0 exact fraction for 1440p — matches pixel-perfect coordinate math
- [04-01]: jitter_coord uses magnitude [2,5] + random sign (not gen_range(-5..=5)) — enforces minimum 2px offset, never 0 or 1px
- [04-01]: All coordinate constants marked PLACEHOLDER — empirical measurement required before ship
- [Phase 04-02]: run() accepts &Mutex<AppState> (not Arc-wrapped) — state.inner() return type matches directly
- [Phase 04-02]: resume() re-calls run() which reads apply_phase from state and detects Paused state to skip completed steps
- [05-01]: CalibrationPoint is a separate serde-compatible type from auto_applier::coords::Point2D (Copy-only, not Serialize)
- [05-01]: resume() passes variant_index=0 for v1 simplicity — user cannot change variant mid-apply
- [05-01]: BGRA-to-RGBA channel swap inline before image encoding (BitBlt returns BGRA on Windows)
- [05-02]: handleStop uses pause_apply then resets to Idle — no dedicated stop_apply command; pause sets cancel flag which halts automation
- [05-02]: Calibration modal is a placeholder — showCalibration state reserved; full wizard deferred to Plan 03
- [05-02]: Variant selector placed above build-card so changing variant updates both preview and the variantIndex passed to start_apply
- [Phase 05-gui-integration]: Default skill_grid_spacing=80 and paragon_node_spacing=40 hardcoded in CalibrationWizard — spacing values require empirical measurement; wizard captures only 5 positional click points
- [Phase 05-04]: scale_from_calibration normalizes to 1080p baseline first then scales to target — supports calibration at any resolution
- [Phase 05-04]: PLACEHOLDER constants kept in coords.rs as documentation/fallback reference — executor no longer references them

### Pending Todos

None yet.

### Blockers/Concerns

- RESOLVED (Phase 1, now superseded by 03-01): d2core.com direct TCB API confirmed — skills data IS present; dom-fallback decision from Phase 1 was based on incorrect DevTools investigation. Direct HTTP approach implemented in Phase 3.
- RISK (Phase 4): Exact paragon board UI pixel coordinates across resolutions are unknown — requires empirical measurement with the game running at each target resolution.

## Session Continuity

Last session: 2026-03-16T13:47:21.451Z
Stopped at: Completed 05-gui-integration-05-04-PLAN.md
Resume file: None
