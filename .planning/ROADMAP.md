# Roadmap: Diablo4 Build Applier

## Overview

Five phases deliver the complete apply-a-build-from-a-link flow. Phase 1 resolves the single hard empirical blocker (d2core skills data availability) before any production code is written. Phase 2 establishes the project scaffold with game capture and the corrected safety invariant (game-UI-state, not network). Phase 3 builds the web parser using findings from Phase 1. Phase 4 implements the auto applier using BuildPlan and GameState from the two preceding phases. Phase 5 wires the complete frontend GUI and delivers the end-to-end user experience.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Research Spike** - Empirically verify d2core.com API skills data availability before any parser code is written — COMPLETE 2026-03-16
- [x] **Phase 2: Scaffold + Safety + Game Capture** - Project skeleton, corrected safety invariant, and game window capture module — COMPLETE 2026-03-16
- [x] **Phase 3: Web Parser** - d2core link parsing and BuildPlan extraction, informed by Phase 1 findings (completed 2026-03-16)
- [ ] **Phase 4: Auto Applier** - Resolution-adaptive skill and paragon click automation with humanization and safety re-checks
- [ ] **Phase 5: GUI + Integration** - Frontend interface, build preview, controls, and end-to-end wiring

## Phase Details

### Phase 1: Research Spike
**Goal**: Developer has confirmed empirically whether d2core.com API includes skill allocation data and documented the exact endpoint, request format, and response JSON schema
**Depends on**: Nothing (first phase)
**Requirements**: SPIKE-01, SPIKE-02
**Success Criteria** (what must be TRUE):
  1. Developer has confirmed YES or NO for skills data presence in the d2core API response via live browser devtools inspection
  2. The exact API endpoint URL, required request headers, and response JSON schema are written down in a spike document
  3. The parser architecture decision (direct HTTP vs. DOM fallback) is recorded based on the findings
**Plans**: 1 plan

Plans:
- [x] 01-01-PLAN.md — Investigate d2core API (human DevTools capture + author SPIKE-FINDINGS.md) — COMPLETE 2026-03-16 | dom-fallback architecture confirmed

### Phase 2: Scaffold + Safety + Game Capture
**Goal**: Working Tauri project skeleton with DPI-aware manifest, typed AppState/BuildPlan/GameState structs, safety module that gates on game-UI-state (not network), and game window capture module
**Depends on**: Phase 1
**Requirements**: SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, CAPT-01, CAPT-02, CAPT-03, CAPT-04, CAPT-05, CAPT-06
**Success Criteria** (what must be TRUE):
  1. `cargo build` succeeds and `cargo test` runs the safety module unit tests with all passing
  2. Safety module correctly identifies in-game safe states (skill tree screen, paragon board screen) and blocks automation in any other state
  3. App detects the Diablo IV window handle, reads its resolution, and normalizes coordinates for DPI scaling at 100%, 125%, and 150% display scaling
  4. Emergency stop (hotkey or button) halts all automation immediately when triggered
  5. App warns the user and refuses to continue if the game is running in exclusive fullscreen
**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Tauri v2 scaffold with React template, shared types, DPI manifest, module stubs — COMPLETE 2026-03-16
- [x] 02-02-PLAN.md — Game capture module: window finding, DPI/resolution, screenshot capture — COMPLETE 2026-03-16
- [x] 02-03-PLAN.md — Safety module: pixel sampling detector, gate function, event emission — COMPLETE 2026-03-16
- [x] 02-04-PLAN.md — Tauri wiring: F10 emergency stop hotkey, commands, integration — COMPLETE 2026-03-16

### Phase 3: Web Parser
**Goal**: `web_parser::fetch_and_decode()` returns a typed `BuildPlan` from a pasted d2core.com link, with pinned test vectors, typed error variants, and full GUI preview of parsed build
**Depends on**: Phase 1
**Requirements**: PARSE-01, PARSE-02, PARSE-03, PARSE-04, PARSE-05, PARSE-06, PARSE-07
**Success Criteria** (what must be TRUE):
  1. User can paste a `d2core.com/d4/planner?bd=XXXX` link and the app displays a human-readable preview of skills and paragon board before any automation runs
  2. Parser unit tests against pinned bd= test vectors all pass, verifying known-good builds decode correctly
  3. Invalid or expired build links show a clear, specific error message (not a panic or generic failure)
  4. Paragon board data (board names, node coordinates, glyphs, rotation) is fully parsed into a typed BuildPlan struct
**Plans**: 3 plans

Plans:
- [x] 03-01-PLAN.md — web_parser Rust module: D2CoreClient, extract/parse functions, error types, Tauri command — COMPLETE 2026-03-16
- [x] 03-02-PLAN.md — Pinned test fixtures and unit tests for web_parser — COMPLETE 2026-03-16
- [x] 03-03-PLAN.md — Frontend React UI: link input, build preview card, dark theme — COMPLETE 2026-03-16

### Phase 4: Auto Applier
**Goal**: `auto_applier::executor::run()` applies a full BuildPlan to the character via resolution-adaptive, humanized click sequences with per-step safety re-checks, progress events, and cancel support
**Depends on**: Phase 2, Phase 3
**Requirements**: APPLY-01, APPLY-02, APPLY-03, APPLY-04, APPLY-05, APPLY-06, APPLY-07
**Success Criteria** (what must be TRUE):
  1. App applies skill allocations to the character in-game by clicking the correct UI elements at the correct coordinates for the detected resolution
  2. App applies paragon board choices by navigating the board UI and clicking the correct nodes
  3. Click coordinates are correct for at minimum 1080p, 1440p, and 4K resolutions (verified by coordinate mapping unit tests)
  4. All click events include coordinate jitter and timing variation so they do not carry the synthetic-input signature
  5. User can stop the apply process at any time and the automation halts within one click step
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — Pure foundations: coords.rs, humanize.rs, error.rs, Cargo.toml deps, unit tests — COMPLETE 2026-03-16
- [ ] 04-02-PLAN.md — Executor loop, step sequencer, safety integration, Tauri command wiring

### Phase 5: GUI + Integration
**Goal**: Complete frontend UI connects all backend modules into a single end-to-end user flow: paste link → preview build → start automation → monitor progress → stop if needed
**Depends on**: Phase 2, Phase 3, Phase 4
**Requirements**: GUI-01, GUI-02, GUI-03, GUI-04, GUI-05, GUI-06
**Success Criteria** (what must be TRUE):
  1. User can paste a d2core build link into the input field, see the parsed build preview, and start the apply process without touching any other tool
  2. User sees real-time per-step progress (which skill or paragon node is currently being applied) during automation
  3. User sees clear, specific error messages for every failure state: game not found, invalid link, unsafe game state
  4. The app window remains interactive (scrollable, clickable) while a long-running automation sequence is running in the background
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5
Note: Phase 2 and Phase 3 can be worked in parallel once Phase 1 is complete (web_parser has no Windows API dependency).

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Research Spike | 1/1 | Complete    | 2026-03-16 |
| 2. Scaffold + Safety + Game Capture | 4/4 | Complete | 2026-03-16 |
| 3. Web Parser | 3/3 | Complete   | 2026-03-16 |
| 4. Auto Applier | 1/2 | In progress | - |
| 5. GUI + Integration | 0/? | Not started | - |
