# Project Research Summary

**Project:** Diablo IV Build Applier
**Domain:** Windows desktop game automation tool (Rust + Tauri)
**Researched:** 2026-03-16
**Confidence:** MEDIUM (one critical unverified gap: d2core.com skills data in API)

## Executive Summary

This project is a Windows desktop automation tool that reads a Diablo IV build from a d2core.com planner link and applies it in-game via simulated mouse input. The recommended expert approach is a Rust + Tauri v2 desktop application with a clear three-module Rust backend (web parser, game capture, auto-applier), a thin TypeScript/Vite frontend, and strict separation of Tauri command handlers from business logic to preserve unit testability. No existing tool occupies this niche — Diablo4Companion parses d2core builds but only for gear tracking, not in-game application.

The single most important finding is that Diablo IV has no offline mode. Every session is an always-online Battle.net connection, which invalidates the assumed "offline = safe" safety gate. The safety invariant must be redefined as game-UI-state detection (is the character on the skill/paragon assignment screen?) rather than network connectivity. Additionally, the d2core.com API's inclusion of skill allocation data is unverified — Diablo4Companion only reads gear and paragon fields — and this gap must be resolved empirically before any parser code is written.

The primary risks are: (1) d2core encoding change breaking the parser with no notice, (2) Warden anti-cheat detecting synthetic mouse input via the LLMHF_INJECTED flag if click humanization is absent, (3) DPI scaling causing all click coordinates to be wrong on non-100%-scale displays, and (4) game capture failing under exclusive fullscreen. All four have clear prevention strategies but must be addressed from day one — they are not retrofit-friendly.

---

## Key Findings

### Recommended Stack

The stack is well-determined by the project's own constraints. Rust 1.85+ and Tauri 2.10.3 are the core; Tauri's v2 line is the current stable release (Oct 2024), produces a ~4 MB installer, and provides a typed IPC bridge between Rust backend and TypeScript frontend. The critical version constraint is that `windows-capture 1.5` pins `windows = "^0.61.3"` — the explicit `windows` crate dependency must stay at 0.61, not the current 0.62, to avoid build conflicts.

**Core technologies:**
- **Rust 1.85+**: Backend language — zero-cost abstractions, Windows FFI, required by project spec
- **Tauri 2.10.3**: Desktop shell + IPC bridge — 4 MB binary, Rust backend, typed invoke()/emit() API
- **TypeScript + Vite 6.x**: Frontend — official Tauri template, hot-reload dev experience
- **windows 0.61**: Win32 API bindings — Microsoft's official Rust-for-Windows crate; pinned to 0.61 for windows-capture compatibility
- **windows-capture 1.5.0**: Game window capture — uses Windows Graphics Capture API (WGC), handles DWM compositing and borderless windowed mode; not BitBlt
- **enigo 0.6.x**: Mouse automation — wraps SendInput; must be called from `spawn_blocking` in async Tauri commands
- **serde + serde_json 1.0**: JSON parsing of d2core API response
- **flate2 1.1 / base64 0.22**: Payload decoding (if bd= format involves compression + encoding; verify empirically)

### Expected Features

The feature set is tightly constrained by the core data flow: parse link → preview build → safety check → apply skills → apply paragon. Every P1 feature is a dependency in this chain; none can be skipped.

**Must have (table stakes):**
- Parse d2core.com build link (bd= → API JSON) — core premise; everything else depends on this
- Skills data extraction from d2core — currently unverified in API; BLOCKER if absent
- Display decoded build preview before applying — safety net; required before automation starts
- Online/offline (game-state) safety check — non-negotiable; must block at game-UI state level, not network
- Detect Diablo IV game window + resolution — prerequisite for all coordinate math
- Resolution-adaptive click coordinate mapping — required for tool to work across PC setups
- Apply skill points in dependency order — first half of build application
- Apply paragon boards (board selection, rotation, node clicks) — second half
- Per-build variant selection — d2core variants[] array; users must choose which variant
- Start/stop controls — bare minimum for any automation tool

**Should have (competitive):**
- "Refund all first" automation — removes manual pre-step friction
- Dry-run / preview mode — highlights what will be clicked; builds user trust
- Per-step progress feedback — essential for a 30–90 second automation sequence

**Defer (v2+):**
- Support for other build planners (Maxroll, D4Builds, Mobalytics) — separate reverse-engineering per site
- Pause and resume mid-apply — complex state tracking; not critical for launch
- Build history / recently applied — QoL storage feature; not launch-blocking

### Architecture Approach

The architecture follows the Tauri v2 "thin command, fat module" pattern: three independent Rust modules (`web_parser`, `game_capture`, `auto_applier`) each expose a pure Rust API with no Tauri types in their public interfaces, making them unit-testable with `cargo test` alone. Tauri `#[tauri::command]` handlers are one-liners that delegate to these modules, acquire/release the shared `Mutex<AppState>`, and return. The apply sequence runs as a background tokio task that streams progress events to the frontend via `AppHandle::emit()`.

**Major components:**
1. `web_parser` — Fetches d2core.com URL, decodes bd= parameter, produces `BuildPlan` struct; no Tauri dependency
2. `game_capture` — Win32 FindWindow + WGC screenshot, resolution detection, game-state safety check; no Tauri dependency
3. `auto_applier` — Consumes BuildPlan + GameState, emits resolution-mapped click sequences via enigo, streams progress events via AppHandle; only module that takes an AppHandle
4. `commands/` — Thin Tauri command handlers; delegate to modules above, manage AppState lock
5. Frontend (TypeScript/Vite) — Renders build preview, exposes start/stop controls, subscribes to progress events via listen()
6. `AppState` (Mutex) — Holds Option<BuildPlan>, GameState, apply phase, AtomicBool cancel flag

### Critical Pitfalls

1. **Diablo IV is always-online — offline mode does not exist** — Redefine "safe to automate" as game-UI-state (skill screen open, solo play) rather than network connectivity. Never use ping/network checks as the safety gate.

2. **SendInput sets LLMHF_INJECTED — Warden can observe it** — Add sub-pixel jitter (±2px), randomized bezier mouse paths, and timing variation (±20%) to every click. This is a safety feature, not polish, and cannot be retrofitted after the fact.

3. **DPI scaling makes all coordinates wrong** — Declare the process as Per-Monitor DPI Aware v2 in the app manifest. Use `GetDpiForWindow` to normalize all coordinates. Test at 100%, 125%, and 150% Windows display scaling before any other feature is complete.

4. **d2core.com encoding is undocumented and volatile** — Write pinned test vectors against specific bd= values before shipping. Return typed errors (`UnsupportedEncodingVersion`, `ParseFailed`), never panics. Version-detect the encoding format on every parse.

5. **Exclusive fullscreen causes black-frame capture** — Use WGC (windows-capture), not BitBlt. Validate that captured frames are non-blank. Warn the user to run the game in borderless windowed mode.

6. **Blocking Tauri commands freeze the UI** — All long-running operations must be `async`. Use `tokio::task::spawn_blocking` for Win32 blocking calls. The stop button requires an `AtomicBool` cancel flag checked between every click step.

---

## Implications for Roadmap

Based on the strict dependency chain discovered in research, the build order is non-negotiable: the parser must exist before the applier can be built, and the game capture module must be proven before coordinate math is written. The safety invariant redefinition is an architectural decision that must precede all automation code.

### Phase 1: Spike — Verify d2core Skills Data

**Rationale:** The entire project is blocked by one empirical question: does the `function-planner-queryplan` API response include skill allocation data? This is a 30-minute browser devtools investigation that determines whether the web_parser module is straightforward or requires DOM parsing fallback. Do this before writing a single line of production code.
**Delivers:** Confirmed knowledge of bd= encoding format and skills data availability; decision on parser approach (direct HTTP vs. headless browser fallback)
**Addresses:** Skills data blocker from FEATURES.md; d2core encoding volatility pitfall from PITFALLS.md
**Avoids:** Building the wrong parser architecture, wasting a full phase on an approach that cannot work

### Phase 2: Project Scaffold + Safety Module

**Rationale:** Safety invariant redefinition must happen before automation code. Establishing the Tauri project structure, AppState types, and the corrected safety gate first prevents the most dangerous architectural mistake (building on a wrong safety assumption).
**Delivers:** Working Tauri app skeleton; AppState, BuildPlan, GameState types in state.rs; safety module that checks game-UI state (not network connectivity); DPI-aware process manifest
**Uses:** Tauri 2.10.3, Rust workspace layout from ARCHITECTURE.md, windows crate 0.61
**Implements:** Project structure (commands/, web_parser/, game_capture/, auto_applier/ directories); main.rs → lib.rs split
**Avoids:** Always-online pitfall; DPI scaling pitfall (declared at manifest level)

### Phase 3: web_parser Module

**Rationale:** Parser is the first dependency in the apply chain. Must be complete and tested before game_capture or auto_applier can be meaningfully integrated. Results of Phase 1 spike determine the exact implementation path.
**Delivers:** `web_parser::fetch_and_decode()` returning a typed `BuildPlan`; pinned test vectors for bd= decode; typed error variants; variant selection support
**Uses:** reqwest (or built-in HTTP), base64 0.22, flate2 1.1, serde_json 1.0
**Implements:** `web_parser/fetcher.rs` + `web_parser/decoder.rs`; `commands/parse.rs` thin handler
**Avoids:** d2core encoding volatility pitfall; parser panic on encoding change

### Phase 4: game_capture Module

**Rationale:** Resolution detection and game-state safety check are prerequisites for all coordinate math. Must be proven before any click coordinates are written.
**Delivers:** `game_capture::capture_state()` returning `GameState { window_hwnd, resolution, is_online_safe }`; black-frame detection; borderless windowed mode enforcement; DPI-normalized window rect
**Uses:** windows crate 0.61 (FindWindow, GetWindowRect, GetDpiForWindow), windows-capture 1.5.0
**Implements:** `game_capture/window.rs`, `game_capture/screenshot.rs`, `game_capture/online_check.rs`
**Avoids:** DPI scaling pitfall; exclusive fullscreen black-frame pitfall

### Phase 5: auto_applier Module

**Rationale:** Depends on BuildPlan (Phase 3) and GameState (Phase 4) both being complete. This is the highest-risk module from an anti-cheat perspective; humanization must be built in from the start.
**Delivers:** `auto_applier::executor::run()` with full click sequence for skills + paragon boards; per-step safety re-check; jitter + timing variation; AtomicBool cancel flag; AppHandle progress events
**Uses:** enigo 0.6.x (via spawn_blocking), tokio 1.x async, AppHandle emit
**Implements:** `auto_applier/safety.rs`, `auto_applier/resolution_map.rs`, `auto_applier/click_sequencer.rs`, `auto_applier/executor.rs`; `commands/apply.rs` thin handler
**Avoids:** SendInput/Warden detection pitfall; blocking Tauri command pitfall; single safety-check pitfall

### Phase 6: Frontend GUI + End-to-End Integration

**Rationale:** GUI can be scaffolded in parallel with early phases using mock data, but full wiring requires all three backend modules. This phase connects everything and delivers the complete user flow.
**Delivers:** Build URL input, decoded build preview panel, variant selector, start/stop controls, per-step progress display, error/abort messaging; full end-to-end UX
**Uses:** TypeScript + Vite, Tauri invoke()/listen() API
**Implements:** `src/components/BuildInput.ts`, `BuildPreview.ts`, `ApplyControls.ts`; event listener lifecycle management
**Avoids:** UX pitfalls (no preview before apply, no progress indicator, no stop button feedback)

### Phase 7: v1.x Polish (Post-Validation)

**Rationale:** Add friction-reducing features after core apply flow is proven working on real user machines.
**Delivers:** "Refund all first" automation; dry-run / click-preview mode; detailed per-step progress log
**Avoids:** Scope creep before core is validated

### Phase Ordering Rationale

- Phase 1 (spike) is justified because it eliminates a hard blocker before any engineering investment
- Phases 3 and 4 can run in parallel after Phase 2 scaffold is complete — web_parser has no Windows API dependency
- Phase 5 strictly depends on both Phase 3 and Phase 4 outputs (BuildPlan + GameState types must be stable)
- Safety module design is embedded in Phase 2, not deferred, because the wrong safety invariant poisons every subsequent phase
- DPI-awareness and humanization are Phase 2 and Phase 5 concerns respectively — both are declared non-retrofit-able by pitfalls research

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (spike):** This IS the research; no standard pattern — must be done empirically with browser devtools on a live d2core.com build URL
- **Phase 4 (game_capture):** DPI-aware coordinate normalization in Rust + Tauri has sparse documentation; needs targeted research on `GetDpiForWindow` integration with Tauri's Win32 process context
- **Phase 5 (auto_applier):** Paragon board UI structure (21x21 grid, board selection wheel, rotation controls) requires empirical reverse-engineering of the in-game UI layout per resolution

Phases with standard patterns (skip research-phase):
- **Phase 2 (scaffold):** Tauri v2 project structure is fully documented and follows established patterns
- **Phase 3 (web_parser):** HTTP + JSON parsing in Rust is well-documented; only the d2core-specific format needs the Phase 1 spike
- **Phase 6 (frontend):** Tauri TypeScript frontend patterns are well-documented; no novel integrations

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core Tauri/Rust stack is fully documented; version compatibility constraints are verified (windows-capture 1.5 pins windows 0.61) |
| Features | MEDIUM | P1 features are well-scoped; skills data in d2core API is the critical unverified gap — HIGH confidence on everything except skills presence |
| Architecture | HIGH | Tauri v2 patterns are official-documented; module boundaries and data flow are derived from official architecture docs |
| Pitfalls | MEDIUM | Warden behavior is community-verified; DPI and async pitfalls are Microsoft-documented; d2core encoding volatility is inferred (no public docs) |

**Overall confidence:** MEDIUM — HIGH on technical approach, gated by the d2core skills data empirical gap

### Gaps to Address

- **d2core.com skills data in API:** The `function-planner-queryplan` response structure from Diablo4Companion source only shows `gear` and `paragon` fields. Whether skills data exists in the full response is unknown. Action: inspect a live d2core.com planner URL in browser devtools before writing any parser code. If skills are absent, the fallback is page DOM parsing — more fragile and should be documented as a known risk.

- **In-game paragon board UI coordinate map:** The exact pixel/fraction positions of paragon board UI elements (board wheel, rotation handle, individual node positions on 21x21 grid) across target resolutions (1080p, 1440p, 4K) are unknown. Action: empirical measurement during Phase 4/5, likely requiring a dedicated mapping session with the game running at each target resolution.

- **d2core.com API authentication requirements:** Research inferred that public planner links require no auth for the `function-planner-queryplan` endpoint (as Diablo4Companion uses it without credentials), but this needs confirmation. If auth headers are required, the direct-HTTP approach may need to adopt Diablo4Companion's Selenium/DevTools interception pattern instead.

---

## Sources

### Primary (HIGH confidence)
- https://v2.tauri.app/ — Tauri v2 architecture, state management, commands, events, project structure
- https://github.com/josdemmers/Diablo4Companion — D2CoreBuildJson.cs and BuildsManagerD2Core.cs; d2core API structure and JSON schema
- https://docs.rs/base64/latest — base64 0.22.1 URL_SAFE engine
- https://docs.rs/crate/flate2/latest — flate2 1.1 DEFLATE/zlib/gzip support
- https://github.com/microsoft/windows-rs/releases — windows crate version history; 0.61 vs 0.62 compat
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-input — SendInput LLMHF_INJECTED flag (Win32 docs)
- https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture — WGC API
- https://www.pcgamesn.com/diablo-4/offline + https://www.gamespot.com/articles/diablo-4-doesnt-have-an-offline-mode-all-versions-/ — D4 always-online confirmation

### Secondary (MEDIUM confidence)
- https://github.com/NiiightmareXD/windows-capture — windows-capture 1.5.0 capabilities and API
- https://github.com/enigo-rs/enigo — enigo 0.6.x cross-platform input simulation
- https://hackmd.io/@littlex/SJzt2OEdh — community reference confirming bd= is a short build ID (not encoded blob)
- https://guidedhacking.com/threads/how-to-detect-input-injection-anticheat-feature.20662/ — input injection detection patterns
- https://sneakycrow.dev/blog/2024-05-12-running-async-tasks-in-tauri-v2 — Tauri v2 async task patterns
- DPI scaling: Microsoft Learn UI Automation and Screen Scaling + DPI and device-independent pixels

### Tertiary (LOW confidence)
- https://www.ownedcore.com/forums/fps/overwatch-exploits-hacks/576716-how-blizzard-detects-ahk.html — Blizzard AHK detection (community forum; consistent with documented LLMHF_INJECTED behavior)
- d2core.com paragon board structure — inferred from Diablo4Companion data model; not directly observed

---
*Research completed: 2026-03-16*
*Ready for roadmap: yes*
