# Architecture Research

**Domain:** Rust + Tauri Windows desktop automation tool (game build applier)
**Researched:** 2026-03-16
**Confidence:** HIGH (Tauri/Rust patterns), MEDIUM (anti-cheat/safety constraints)

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        GUI Layer (Frontend)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  Build Input  │  │ Build Preview│  │  Apply Controls          │  │
│  │  (URL paste)  │  │ (skills/prgn)│  │  (start/stop/progress)   │  │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬──────────────┘  │
│         │                 │                       │                  │
│         └─────────────────┴───────────────────────┘                 │
│                           │  invoke() / listen()                     │
├───────────────────────────┼─────────────────────────────────────────┤
│              IPC Bridge (Tauri Command System)                       │
│  Commands: parse_build | get_game_state | apply_build | cancel       │
│  Events:  apply_progress | apply_complete | safety_abort             │
├───────────────────────────┼─────────────────────────────────────────┤
│                    Rust Backend (src-tauri/src/)                     │
│                                                                      │
│  ┌─────────────┐   ┌──────────────┐   ┌──────────────────────────┐ │
│  │ web_parser  │   │ game_capture │   │     auto_applier         │ │
│  │             │   │              │   │                          │ │
│  │ Decode bd=  │   │ FindWindow   │   │ Safety gate (online?)    │ │
│  │ URL param   │   │ Screenshot   │   │ Resolution mapping       │ │
│  │ → BuildPlan │   │ Resolution   │   │ Click sequencer          │ │
│  │             │   │ detect       │   │ AppHandle events out     │ │
│  └──────┬──────┘   └──────┬───────┘   └───────────┬─────────────┘ │
│         │                 │                        │               │
│         └─────────────────┴──────────── AppState ──┘               │
│                                    (Mutex<AppState>)                │
├─────────────────────────────────────────────────────────────────────┤
│                    OS / Windows API Layer                            │
│  ┌──────────────┐  ┌─────────────────┐  ┌──────────────────────┐  │
│  │  HTTP client │  │  windows-capture │  │  enigo / SendInput   │  │
│  │  (reqwest)   │  │  (GFX Capture)   │  │  (UI click sim)      │  │
│  └──────────────┘  └─────────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Key Boundary |
|-----------|---------------|--------------|
| `web_parser` | Fetch d2core.com URL, decode `bd=` parameter, produce a `BuildPlan` struct (skills + paragon choices) | Pure logic — no I/O except HTTP. Input: URL string. Output: `BuildPlan` or error. |
| `game_capture` | Find the Diablo IV window via Win32 API, capture a screenshot, detect resolution, determine online/offline state | Read-only window inspection — no clicks, no writes. Output: `GameState` struct. |
| `auto_applier` | Consume `BuildPlan` + `GameState`, emit resolution-mapped click sequences via `enigo`, stream progress events to frontend via `AppHandle`, abort if online | Pure executor. Never accesses d2core or screen directly — depends on other modules' outputs. |
| `gui` (frontend) | Render build preview, expose start/stop controls, display live progress via event listeners | Thin presentation layer. No business logic. All actions go through `invoke()`. |
| Shared state (`AppState`) | Hold `Option<BuildPlan>`, `GameState`, apply phase, and cancellation flag; protected by `Mutex` | Injected into Tauri commands via `State<Mutex<AppState>>`. Never accessed outside command handlers. |

## Recommended Project Structure

```
diablo4-tool/
├── package.json                  # Frontend build (Vite + React/vanilla)
├── index.html
├── src/                          # Frontend source
│   ├── main.ts
│   ├── components/
│   │   ├── BuildInput.ts         # URL paste + parse trigger
│   │   ├── BuildPreview.ts       # Skill + paragon display
│   │   └── ApplyControls.ts      # Start/stop + progress bar
│   └── api.ts                    # Typed wrappers around invoke()
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    └── src/
        ├── main.rs               # Desktop entry: calls lib::run()
        ├── lib.rs                # Tauri builder, state init, command registration
        ├── state.rs              # AppState struct, BuildPlan, GameState types
        ├── commands/
        │   ├── mod.rs            # pub mod declarations
        │   ├── parse.rs          # parse_build command (delegates to web_parser)
        │   ├── capture.rs        # get_game_state command (delegates to game_capture)
        │   └── apply.rs          # apply_build / cancel_apply commands (delegates to auto_applier)
        ├── web_parser/
        │   ├── mod.rs
        │   ├── fetcher.rs        # HTTP fetch of d2core URL
        │   └── decoder.rs        # bd= parameter decoding logic
        ├── game_capture/
        │   ├── mod.rs
        │   ├── window.rs         # FindWindow, resolution detection
        │   ├── screenshot.rs     # windows-capture integration
        │   └── online_check.rs   # Online/offline state detection
        └── auto_applier/
            ├── mod.rs
            ├── safety.rs         # Online guard — aborts if online
            ├── resolution_map.rs # Coordinate tables per resolution
            ├── click_sequencer.rs# Ordered click plan generation
            └── executor.rs       # enigo execution + AppHandle progress events
```

### Structure Rationale

- **`commands/`**: Thin Tauri command handlers only — they validate inputs, acquire state lock, call into the real module, release lock, and return. No business logic lives here.
- **`web_parser/`, `game_capture/`, `auto_applier/`**: Each is a standard Rust module (no Tauri dependency in their pub API). This keeps them unit-testable without a Tauri runtime.
- **`state.rs`**: Centralises all shared types. When `BuildPlan` or `GameState` evolve, only this file and its callers change.
- **`main.rs` → `lib.rs` split**: Required by Tauri for mobile-compatible entry points; also enables integration testing of `lib::run()`.

## Architectural Patterns

### Pattern 1: Thin Command, Fat Module

**What:** Tauri `#[tauri::command]` functions are one-liners that delegate immediately to module functions. All logic — parsing, safety checks, click sequencing — lives in the plain Rust modules, not in the command handler.

**When to use:** Always, for this project. Keeps modules testable with `cargo test` without a running Tauri app.

**Trade-offs:** Slight indirection; worth it for testability and clarity.

**Example:**
```rust
// commands/parse.rs
#[tauri::command]
pub async fn parse_build(
    url: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<BuildPlan, String> {
    let plan = web_parser::fetch_and_decode(&url).await.map_err(|e| e.to_string())?;
    let mut s = state.lock().unwrap();
    s.build_plan = Some(plan.clone());
    Ok(plan)
}
```

### Pattern 2: AppHandle Event Streaming for Long Operations

**What:** `auto_applier::executor` receives a cloned `AppHandle` and emits progress events (`apply_progress`, `apply_complete`, `safety_abort`) as it works through the click sequence. The frontend listens with `listen()`.

**When to use:** Any operation that takes more than ~200ms and has observable sub-steps (applying 20+ skills and paragon nodes qualifies).

**Trade-offs:** Requires the frontend to manage event listener lifecycle (unlisten on component unmount). More complex than a single `invoke()` but essential for user feedback.

**Example:**
```rust
// auto_applier/executor.rs
pub async fn run(plan: BuildPlan, game: GameState, app: AppHandle) -> Result<(), ApplyError> {
    let steps = click_sequencer::build_sequence(&plan, &game)?;
    for (i, step) in steps.iter().enumerate() {
        safety::assert_offline(&game)?;   // re-check each step
        step.execute()?;
        app.emit("apply_progress", ProgressPayload { step: i, total: steps.len() }).ok();
        tokio::time::sleep(Duration::from_millis(step.delay_ms)).await;
    }
    app.emit("apply_complete", ()).ok();
    Ok(())
}
```

### Pattern 3: Module-Level Safety Gate

**What:** `game_capture::online_check` is called at two points: (1) before `auto_applier` starts, and (2) before every individual click action inside the executor loop. If the game transitions online mid-apply, the sequence aborts immediately.

**When to use:** Any time `auto_applier` is about to perform a click. Non-negotiable given Warden anti-cheat risk.

**Trade-offs:** Slight overhead per click (process enumeration or network check). Acceptable — human-speed click sequences are slow anyway.

## Data Flow

### Primary Flow: Apply Build from URL

```
User pastes URL
    ↓
GUI: invoke("parse_build", { url })
    ↓
commands/parse.rs → web_parser::fetch_and_decode(url)
    ↓ HTTP GET d2core.com/d4/planner?bd=XXXX
    ↓ decoder::decode_bd_param(raw_bytes) → BuildPlan
    ↓ stored in AppState.build_plan
    ↓ BuildPlan returned to frontend
    ↓
GUI renders BuildPreview (skills + paragon nodes)

User clicks "Apply"
    ↓
GUI: invoke("apply_build")
    ↓
commands/apply.rs
    ↓ acquire AppState → read BuildPlan
    ↓ game_capture::capture_state() → GameState { resolution, is_online }
    ↓ spawn tokio task: auto_applier::executor::run(plan, game, app_handle)
    ↓ return Ok immediately (apply runs in background)

auto_applier::executor::run (background task)
    ↓ safety::assert_offline() — abort if online
    ↓ click_sequencer::build_sequence() → Vec<ClickStep>
    ↓ for each step:
        ↓ safety::assert_offline()
        ↓ enigo::mouse_move + click
        ↓ app.emit("apply_progress", { step, total })
        ↓ sleep(delay_ms)
    ↓ app.emit("apply_complete", {})

GUI receives "apply_progress" events → updates progress bar
GUI receives "apply_complete" → shows success
```

### Safety Abort Flow

```
game_capture::online_check detects online state
    ↓
safety::assert_offline() returns Err(SafetyError::GameIsOnline)
    ↓
executor::run propagates error
    ↓
app.emit("safety_abort", { reason: "Game detected as online" })
    ↓
GUI shows warning, disables Apply button
```

### Key Data Structures

```
BuildPlan {
    skills: Vec<SkillAllocation>,     // skill name + rank
    paragon_boards: Vec<ParagonNode>, // node id + type
}

GameState {
    window_hwnd: HWND,
    resolution: (u32, u32),           // e.g. (2560, 1440)
    is_online: bool,                  // safety gate value
}

AppState {
    build_plan: Option<BuildPlan>,
    game_state: Option<GameState>,
    apply_phase: ApplyPhase,          // Idle / Running / Complete / Aborted
    cancel_flag: Arc<AtomicBool>,     // checked by executor each step
}
```

## Suggested Build Order (Phase Dependencies)

The four modules have a strict dependency chain for the apply flow, which dictates build order:

```
Phase 1: web_parser
    No external Rust dependencies — just HTTP + decode logic.
    Deliverable: BuildPlan type + decode roundtrip tests.

Phase 2: game_capture
    Depends on: Windows API (windows crate, windows-capture)
    No dependency on web_parser.
    Deliverable: GameState type + resolution detection + online check.

Phase 3: auto_applier
    Depends on: BuildPlan (from Phase 1) + GameState (from Phase 2) + enigo
    Cannot be fully implemented without both above modules complete.
    Deliverable: Click sequence execution + safety gate + progress events.

Phase 4: gui + Tauri wiring
    Depends on: All three backend modules (commands wrap them all)
    Deliverable: End-to-end UX — paste URL, preview, apply, progress, stop.
```

The GUI can be scaffolded in parallel with Phase 1 (mock data for preview), but cannot be wired to real commands until the backing module exists.

## Anti-Patterns

### Anti-Pattern 1: Business Logic in Command Handlers

**What people do:** Write parsing, click logic, and safety checks directly inside `#[tauri::command]` functions.

**Why it's wrong:** Commands require a Tauri `State<>` parameter — they cannot be called from `cargo test` without spinning up a full Tauri app. All real logic becomes untestable.

**Do this instead:** Command handlers are one-liners. Every real function lives in `web_parser`, `game_capture`, or `auto_applier` with no Tauri import, and is tested directly.

### Anti-Pattern 2: Blocking the Tauri Async Runtime in Apply

**What people do:** Call `std::thread::sleep()` or blocking Win32 APIs inside an `async` Tauri command.

**Why it's wrong:** Tauri uses Tokio. Blocking calls stall the async thread pool, causing the frontend to become unresponsive and preventing cancel signals from being processed.

**Do this instead:** Use `tokio::time::sleep()` for delays, spawn blocking Win32 calls with `tokio::task::spawn_blocking()`, and check `cancel_flag` between steps in the async executor.

### Anti-Pattern 3: Single Online Check at Start

**What people do:** Check `is_online` once before beginning the apply sequence, then proceed without re-checking.

**Why it's wrong:** The game may transition from offline to online mid-apply (e.g., Battle.net reconnects). A single up-front check does not protect against this.

**Do this instead:** Re-check `is_online` before every click step. The overhead is negligible vs. the safety guarantee.

### Anti-Pattern 4: Hardcoding Pixel Coordinates

**What people do:** Record absolute (x, y) coordinates for UI elements at one resolution and use them for all resolutions.

**Why it's wrong:** Diablo IV scales UI elements with resolution. Coordinates at 1920x1080 are wrong at 2560x1440 or 3840x2160.

**Do this instead:** Store coordinates as fractions of screen dimensions or as per-resolution lookup tables in `resolution_map.rs`. Detect resolution at runtime from `GameState` before building the click sequence.

### Anti-Pattern 5: PostMessage for Click Simulation

**What people do:** Use `PostMessage(hwnd, WM_LBUTTONDOWN, ...)` to send clicks directly to the Diablo IV window handle.

**Why it's wrong:** PostMessage bypasses the system input queue and does not set `LLMHF_INJECTED`. This is more suspicious than `SendInput` to some detection schemes, and it can fail in fullscreen DirectX windows. Warden can detect messages arriving outside the normal input pipeline.

**Do this instead:** Use `enigo` (which wraps `SendInput`) with the game window in the foreground. Keep click timing human-like (50–200ms delays). Operate offline-only to minimise the risk window.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| d2core.com/d4/planner | HTTP GET with `reqwest` in `web_parser::fetcher` | No auth required for public links. Parse `bd=` query param from URL before fetching if the encoded data is embedded in the URL directly. |
| Diablo IV game window | Win32 `FindWindowEx` + `windows-capture` in `game_capture::window` | Game must be running and not minimised for capture. Resolution auto-detected from window rect. |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `web_parser` → commands | Returns `BuildPlan` or `anyhow::Error` | No Tauri types in web_parser's pub API |
| `game_capture` → commands | Returns `GameState` or error | `HWND` stored in `GameState`; valid only for process lifetime |
| `commands` → `auto_applier` | Passes `BuildPlan`, `GameState`, cloned `AppHandle`, and `Arc<AtomicBool>` cancel flag | `AppHandle` is the only Tauri type crossing into auto_applier |
| `auto_applier` → GUI | `AppHandle::emit()` push events: `apply_progress`, `apply_complete`, `safety_abort` | Frontend uses `listen()` to subscribe; must `unlisten()` on teardown |
| All modules → `AppState` | `state.lock().unwrap()` inside command handlers | Mutex ensures serial access; commands should hold lock briefly |

## Scaling Considerations

This is a local desktop tool; "scaling" means handling complexity growth, not user load.

| Concern | Current scope | If feature scope grows |
|---------|--------------|------------------------|
| New game resolutions | Add row to `resolution_map.rs` lookup table | If Blizzard adds dynamic UI scaling, switch from lookup table to proportional coordinate math |
| Additional build planner sites | Trait `BuildDecoder` with a `D2corDecoder` impl; add impls per site | Keep `web_parser` pub API generic from day one to avoid rewrites |
| More Diablo IV patch UI changes | Update coordinate tables in `resolution_map.rs` | CI test with reference screenshots to catch coordinate drift |
| Cancel mid-apply | `AtomicBool` cancel flag checked per step | Sufficient for this tool; no need for full task cancellation framework |

## Sources

- [Tauri v2 Architecture Concepts](https://v2.tauri.app/concept/architecture/) — HIGH confidence
- [Tauri v2 State Management](https://v2.tauri.app/develop/state-management/) — HIGH confidence
- [Tauri v2 Calling Rust from Frontend (Commands)](https://v2.tauri.app/develop/calling-rust/) — HIGH confidence
- [Tauri v2 Calling Frontend from Rust (Events)](https://v2.tauri.app/develop/calling-frontend/) — HIGH confidence
- [Tauri v2 Project Structure](https://v2.tauri.app/start/project-structure/) — HIGH confidence
- [How to Reasonably Keep Tauri Commands Organized in Rust](https://dev.to/n3rd/how-to-reasonably-keep-your-tauri-commands-organized-in-rust-2gmo) — MEDIUM confidence
- [windows-capture crate (Graphics Capture API)](https://github.com/NiiightmareXD/windows-capture) — MEDIUM confidence
- [enigo cross-platform input simulation](https://github.com/enigo-rs/enigo) — MEDIUM confidence
- [SendInput detection and anti-cheat considerations](https://guidedhacking.com/threads/low-level-methods-of-sending-mouse-input-that-bypass-anticheat.14555/) — MEDIUM confidence (community forum, verified against Windows API docs pattern)
- [Long-running async tasks in Tauri v2](https://sneakycrow.dev/blog/2024-05-12-running-async-tasks-in-tauri-v2) — MEDIUM confidence

---
*Architecture research for: Diablo IV Build Applier — Rust + Tauri desktop automation tool*
*Researched: 2026-03-16*
