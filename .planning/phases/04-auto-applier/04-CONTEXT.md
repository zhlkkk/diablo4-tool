# Phase 4: Auto Applier - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

`auto_applier::executor::run()` applies a full BuildPlan to the character via resolution-adaptive, humanized click sequences with per-step safety re-checks, progress events, and cancel/pause support. Takes BuildPlan from AppState and GameState (resolution, window handle) and executes skill allocations then paragon board choices through simulated mouse clicks.

</domain>

<decisions>
## Implementation Decisions

### Coordinate mapping strategy
- Ratio-based scaling from a 1080p reference resolution — define coordinates once at 1080p, scale by width/height ratio for 1440p (×1.333) and 4K (×2.0)
- Named coordinate constants in a dedicated `coords.rs` module with `SkillTreeCoords` and `ParagonBoardCoords` structs
- Coordinates define click targets for: skill tree nodes, skill point allocation buttons, paragon board navigation, paragon node positions
- Reference coordinates need empirical measurement at 1080p (known risk — requires game running)

### Click humanization
- ±2-5 pixels random coordinate jitter per click (subtle enough to look human, small enough not to miss UI targets)
- 50-200ms random delay between click actions (mimics human reaction time)
- Direct mouse move with small random offset — no bezier curve paths (simpler, reliable, click-only automation doesn't benefit from complex paths)
- Humanization is a safety feature, not polish — must be present from day one

### Execution sequence
- Skills first, then paragon boards — skills are simpler UI, validates automation works before tackling complex paragon navigation
- Skill order follows `skill_order` field from BuildPlan/Variant if available; otherwise iterates skill HashMap
- Equip skills applied after regular skill points (separate UI interaction)
- Paragon boards applied in sorted index order (already sorted by parser)

### Progress events & pause/resume
- Per-click step events emitted via Tauri event emitter — ApplyPhase::Running { step, total } already defined in types.rs
- Pause sets cancel_flag + saves current step index in ApplyPhase::Paused { step, total }
- Resume clears cancel_flag + restarts from saved step index
- Complete sets ApplyPhase::Complete; abort sets ApplyPhase::Aborted { reason }

### Safety integration
- Call `safety::assert_safe_state()` before EVERY click action (SAFE-03 carried from Phase 2)
- Capture fresh screenshot via `game_capture::screenshot::capture_window()` before each safety check
- On safety failure: immediate halt, emit SafetyEvent::AutomationAborted, set ApplyPhase::Aborted
- Emergency stop (F10) checked via cancel_flag in assert_safe_state — already wired

### Claude's Discretion
- Exact reference 1080p coordinates for skill tree and paragon board UI elements (requires empirical measurement or placeholder values)
- Mouse input API choice (enigo crate vs windows-rs SendInput)
- Internal module file organization within auto_applier/
- Error recovery strategy for individual click failures (retry once vs abort)
- Whether to add a small initial delay before starting automation (countdown)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing module APIs
- `src-tauri/src/safety/mod.rs` — `assert_safe_state(pixels, width, height, cancel_flag)` pure function, SafetyEvent enum, safety_result_to_event helper
- `src-tauri/src/game_capture/screenshot.rs` — `capture_window(hwnd, width, height)` returns BGRA pixel buffer
- `src-tauri/src/game_capture/window.rs` — `find_diablo_window()` returns HWND
- `src-tauri/src/game_capture/dpi.rs` — `get_game_resolution(hwnd)`, `get_game_dpi(hwnd)`
- `src-tauri/src/types.rs` — BuildPlan, Variant, EquipSkill, ParagonBoard, ApplyPhase, AppState, Resolution, SafetyState, DetectedScreen, GameState

### Architecture and patterns
- `src-tauri/src/lib.rs` — Thin Tauri command pattern, cfg(windows)/cfg(not(windows)) guards, AppState via Mutex
- `.planning/research/ARCHITECTURE.md` — Module architecture diagram, component responsibilities
- `.planning/research/STACK.md` — Rust crate recommendations (enigo for mouse input)

### Requirements
- `.planning/REQUIREMENTS.md` — APPLY-01 through APPLY-07 (all Phase 4 requirements)

### Prior decisions
- `.planning/phases/02-scaffold-safety-game-capture/02-CONTEXT.md` — Safety gate, emergency stop, game capture, resolution support decisions
- `.planning/phases/03-web-parser/03-CONTEXT.md` — First variant auto-selected, BuildPlan in AppState, Tauri command pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `types::ApplyPhase` enum: Idle/Running/Paused/Complete/Aborted — state machine already defined, ready for auto_applier to drive
- `types::AppState`: `cancel_flag: Arc<AtomicBool>` — shared cancellation mechanism already wired to F10 hotkey
- `types::BuildPlan/Variant`: Parsed build data with skills, equip_skills, paragon boards — direct input to applier
- `types::Resolution`: Enum with 1080p/1440p/4K — use for coordinate scaling factor lookup
- `safety::assert_safe_state()`: Pure function gate — call before each click with fresh screenshot pixels
- `game_capture::screenshot::capture_window()`: BGRA pixel buffer capture — feed to safety check
- `game_capture::window::find_diablo_window()`: HWND lookup — needed for screenshot capture and mouse targeting

### Established Patterns
- `cfg(windows)/cfg(not(windows))` guards on all Win32 code — auto_applier MUST use this (mouse input requires Win32)
- Thin Tauri command pattern — `start_apply`, `pause_apply`, `resume_apply` commands delegate to auto_applier module functions
- AppState via `Mutex<AppState>` with `tauri::State` — follow for apply_phase state tracking
- Pure function pattern for testable logic (see safety::detect_safe_state) — coordinate mapping should be pure functions too

### Integration Points
- `lib.rs` invoke_handler: add `start_apply`, `pause_apply`, `resume_apply` Tauri commands
- `AppState.apply_phase`: Update during execution (Idle → Running → Complete/Paused/Aborted)
- `AppState.build_plan`: Read to get target build for automation
- `AppState.game_state`: Read to get resolution for coordinate scaling
- `Cargo.toml`: add mouse input crate (enigo or windows-rs SendInput)
- Tauri events: emit progress and safety events to frontend

</code_context>

<specifics>
## Specific Ideas

- RISK: Exact pixel coordinates for skill tree and paragon board UI elements at 1080p are unknown — requires empirical measurement with game running. Placeholder coordinates acceptable for initial implementation, refined later.
- Paragon board navigation is more complex than skill tree — boards have scroll/zoom, node positions vary by rotation value in BuildPlan
- The `nodes: Vec<String>` field in ParagonBoard contains node identifiers that need mapping to board positions
- Mouse input must target the game window (HWND) specifically — not just screen coordinates
- Between skill application and paragon board application, may need to navigate game menus (close skill tree, open paragon board)

</specifics>

<deferred>
## Deferred Ideas

- Dry-run mode showing planned clicks without executing (APPLY-V2-01)
- Skill refund/reset automation before applying new build (APPLY-V2-02)
- Paragon board reset automation (APPLY-V2-03)
- Variant picker UI for multi-variant builds — Phase 5 or v2
- Skill name display mapping (raw API keys → human names) — Phase 5 or v2

</deferred>

---

*Phase: 04-auto-applier*
*Context gathered: 2026-03-16*
