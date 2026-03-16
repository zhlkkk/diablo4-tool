# Phase 2: Scaffold + Safety + Game Capture - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Working Tauri v2 project skeleton with DPI-aware manifest, typed shared data structures (AppState, BuildPlan, GameState), a safety module that gates automation on game-UI-state (skill tree / paragon board screen detection via pixel sampling), and a game window capture module that finds the D4 window, detects resolution, handles DPI normalization, and captures screenshots for state detection.

</domain>

<decisions>
## Implementation Decisions

### Safe-state detection
- Pixel sampling at known screen coordinates to detect skill tree / paragon board screens
- Re-check game state before every click action (maximum safety, ~1-5ms overhead per check)
- On safety failure: immediate halt + emit safety_abort event to GUI + log reason. User must manually restart when game is back in safe state
- Safety events displayed in GUI status area only (no log file)

### Emergency stop
- Global hotkey: F10 (system-wide, works even when D4 has focus)
- No GUI-only stop button needed at this phase (GUI is Phase 5)
- After emergency stop, track progress so user can resume from where it stopped

### Game window capture
- Find D4 window via Win32 FindWindowW using known window class ("D3 Main Window Class"), fallback to EnumWindows + title match
- Support 1080p, 1440p, and 4K resolutions at launch with calibrated coordinate maps
- Detect exclusive fullscreen and warn user to switch to borderless windowed; block automation in exclusive fullscreen
- DPI normalization via GetDpiForWindow (Per-Monitor DPI Aware v2 manifest)

### Tauri scaffold
- Tauri v2 (current stable)
- Single crate with module folders: mod web_parser, mod game_capture, mod auto_applier, mod safety
- React frontend for the webview (Tauri official React template)
- Shared types module (types.rs or models.rs) at crate root for BuildPlan, GameState, AppState

### Claude's Discretion
- Screenshot capture API choice (BitBlt/PrintWindow vs Windows Graphics Capture API) — pick based on compatibility and simplicity
- Exact pixel coordinates for safe-state detection calibration points
- React project setup details (Vite config, folder structure)
- Internal module file organization within each mod folder

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### API and architecture
- `.planning/phases/01-research-spike/SPIKE-FINDINGS.md` — Verified TCB API endpoint, request format, full response schema, test vectors
- `docs/superpowers/specs/2026-03-16-web-parser-design.md` — Full web_parser module design: data structures, error handling, testing strategy
- `.planning/research/ARCHITECTURE.md` — Module architecture diagram, component responsibilities, recommended project structure
- `.planning/research/STACK.md` — Rust crate recommendations (reqwest, serde, windows-rs, enigo, etc.)

### Requirements
- `.planning/REQUIREMENTS.md` — SAFE-01 through SAFE-06, CAPT-01 through CAPT-06 (all Phase 2 requirements)

### Prior decisions
- `.planning/phases/01-research-spike/01-CONTEXT.md` — Phase 1 decisions: direct-http architecture, TCB API details

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no source code exists yet

### Established Patterns
- None — this phase establishes the project patterns

### Integration Points
- SPIKE-FINDINGS.md provides API details for Phase 3 web_parser
- BuildPlan type defined here will be consumed by Phase 3 (parser output) and Phase 4 (applier input)
- GameState type defined here will be consumed by Phase 4 (auto_applier needs resolution + safe-state info)
- Safety module will be called by Phase 4 auto_applier before each click

</code_context>

<specifics>
## Specific Ideas

- Safety invariant is game-UI-state (skill tree / paragon board screen visible), NOT network connectivity — Diablo IV is always-online, there is no offline mode
- DPI-aware v2 manifest is non-retrofittable — must be set up from the start
- Click humanization belongs in Phase 4 auto_applier, not in this phase's safety module
- F10 hotkey chosen because it's not used by Diablo IV and is easy to reach

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-scaffold-safety-game-capture*
*Context gathered: 2026-03-16*
