# Phase 4: Auto Applier - Research

**Researched:** 2026-03-16
**Domain:** Rust mouse automation, Tauri async event streaming, resolution-adaptive coordinate mapping
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Coordinate mapping strategy:**
- Ratio-based scaling from a 1080p reference resolution — define coordinates once at 1080p, scale by width/height ratio for 1440p (x1.333) and 4K (x2.0)
- Named coordinate constants in a dedicated `coords.rs` module with `SkillTreeCoords` and `ParagonBoardCoords` structs
- Coordinates define click targets for: skill tree nodes, skill point allocation buttons, paragon board navigation, paragon node positions
- Reference coordinates need empirical measurement at 1080p (known risk — requires game running)

**Click humanization:**
- +-2-5 pixels random coordinate jitter per click (subtle enough to look human, small enough not to miss UI targets)
- 50-200ms random delay between click actions (mimics human reaction time)
- Direct mouse move with small random offset — no bezier curve paths (simpler, reliable, click-only automation doesn't benefit from complex paths)
- Humanization is a safety feature, not polish — must be present from day one

**Execution sequence:**
- Skills first, then paragon boards — skills are simpler UI, validates automation works before tackling complex paragon navigation
- Skill order follows `skill_order` field from BuildPlan/Variant if available; otherwise iterates skill HashMap
- Equip skills applied after regular skill points (separate UI interaction)
- Paragon boards applied in sorted index order (already sorted by parser)

**Progress events and pause/resume:**
- Per-click step events emitted via Tauri event emitter — ApplyPhase::Running { step, total } already defined in types.rs
- Pause sets cancel_flag + saves current step index in ApplyPhase::Paused { step, total }
- Resume clears cancel_flag + restarts from saved step index
- Complete sets ApplyPhase::Complete; abort sets ApplyPhase::Aborted { reason }

**Safety integration:**
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

### Deferred Ideas (OUT OF SCOPE)
- Dry-run mode showing planned clicks without executing (APPLY-V2-01)
- Skill refund/reset automation before applying new build (APPLY-V2-02)
- Paragon board reset automation (APPLY-V2-03)
- Variant picker UI for multi-variant builds — Phase 5 or v2
- Skill name display mapping (raw API keys to human names) — Phase 5 or v2
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| APPLY-01 | App applies skill allocations to character via resolution-adaptive UI click automation | enigo 0.6.1 API confirmed; coordinate scaling pattern documented; `SkillTreeCoords` struct approach defined |
| APPLY-02 | App applies paragon board choices via resolution-adaptive UI click automation | Same enigo/coords infrastructure; `ParagonBoardCoords` struct; boards applied in sorted index order |
| APPLY-03 | Click coordinates adapt correctly to detected game resolution | `Resolution` enum already in types.rs with 1080p/1440p/4K; scale factors 1.0/1.333/2.0; `scale_coord()` pure function pattern |
| APPLY-04 | Click automation includes humanization (coordinate jitter, timing variation) | rand crate for jitter; `tokio::time::sleep` for delays; +-2-5px jitter, 50-200ms delay; must wrap enigo in spawn_blocking |
| APPLY-05 | App shows real-time progress of apply operation (which skill/node is being applied) | `AppHandle::emit()` with `Emitter` trait; payload must be Clone+Serialize; ApplyPhase::Running{step,total} already defined |
| APPLY-06 | App can pause and resume the apply process | cancel_flag AtomicBool already in AppState; Paused{step,total} variant exists; resume_apply command clears flag and restarts from step |
| APPLY-07 | Unit tests verify coordinate mapping calculations for multiple resolutions | Pure function pattern for scale_coord(); no Win32 dependency needed; all three resolutions tested |
</phase_requirements>

## Summary

Phase 4 implements `auto_applier::executor::run()`, which takes a `BuildPlan` from `AppState`, acquires a fresh `GameState`, and applies skill + paragon choices via simulated mouse clicks. All upstream infrastructure is complete and confirmed: `safety::assert_safe_state()`, `game_capture::screenshot::capture_window()`, `types::BuildPlan/Variant/ParagonBoard/ApplyPhase`, and `AppState.cancel_flag` are production-ready.

The mouse automation uses `enigo` 0.6.1 (latest), which wraps Windows `SendInput` under the hood. Since `enigo::Enigo` is synchronous (not Send), all enigo calls must be wrapped in `tokio::task::spawn_blocking()` so they don't block Tauri's async runtime. Progress events flow from the executor to the frontend via `AppHandle::emit()` using the `tauri::Emitter` trait.

The highest-risk item is the exact 1080p pixel coordinates for skill tree and paragon board UI elements — these require empirical measurement with the game running. Placeholder coordinates are acceptable for initial implementation and will be refined during Phase 5 integration testing. All coordinate math itself (scaling, jitter) is pure and fully testable.

**Primary recommendation:** Build the executor loop first with placeholder coords and full safety/event plumbing, then refine coords empirically. Keep coordinate logic in pure functions so APPLY-07 tests pass on all platforms without Win32.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| enigo | 0.6.1 | Mouse move + click simulation | Wraps Windows SendInput; cross-platform; recommended in STACK.md; latest stable |
| tauri (Emitter trait) | 2.x | Emit progress events to frontend | AppHandle::emit() is the established project pattern |
| tokio | 1.x | Async runtime, spawn_blocking | Already in Cargo.toml; enigo is sync — must use spawn_blocking |
| rand | 0.8.x | Coordinate jitter + delay randomization | Standard Rust randomization crate; no_std compatible |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| thiserror | 2.0.x | ApplyError enum | Already in project; one error type per module |
| std::sync::atomic::AtomicBool | stdlib | cancel_flag check | Already wired; checked before each click |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| enigo 0.6.1 | windows-rs SendInput directly | enigo wraps SendInput cleanly; direct SendInput requires unsafe Win32 blocks and is more code for no benefit |
| tokio::task::spawn_blocking | std::thread::spawn | spawn_blocking integrates with tokio scheduler; std thread would orphan and not propagate cancellation |

**Installation — add to Cargo.toml:**
```toml
enigo = "0.6"
rand = "0.8"
```

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/src/auto_applier/
├── mod.rs          # pub use executor::run; pub use error::ApplyError
├── coords.rs       # SkillTreeCoords, ParagonBoardCoords, scale_coord() pure functions
├── humanize.rs     # jitter_coord(), random_delay_ms() using rand crate
├── executor.rs     # run(), pause, resume — main loop, safety gate, event emission
└── error.rs        # ApplyError enum via thiserror
```

### Pattern 1: Pure Coordinate Scaling (enables APPLY-07 unit tests)
**What:** Store 1080p reference coords as constants; scale by resolution ratio in a pure function with no Win32 dependency.
**When to use:** Every coordinate before passing to enigo.
**Example:**
```rust
// Source: coords.rs pattern matching Resolution enum from types.rs
pub fn scale_factor(res: &Resolution) -> f64 {
    match res {
        Resolution::Res1080p => 1.0,
        Resolution::Res1440p => 1440.0 / 1080.0,  // 1.3333...
        Resolution::Res4K => 2160.0 / 1080.0,      // 2.0
    }
}

pub fn scale_coord(x: u32, y: u32, res: &Resolution) -> (u32, u32) {
    let f = scale_factor(res);
    ((x as f64 * f).round() as u32, (y as f64 * f).round() as u32)
}
```
This is fully testable: `scale_coord(960, 540, &Resolution::Res4K)` == `(1920, 1080)`.

### Pattern 2: Enigo Wrapped in spawn_blocking
**What:** enigo::Enigo is not Send. Instantiate it inside spawn_blocking on each click.
**When to use:** Every simulated click in the executor loop.
**Example:**
```rust
// Source: enigo 0.6.1 docs + Tauri spawn_blocking pattern
use enigo::{Button, Coordinate, Direction::Click, Enigo, Mouse, Settings};

pub async fn click_at(x: u32, y: u32) -> Result<(), ApplyError> {
    tokio::task::spawn_blocking(move || {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        enigo.move_mouse(x as i32, y as i32, Coordinate::Abs)
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        enigo.button(Button::Left, Click)
            .map_err(|e| ApplyError::InputFailed(e.to_string()))?;
        Ok::<_, ApplyError>(())
    })
    .await
    .map_err(|e| ApplyError::TaskPanic(e.to_string()))?
}
```

### Pattern 3: Executor Loop with Safety Gate
**What:** Per-step screenshot + assert_safe_state + cancel_flag check before each enigo call.
**When to use:** The core executor loop — non-negotiable per SAFE-03.
**Example:**
```rust
// Source: existing safety::assert_safe_state() API + Architecture Pattern 2
pub async fn run(
    plan: BuildPlan,
    app: tauri::AppHandle,
    state: Arc<Mutex<AppState>>,
) -> Result<(), ApplyError> {
    use tauri::Emitter;

    let steps = build_step_sequence(&plan, &state)?;
    let total = steps.len();

    for (i, step) in steps.iter().enumerate() {
        // Safety re-check before EVERY click (SAFE-03)
        let (hwnd, width, height, cancel_flag) = acquire_capture_params(&state)?;
        let pixels = game_capture::screenshot::capture_window(hwnd, width, height)
            .map_err(|e| ApplyError::CaptureFailed(e.to_string()))?;
        safety::assert_safe_state(&pixels, width, height, &cancel_flag)
            .map_err(|e| {
                let _ = app.emit("safety_event", SafetyEvent::AutomationAborted {
                    reason: e.to_string(),
                });
                ApplyError::SafetyFailure(e.to_string())
            })?;

        // Apply jitter and click
        let (jx, jy) = humanize::jitter_coord(step.x, step.y);
        click_at(jx, jy).await?;

        // Emit progress
        let _ = app.emit("apply_progress", ApplyPhase::Running { step: i + 1, total });

        // Human delay
        tokio::time::sleep(Duration::from_millis(humanize::random_delay_ms())).await;
    }

    let _ = app.emit("apply_complete", ApplyPhase::Complete);
    Ok(())
}
```

### Pattern 4: Tauri Command Delegation (thin command pattern)
**What:** `start_apply`, `pause_apply`, `resume_apply` are one-liners in lib.rs. All logic in auto_applier.
**When to use:** Always — established project pattern from Phase 2.
**Example:**
```rust
// In lib.rs invoke_handler — new commands to register
#[tauri::command]
async fn start_apply(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    auto_applier::executor::start(app, state.inner().clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn pause_apply(state: tauri::State<'_, Mutex<AppState>>) {
    auto_applier::executor::pause(state.inner());
}

#[tauri::command]
fn resume_apply(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) {
    auto_applier::executor::resume(app, state.inner().clone());
}
```

### Anti-Patterns to Avoid
- **Calling enigo directly in async context:** enigo::Enigo is not Send. Always use spawn_blocking.
- **Single safety check at start only:** Re-check before every click — mid-apply state changes are real.
- **Blocking sleep in async:** Use `tokio::time::sleep()`, not `std::thread::sleep()`.
- **PostMessage for clicks:** Use enigo (SendInput path), not PostMessage — PostMessage bypasses system input queue and is more suspicious to anti-cheat.
- **Hardcoded resolution pixels:** Always scale from 1080p reference via scale_coord().
- **Business logic in Tauri command handlers:** Executor logic lives in auto_applier module, commands are one-liners.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mouse click simulation | Custom Win32 SendInput wrapper | enigo 0.6.1 | Handles SendInput, input injection, platform differences correctly |
| Coordinate randomization | Custom LCG or time-based jitter | rand 0.8 crate | rand handles seeding, distribution, thread safety correctly |
| Async delay | spin loop or std::thread::sleep | tokio::time::sleep | Non-blocking; doesn't stall Tauri runtime |
| Event emission to frontend | Custom IPC | AppHandle::emit() + Emitter trait | Already the project pattern; handles serialization |

**Key insight:** The only truly custom code in this phase is coordinate tables (empirical) and the sequencer that maps BuildPlan fields to click steps. Everything else uses established crates.

## Common Pitfalls

### Pitfall 1: enigo Not Send
**What goes wrong:** `Enigo::new()` is called outside spawn_blocking; compiler error "cannot be sent between threads safely" or runtime panic.
**Why it happens:** enigo::Enigo holds a non-Send internal handle on Windows.
**How to avoid:** Always instantiate Enigo inside `tokio::task::spawn_blocking(move || { ... })`.
**Warning signs:** "Enigo doesn't implement Send" compile error in async context.

### Pitfall 2: State Lock Held During Win32 Calls
**What goes wrong:** AppState Mutex lock held while calling capture_window() or enigo click. Other commands deadlock waiting for the lock.
**Why it happens:** Lock acquired in command handler, passed into executor which takes a long time.
**How to avoid:** Extract needed values (hwnd, resolution, cancel_flag, build_plan) from AppState under a brief lock, then drop lock before all Win32 calls. Established in existing check_safety command.
**Warning signs:** Frontend becomes unresponsive during apply; get_game_state command hangs.

### Pitfall 3: Pause/Resume State Race
**What goes wrong:** pause_apply sets cancel_flag but executor already past the cancel check; resumes from wrong step.
**Why it happens:** AtomicBool is set by one thread, checked by another — timing gap between set and check.
**How to avoid:** Pause captures current step index into `ApplyPhase::Paused { step, total }` atomically under a brief lock when the flag is observed. Resume reads step from ApplyPhase::Paused and restarts sequence from that index.
**Warning signs:** Automation continues past pause; resumes from step 0 instead of saved step.

### Pitfall 4: Resolution Scale Factor Floating-Point Drift
**What goes wrong:** Accumulated rounding errors in scaled coordinates cause clicks to miss by several pixels at high resolutions.
**Why it happens:** Multiplying integer coords by f64 ratio and rounding to u32 independently per coordinate.
**How to avoid:** Round to nearest integer, not truncate. The +-2-5px jitter subsumes small rounding errors. Test scale_coord at boundary values.
**Warning signs:** APPLY-07 tests show off-by-one errors at 4K.

### Pitfall 5: Game Window Not Foreground During Click
**What goes wrong:** enigo clicks go to the wrong window (the tool's own UI).
**Why it happens:** Diablo IV window not in foreground when SendInput fires.
**How to avoid:** Use Win32 `SetForegroundWindow(hwnd)` before the click sequence begins. Optionally verify foreground before each step. Wrap in `cfg(windows)` guard.
**Warning signs:** Clicks affect the tool UI, not the game.

### Pitfall 6: Cargo.toml Missing enigo and rand
**What goes wrong:** Build fails with "unresolved import" for enigo or rand.
**Why it happens:** enigo and rand are not yet in Cargo.toml (only a stub auto_applier/mod.rs exists).
**How to avoid:** Wave 0 task must add `enigo = "0.6"` and `rand = "0.8"` before any executor code.
**Warning signs:** `cargo check` fails immediately on any auto_applier import.

## Code Examples

Verified patterns from official sources:

### enigo 0.6.1: Click at Absolute Coordinates
```rust
// Source: https://docs.rs/enigo/0.6.0/enigo/
use enigo::{Button, Coordinate, Direction::Click, Enigo, Mouse, Settings};

let mut enigo = Enigo::new(&Settings::default()).unwrap();
enigo.move_mouse(500, 200, Coordinate::Abs).unwrap();
enigo.button(Button::Left, Click).unwrap();
```

### Tauri v2: Emit Event from Background Task
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/
use tauri::{AppHandle, Emitter};

// app_handle cloned before spawning
let app_clone = app.clone();
tokio::task::spawn(async move {
    app_clone.emit("apply_progress", payload).unwrap();
});
```

### Coordinate Scaling Pure Function
```rust
// Pattern derived from Resolution enum in types.rs
use crate::types::Resolution;

pub fn scale_coord(x: u32, y: u32, res: &Resolution) -> (u32, u32) {
    let f: f64 = match res {
        Resolution::Res1080p => 1.0,
        Resolution::Res1440p => 2560.0 / 1920.0,
        Resolution::Res4K => 3840.0 / 1920.0,
    };
    ((x as f64 * f).round() as u32, (y as f64 * f).round() as u32)
}
```

### Humanization: Jitter + Delay
```rust
// Pattern: rand 0.8 with thread_rng
use rand::Rng;

pub fn jitter_coord(x: u32, y: u32) -> (u32, u32) {
    let mut rng = rand::thread_rng();
    let jx: i32 = rng.gen_range(-5..=5);
    let jy: i32 = rng.gen_range(-5..=5);
    ((x as i32 + jx).max(0) as u32, (y as i32 + jy).max(0) as u32)
}

pub fn random_delay_ms() -> u64 {
    rand::thread_rng().gen_range(50..=200)
}
```

### Safety Gate Integration (existing API)
```rust
// Source: src-tauri/src/safety/mod.rs — confirmed API
// assert_safe_state(pixels: &[u8], width: u32, height: u32, cancel_flag: &Arc<AtomicBool>)
//   -> Result<SafetyState, SafetyError>
let result = safety::assert_safe_state(&pixels, width, height, &cancel_flag);
if let Err(e) = result {
    return Err(ApplyError::SafetyFailure(e.to_string()));
}
```

### Pause/Resume via cancel_flag
```rust
// cancel_flag: Arc<AtomicBool> from AppState
// Pause: set flag, update apply_phase
pub fn pause(state: &Mutex<AppState>) {
    let mut s = state.lock().unwrap();
    s.cancel_flag.store(true, Ordering::SeqCst);
    if let ApplyPhase::Running { step, total } = s.apply_phase {
        s.apply_phase = ApplyPhase::Paused { step, total };
    }
}

// Resume: clear flag, re-spawn executor from saved step
pub fn resume(app: AppHandle, state: Arc<Mutex<AppState>>) {
    {
        let mut s = state.lock().unwrap();
        s.cancel_flag.store(false, Ordering::SeqCst);
    }
    tokio::task::spawn(run_from_saved_step(app, state));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| enigo 0.5.x (older API) | enigo 0.6.1 with Direction enum (Press/Release/Click) | 2024 | Direction::Click replaces separate press+release calls |
| Tauri v1 emit_all() | Tauri v2 emit() via Emitter trait | Oct 2024 | emit() replaces emit_all() in v2 |
| AppHandle not Clone in v1 | AppHandle: Clone in v2 | Oct 2024 | Can clone AppHandle before spawning tasks |

**Deprecated/outdated:**
- `enigo.mouse_click()`: Replaced by `enigo.button(Button::Left, Click)` in 0.6.x
- `app.emit_all()`: Tauri v2 uses `app.emit()` from the `Emitter` trait — import `tauri::Emitter`

## Open Questions

1. **Exact 1080p UI coordinates for skill tree and paragon board**
   - What we know: UI is at fixed positions relative to screen resolution; scales linearly
   - What's unclear: Actual pixel positions — require the game running at 1080p to measure
   - Recommendation: Use placeholder coordinates (center-of-screen) in Wave 1; flag for empirical measurement in a dedicated coordinates-validation task

2. **SetForegroundWindow requirement**
   - What we know: enigo SendInput fires globally; game must be foreground window
   - What's unclear: Whether enigo handles focus internally on Windows or caller must bring window to foreground
   - Recommendation: Add explicit `SetForegroundWindow(hwnd)` call at start of executor run; wrap in cfg(windows)

3. **Paragon board node-to-coordinate mapping**
   - What we know: `ParagonBoard.nodes: Vec<String>` contains node identifiers; boards have rotate field
   - What's unclear: Whether node identifiers correspond to grid positions that can be mapped to pixel coords, or require visual recognition
   - Recommendation: Start with sequential grid-position assumption; document as known gap requiring game-running validation

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `cargo test` |
| Config file | None — inline test modules |
| Quick run command | `cargo test -p diablo4-tool --lib auto_applier` |
| Full suite command | `cargo test -p diablo4-tool --lib` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| APPLY-01 | Skills applied in correct order | unit | `cargo test -p diablo4-tool --lib auto_applier::executor::tests` | ❌ Wave 0 |
| APPLY-02 | Paragon steps generated for all boards | unit | `cargo test -p diablo4-tool --lib auto_applier::executor::tests` | ❌ Wave 0 |
| APPLY-03 | scale_coord correct for 1080p/1440p/4K | unit | `cargo test -p diablo4-tool --lib auto_applier::coords::tests` | ❌ Wave 0 |
| APPLY-04 | Jitter stays within +-5px bounds | unit | `cargo test -p diablo4-tool --lib auto_applier::humanize::tests` | ❌ Wave 0 |
| APPLY-05 | Progress events: step count matches plan size | unit (mock emit) | `cargo test -p diablo4-tool --lib auto_applier::executor::tests` | ❌ Wave 0 |
| APPLY-06 | Pause sets Paused state; resume restores step | unit | `cargo test -p diablo4-tool --lib auto_applier::executor::tests` | ❌ Wave 0 |
| APPLY-07 | scale_coord at 1080p/1440p/4K specific values | unit | `cargo test -p diablo4-tool --lib auto_applier::coords::tests` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p diablo4-tool --lib auto_applier`
- **Per wave merge:** `cargo test -p diablo4-tool --lib`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/auto_applier/coords.rs` — covers APPLY-03, APPLY-07
- [ ] `src-tauri/src/auto_applier/humanize.rs` — covers APPLY-04
- [ ] `src-tauri/src/auto_applier/error.rs` — ApplyError enum
- [ ] `src-tauri/src/auto_applier/executor.rs` — covers APPLY-01, APPLY-02, APPLY-05, APPLY-06
- [ ] Add to Cargo.toml: `enigo = "0.6"` and `rand = "0.8"`

## Sources

### Primary (HIGH confidence)
- https://docs.rs/enigo/0.6.0/enigo/ — enigo 0.6.1 Mouse trait, Button enum, Direction enum, Enigo::new() signature
- https://v2.tauri.app/develop/calling-frontend/ — AppHandle::emit(), Emitter trait, Clone+Serialize payload requirement
- `src-tauri/src/types.rs` — BuildPlan, Variant, ParagonBoard, ApplyPhase, AppState, Resolution confirmed as-built
- `src-tauri/src/safety/mod.rs` — assert_safe_state() exact signature confirmed
- `src-tauri/src/game_capture/screenshot.rs` — capture_window() exact signature confirmed
- `src-tauri/Cargo.toml` — confirmed enigo and rand are NOT yet in dependencies

### Secondary (MEDIUM confidence)
- https://github.com/enigo-rs/enigo — enigo GitHub; confirmed 0.6.1 is latest
- `.planning/research/STACK.md` — enigo recommendation, spawn_blocking pattern, SendInput vs PostMessage analysis
- `.planning/research/ARCHITECTURE.md` — AppHandle event streaming pattern, anti-patterns validated

### Tertiary (LOW confidence)
- Paragon board node-to-coordinate mapping approach — unverified, requires game-running measurement

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — enigo 0.6.1 confirmed via docs.rs; rand 0.8 is de-facto standard; spawn_blocking pattern confirmed via Tauri docs
- Architecture: HIGH — all existing API shapes confirmed by reading actual source files; patterns match established Phase 2/3 conventions
- Coordinate values: LOW — 1080p reference coordinates for skill tree and paragon board UI require empirical measurement; placeholder approach is the safe path
- Pitfalls: HIGH — enigo Send limitation, Mutex lock anti-pattern, and tokio sleep requirement are all verified technical facts

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (enigo, Tauri stable; coordinate data requires game access regardless of date)
