# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — MVP

**Shipped:** 2026-03-16
**Phases:** 6 | **Plans:** 15 | **Sessions:** 1

### What Was Built
- Complete d2core.com build parser via Tencent CloudBase API reverse engineering
- Win32 game capture with DPI-aware resolution detection and screenshot
- Pixel-sampling safety module with F10 emergency stop hotkey
- Resolution-adaptive click automation with humanized timing
- Full React/Tauri desktop GUI with bilingual UI, calibration wizard, build preview
- GitHub Actions CI (clippy, cargo test, tsc) and Release workflow (tauri-action NSIS installer)

### What Worked
- Research-first approach (Phase 1 spike) de-risked the entire project by confirming API feasibility upfront
- 4-module architecture (web_parser, game_capture, safety, auto_applier) kept concerns cleanly separated
- cfg(windows) guards enabled development and testing on WSL/Linux despite Win32 dependencies
- Offline test fixtures with pinned API responses made tests fast and reliable
- Phased execution with wave-based parallelization kept context windows lean

### What Was Inefficient
- Paragon board pixel coordinates remain unmeasured — empirical calibration still needed with live game
- Calibration wizard wiring (Phase 5, Plan 04) required understanding dependencies across 3 prior plans
- Some SUMMARY.md files lack one_liner fields, making automated accomplishment extraction unreliable

### Patterns Established
- Double-parse pattern for TCB API (JSON string inside JSON body)
- CalibrationData with 5-point capture → resolution factor scaling to 1080p baseline
- Safety gate pattern: assert_safe_state() before each click step
- Thin Tauri command wrappers delegating to module logic with Result<T, String>

### Key Lessons
1. Reverse-engineering third-party APIs should always start with a research spike — the TCB discovery changed the entire technical approach
2. Platform-specific code (Win32) should be behind cfg gates from day one for CI compatibility
3. Safety-critical automation needs defense in depth: pre-check + per-step re-check + emergency stop

### Cost Observations
- Model mix: ~70% opus, ~25% sonnet, ~5% haiku (executors inherit, verifiers use sonnet)
- Sessions: 1 (entire v1.0 built in a single session)
- Notable: 93 commits in ~8 hours, all 15 plans executed successfully on first attempt

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | 1 | 6 | Initial project — established GSD workflow patterns |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | ~20 | Core modules | 0 |

### Top Lessons (Verified Across Milestones)

1. Research spikes before implementation save more time than they cost
2. Platform abstraction via cfg gates enables cross-platform CI from the start
