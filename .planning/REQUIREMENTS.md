# Requirements: Diablo4 Build Applier

**Defined:** 2026-03-16
**Core Value:** Automatically apply a planned build to a Diablo IV character from a single pasted link — safely, without memory reading, and only in safe UI states.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Research Spike

- [x] **SPIKE-01**: Developer can confirm whether d2core.com API includes skill allocation data by inspecting live network traffic — COMPLETE 2026-03-16 (REVISED: API EXISTS via Tencent CloudBase; skills ARE in response as `variants[].skill` + `variants[].equipSkills`)
- [x] **SPIKE-02**: Developer has documented the exact d2core API endpoint, request format, and response JSON schema for builds — COMPLETE 2026-03-16 (REVISED: TCB endpoint verified with live 200 OK responses; full schema documented in SPIKE-FINDINGS.md)

### Web Parser

- [ ] **PARSE-01**: User can paste a d2core.com/d4/planner?bd=XXXX link and the app extracts the build ID
- [ ] **PARSE-02**: App calls d2core.com API with the build ID and retrieves the full build JSON response
- [ ] **PARSE-03**: App parses paragon board data (board names, node coordinates, glyphs, rotation) from API response into a typed BuildPlan
- [ ] **PARSE-04**: App parses skill allocation data from API response into the BuildPlan (SPIKE-01 confirmed: skills available in API)
- [ ] **PARSE-05**: App displays a human-readable build preview (skills + paragon) in the GUI before any automation
- [ ] **PARSE-06**: Parser handles invalid/expired build IDs with clear error messages
- [ ] **PARSE-07**: Parser has pinned test vectors for known-good builds with unit tests

### Safety Module

- [ ] **SAFE-01**: Safety module detects whether Diablo IV is in a safe UI state for automation (character select, skill tree screen, paragon board screen)
- [ ] **SAFE-02**: Safety module refuses to start automation if game is not in a recognized safe state
- [ ] **SAFE-03**: Safety module re-checks game state before each click step (not just once at start)
- [ ] **SAFE-04**: Safety module provides immediate emergency stop (hotkey or button) that halts all automation
- [ ] **SAFE-05**: Safety module logs all automation decisions (start, stop, state checks) for user transparency
- [ ] **SAFE-06**: Unit tests verify safety module correctly identifies safe vs unsafe states

### Game Capture

- [ ] **CAPT-01**: App detects whether Diablo IV process is running and finds the game window handle
- [ ] **CAPT-02**: App detects the current game resolution from the window
- [ ] **CAPT-03**: App handles DPI scaling correctly (Per-Monitor DPI Aware v2 manifest, GetDpiForWindow normalization)
- [ ] **CAPT-04**: App detects if game is in exclusive fullscreen and warns user to switch to borderless windowed
- [ ] **CAPT-05**: App can capture a screenshot of the game window for state detection
- [ ] **CAPT-06**: Unit tests verify resolution detection and DPI normalization logic

### Auto Applier

- [ ] **APPLY-01**: App applies skill allocations to character via resolution-adaptive UI click automation
- [ ] **APPLY-02**: App applies paragon board choices via resolution-adaptive UI click automation
- [ ] **APPLY-03**: Click coordinates adapt correctly to detected game resolution
- [ ] **APPLY-04**: Click automation includes humanization (coordinate jitter, timing variation) to avoid detection
- [ ] **APPLY-05**: App shows real-time progress of apply operation (which skill/node is being applied)
- [ ] **APPLY-06**: App can pause and resume the apply process
- [ ] **APPLY-07**: Unit tests verify coordinate mapping calculations for multiple resolutions

### GUI

- [ ] **GUI-01**: User sees a clean interface to paste d2core build link
- [ ] **GUI-02**: User sees parsed build preview (skills + paragon) before applying
- [ ] **GUI-03**: User has start/stop/pause controls for the apply process
- [ ] **GUI-04**: User sees real-time status and progress during automation
- [ ] **GUI-05**: User sees clear error messages for all failure states (game not found, bad link, unsafe state)
- [ ] **GUI-06**: App window stays responsive during long-running automation operations

## v2 Requirements

### Enhanced Parsing

- **PARSE-V2-01**: Support for additional build planner sites (maxroll.gg, d4builds.gg)
- **PARSE-V2-02**: Build history — remember previously applied builds

### Enhanced Automation

- **APPLY-V2-01**: Dry-run mode that shows what clicks would be made without executing
- **APPLY-V2-02**: Skill refund automation (reset skills before applying new build)
- **APPLY-V2-03**: Paragon board reset automation before applying

### Quality of Life

- **QOL-V2-01**: Auto-detect build link from clipboard
- **QOL-V2-02**: Build comparison (current vs target)
- **QOL-V2-03**: Seasonal build template library

## Out of Scope

| Feature | Reason |
|---------|--------|
| Memory reading or injection | Violates game ToS, triggers Warden anti-cheat |
| Online/multiplayer automation | Unacceptable ban risk; safety module prevents this |
| Build creation or editing | Tool only applies builds from d2core links |
| Mobile or cross-platform | Windows only (Diablo IV PC client) |
| Other build planner sites | d2core.com only for v1 |
| Gear/item automation | Far more complex UI navigation; defer to v2+ |
| OCR-based state detection | Over-engineered for v1; pixel sampling sufficient |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SPIKE-01 | Phase 1 — Research Spike | Complete (2026-03-16) |
| SPIKE-02 | Phase 1 — Research Spike | Complete (2026-03-16) |
| SAFE-01 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| SAFE-02 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| SAFE-03 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| SAFE-04 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| SAFE-05 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| SAFE-06 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-01 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-02 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-03 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-04 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-05 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| CAPT-06 | Phase 2 — Scaffold + Safety + Game Capture | Pending |
| PARSE-01 | Phase 3 — Web Parser | Pending |
| PARSE-02 | Phase 3 — Web Parser | Pending |
| PARSE-03 | Phase 3 — Web Parser | Pending |
| PARSE-04 | Phase 3 — Web Parser | Pending |
| PARSE-05 | Phase 3 — Web Parser | Pending |
| PARSE-06 | Phase 3 — Web Parser | Pending |
| PARSE-07 | Phase 3 — Web Parser | Pending |
| APPLY-01 | Phase 4 — Auto Applier | Pending |
| APPLY-02 | Phase 4 — Auto Applier | Pending |
| APPLY-03 | Phase 4 — Auto Applier | Pending |
| APPLY-04 | Phase 4 — Auto Applier | Pending |
| APPLY-05 | Phase 4 — Auto Applier | Pending |
| APPLY-06 | Phase 4 — Auto Applier | Pending |
| APPLY-07 | Phase 4 — Auto Applier | Pending |
| GUI-01 | Phase 5 — GUI + Integration | Pending |
| GUI-02 | Phase 5 — GUI + Integration | Pending |
| GUI-03 | Phase 5 — GUI + Integration | Pending |
| GUI-04 | Phase 5 — GUI + Integration | Pending |
| GUI-05 | Phase 5 — GUI + Integration | Pending |
| GUI-06 | Phase 5 — GUI + Integration | Pending |

**Coverage:**
- v1 requirements: 34 total
- Mapped to phases: 34
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-16*
*Last updated: 2026-03-16 after Phase 1 Plan 01 completion (SPIKE-01, SPIKE-02 complete)*
