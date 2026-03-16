# Phase 05: GUI + Integration - Research

**Researched:** 2026-03-16
**Domain:** React + Tauri v2 frontend integration, calibration tool, progress events
**Confidence:** HIGH

## Summary

Phase 5 connects all completed backend modules into a single end-to-end user flow. The frontend (`src/App.tsx`) already has the link input, build preview, and 5-state machine working. This phase extends that file with: a variant selector dropdown (conditional on variants.length > 1), apply controls (Start/Pause/Stop row), a real-time progress bar, bilingual error messages, and a calibration wizard. All Tauri backend commands already exist (`start_apply`, `pause_apply`, `resume_apply`); the frontend just needs to invoke them and listen to events.

The one new backend piece is calibration I/O: two Tauri commands (`save_calibration` and `load_calibration`) that read/write `calibration.json` in the Tauri `appDataDir`. This uses `std::fs` in Rust — no new plugin required. The calibration data overrides PLACEHOLDER constants in `coords.rs` at runtime.

The key architectural challenge is state management: the app must track `ApplyPhase` (Idle/Running/Paused/Complete/Aborted) locally in React via useState hooks, driven by Tauri event listeners (`apply_progress`, `safety_event`). Since the existing app uses no global state library and is a single React file, this is achievable with a small state object without extracting components or adding Redux.

**Primary recommendation:** Extend `App.tsx` with new state variables for `applyPhase`, `progress`, `calibrated`, and `selectedVariant`. Add Tauri event listeners in a `useEffect` with cleanup. Keep single-file architecture unless line count exceeds ~400 lines; extract a `CalibrationWizard` component only if the wizard logic becomes unwieldy.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Apply controls & progress display**
- Start/Pause/Stop buttons in a single row below the build preview card
- Start button is primary (gold accent), Pause and Stop are secondary (outlined)
- Progress bar showing automation progress (e.g., `应用中 3/15 技能点 / Applying skill 3 of 15`)
- Real-time status text updated per step event showing current action
- Listen to Tauri `apply_progress` and `safety_event` events via `@tauri-apps/api/event`
- Buttons disabled appropriately: Start disabled when no build loaded or during apply; Pause/Stop disabled when idle
- ApplyPhase state machine drives button visibility (Idle → Running → Paused → Complete/Aborted)

**Variant selector**
- Dropdown selector appears above the build preview ONLY when `variants.length > 1`
- Single-variant builds skip dropdown entirely (current Phase 3 behavior preserved)
- Selecting a different variant updates the preview card immediately
- Selected variant index passed to `start_apply` command

**Language & error display**
- Bilingual display: Chinese primary text + English subtitle for all alerts, errors, and status messages
- Pattern: `"链接无效 / Invalid link"`, `"游戏未找到 / Game not found"`, `"不安全状态 / Unsafe game state"`
- Error messages specific per failure type (GUI-05): game not found, invalid link, unsafe state, automation failed, emergency stop triggered
- Error banner inline below controls, not a modal — consistent with existing Phase 3 error display pattern
- Validation errors remain inline below input (existing pattern)

**Coordinate calibration tool**
- Calibration mode accessible via a settings/gear icon button
- Guided wizard flow: capture game screenshot → overlay screenshot in app → user clicks to mark UI positions → save to JSON config
- Steps: "Click the Skill Tree button" → "Click the first skill slot" → "Click the Paragon Board button" → etc.
- Coordinates saved to a `calibration.json` file in app data directory (Tauri `appDataDir`)
- On app start, check for `calibration.json` — if present, override PLACEHOLDER constants in coords.rs via Tauri command
- If no calibration exists, show warning: `"请先校准坐标 / Please calibrate coordinates first"` and disable Start button
- Calibration wizard captures coordinates at current game resolution, scale_factor applies for other resolutions

**Skill name display**
- Implement a simple lookup table (API key → Chinese name)
- If no mapping found, display raw API key as fallback (existing behavior)

### Claude's Discretion
- Exact progress bar CSS styling (should match dark theme + gold accent)
- Component file organization (single App.tsx vs extracted components)
- Calibration screenshot capture mechanism (Tauri screenshot command vs game_capture reuse)
- Exact calibration wizard step sequence and number of points to capture
- Whether to use React context or prop drilling for app state
- Tauri event listener setup and cleanup patterns

### Deferred Ideas (OUT OF SCOPE)
- Full i18n framework with language toggle (bilingual inline strings sufficient for v1)
- Auto-detect build link from clipboard (QOL-V2-01)
- Build comparison (current vs target) (QOL-V2-02)
- Seasonal build template library (QOL-V2-03)
- Skill ID → human-readable name mapping beyond simple lookup table
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GUI-01 | User sees a clean interface to paste d2core build link | Already exists in App.tsx — extend with calibration warning and Start disabled state |
| GUI-02 | User sees parsed build preview (skills + paragon) before applying | Already exists — extend with variant selector dropdown when variants.length > 1 |
| GUI-03 | User has start/stop/pause controls for the apply process | New: three-button row, ApplyPhase state machine in React, invoke start_apply/pause_apply/resume_apply |
| GUI-04 | User sees real-time status and progress during automation | New: progress bar + status text driven by `apply_progress` event listener |
| GUI-05 | User sees clear error messages for all failure states | New: bilingual error messages per failure type (game not found, bad link, unsafe state, automation failed, emergency stop) |
| GUI-06 | App window stays responsive during long-running automation | Tauri async commands already non-blocking; event listeners are already async; React state updates on events keep UI live |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.0.0 | UI components and state | Already installed, used in Phase 3 |
| @tauri-apps/api | ^2 | `invoke` and `listen` for Tauri bridge | Already installed |
| TypeScript | ~5.6.2 | Type safety | Already configured |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/api/event | (included in @tauri-apps/api ^2) | `listen()` for real-time events | Required for progress bar and safety events |
| std::fs (Rust stdlib) | N/A | Read/write calibration.json in Tauri appDataDir | Calibration I/O — no new Rust crate needed |
| serde_json | 1.0 (already in Cargo.toml) | Serialize/deserialize CalibrationData struct | Calibration file format |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::fs for calibration I/O | tauri-plugin-fs (JS-side file access) | Plugin-fs adds npm + Cargo dependency plus capability config; Rust-side is simpler and already the project pattern |
| useState for ApplyPhase | useReducer | useReducer cleaner for complex state transitions but adds boilerplate; useState sufficient for 5-state enum |
| Single App.tsx | Multiple component files | Splitting adds import complexity; single file fine up to ~400 lines |

**Installation:** No new packages needed. All dependencies are already in place.

---

## Architecture Patterns

### Existing App.tsx State (extend, don't replace)
The current state variables:
```typescript
const [url, setUrl] = useState("");
const [buildPlan, setBuildPlan] = useState<BuildPlan | null>(null);
const [loading, setLoading] = useState(false);
const [error, setError] = useState<string | null>(null);
const [validation, setValidation] = useState<string | null>(null);
```

New state variables to add:
```typescript
const [selectedVariant, setSelectedVariant] = useState(0);
const [applyPhase, setApplyPhase] = useState<ApplyPhaseState>("Idle");
const [progress, setProgress] = useState<ProgressInfo | null>(null);
const [applyError, setApplyError] = useState<string | null>(null);
const [calibrated, setCalibrated] = useState<boolean | null>(null); // null = unknown
const [showCalibration, setShowCalibration] = useState(false);
```

### Pattern 1: Tauri Event Listener with useEffect Cleanup
**What:** Subscribe to backend events on mount, unsubscribe on unmount.
**When to use:** `apply_progress`, `safety_event` — long-lived subscriptions driven by automation.
**Example:**
```typescript
// Source: @tauri-apps/api/event (Tauri v2 docs)
import { listen, UnlistenFn } from "@tauri-apps/api/event";

useEffect(() => {
  let unlistenProgress: UnlistenFn | undefined;
  let unlistenSafety: UnlistenFn | undefined;

  (async () => {
    unlistenProgress = await listen<ProgressPayload>("apply_progress", (event) => {
      setProgress({ step: event.payload.step, total: event.payload.total, label: event.payload.label });
      setApplyPhase("Running");
    });
    unlistenSafety = await listen<SafetyEventPayload>("safety_event", (event) => {
      if (event.payload.type === "AutomationAborted") {
        setApplyPhase("Aborted");
        setApplyError(event.payload.reason ?? "不安全状态 / Unsafe game state");
      } else if (event.payload.type === "EmergencyStop") {
        setApplyPhase("Aborted");
        setApplyError("紧急停止已触发 / Emergency stop triggered");
      } else if (event.payload.type === "AutomationStarted") {
        setApplyPhase("Running");
        setApplyError(null);
      }
    });
  })();

  return () => {
    unlistenProgress?.();
    unlistenSafety?.();
  };
}, []);
```

### Pattern 2: start_apply with Variant Index
**What:** The current `start_apply` in lib.rs always uses `plan.variants.first()`. Must accept `variant_index` parameter.
**When to use:** User selects a variant; index is passed to Tauri command.
```typescript
// Frontend invoke
await invoke("start_apply", { variantIndex: selectedVariant });
```
```rust
// Backend command signature (lib.rs must be updated)
async fn start_apply(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    variant_index: usize,
) -> Result<String, String>
```

### Pattern 3: Calibration I/O Tauri Commands (new Rust commands)
**What:** Two new Tauri commands handle read/write of `calibration.json` in `appDataDir`.
**When to use:** On app startup (load) and after calibration wizard completes (save).
```rust
// Rust side — uses tauri::Manager trait for app_data_dir()
use tauri::Manager;

#[tauri::command]
async fn load_calibration(app: tauri::AppHandle) -> Result<Option<CalibrationData>, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join("calibration.json");
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: CalibrationData = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    Ok(Some(data))
}

#[tauri::command]
async fn save_calibration(app: tauri::AppHandle, data: CalibrationData) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("calibration.json");
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}
```

### Pattern 4: Calibration Screenshot Capture (reuse game_capture)
**What:** The calibration wizard needs a screenshot of the game window for overlay. `game_capture::screenshot::capture_window` already does this.
**When to use:** When user opens calibration wizard and game is running.
```rust
// New Tauri command: capture screenshot and return as base64 PNG for display in React
#[cfg(windows)]
#[tauri::command]
fn capture_game_screenshot() -> Result<String, String> {
    let hwnd = game_capture::window::find_diablo_window().map_err(|e| e.to_string())?;
    let (width, height) = game_capture::dpi::get_game_resolution(hwnd).map_err(|e| e.to_string())?;
    let pixels = game_capture::screenshot::capture_window(hwnd, width, height)
        .map_err(|e| e.to_string())?;
    // Convert BGRA pixels to PNG bytes, then base64
    // Use `image` crate (already in Cargo.toml)
    use image::{ImageBuffer, Rgba};
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels)
        .ok_or_else(|| "Failed to create image buffer".to_string())?;
    let mut png_bytes: Vec<u8> = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&png_bytes))
}
```
Note: `base64` crate must be added to Cargo.toml (`base64 = "0.22"`). Alternatively, skip base64 and use a Tauri asset URL approach. Base64 data URI in an `<img>` src is simpler for v1.

### Pattern 5: ApplyPhase State Machine in React
**What:** Local React state mirrors backend ApplyPhase. Drives button enable/disable.
**Button states by phase:**

| Phase | Start | Pause | Stop |
|-------|-------|-------|------|
| Idle | Enabled (if build loaded + calibrated) | Disabled | Disabled |
| Running | Disabled | Enabled | Enabled |
| Paused | Enabled (resume label) | Disabled | Enabled |
| Complete | Enabled (re-apply) | Disabled | Disabled |
| Aborted | Enabled (retry) | Disabled | Disabled |

```typescript
type ApplyPhaseState = "Idle" | "Running" | "Paused" | "Complete" | "Aborted";
```

### Pattern 6: Progress Bar HTML/CSS
```typescript
{progress && (
  <div className="progress-container">
    <div
      className="progress-bar"
      style={{ width: `${(progress.step / progress.total) * 100}%` }}
    />
    <div className="progress-label">
      {`应用中 ${progress.step}/${progress.total} / Applying ${progress.step} of ${progress.total}`}
    </div>
    <div className="progress-step">{progress.label}</div>
  </div>
)}
```
```css
.progress-container {
  margin-top: 12px;
}
.progress-bar {
  height: 4px;
  background: #d4af37; /* Diablo gold */
  border-radius: 2px;
  transition: width 0.2s ease;
}
```

### Pattern 7: CalibrationData Struct
```rust
// types.rs — add this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub resolution_width: u32,
    pub resolution_height: u32,
    // Skill tree points
    pub skill_allocate_button: Point2D,
    pub skill_panel_origin: Point2D,
    pub skill_grid_spacing: u32,
    // Paragon board points
    pub paragon_center: Point2D,
    pub paragon_node_spacing: u32,
    pub paragon_nav_next: Point2D,
    pub paragon_nav_prev: Point2D,
}
```

### Anti-Patterns to Avoid
- **Spawning async tasks from button handlers without tracking state:** `invoke("start_apply")` returns a Promise; not awaiting it means the UI has no way to know when it finishes or fails. Always `.then()` / `catch()` in the handler.
- **Forgetting to unlisten on unmount:** If `listen()` subscriptions are not cleaned up, they accumulate across hot reloads in dev mode. The `useEffect` cleanup pattern above prevents this.
- **Blocking the Tauri async thread:** `start_apply` is already `async` in Rust and runs in Tokio. The UI thread stays live because Tauri's invoke bridge does not block.
- **Re-rendering on every progress event:** Progress events fire per click step (~100ms intervals). Using `setState` directly is fine for this frequency.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| File read/write for calibration.json | Custom file path logic | Tauri `app.path().app_data_dir()` + std::fs | appDataDir is OS-appropriate (AppData\Roaming on Windows), created automatically by Tauri |
| Screenshot overlay in calibration | Canvas-based pixel manipulation | `<img>` with base64 data URI + CSS `position: absolute` for click capture | Simple overlay with onClick capturing clientX/clientY is sufficient |
| Progress bar animation | JavaScript animation loop | CSS `transition: width` on the bar element | Browser handles smooth interpolation |
| Event subscription cleanup | Manual tracking arrays | `UnlistenFn` returned by `listen()`, called in useEffect cleanup | Tauri's API already handles this correctly |

---

## Common Pitfalls

### Pitfall 1: `start_apply` variant_index mismatch
**What goes wrong:** Backend `run()` in executor.rs always calls `plan.variants.first()`. If user selects variant index 1 but backend ignores it, wrong build is applied.
**Why it happens:** `start_apply` in lib.rs was written before variant selection was a requirement.
**How to avoid:** Update `start_apply` command to accept `variant_index: usize` parameter and pass it to `executor::run()`. Update `executor::run()` signature to accept variant index instead of always using `first()`.
**Warning signs:** Build preview shows variant 1 but automation applies variant 0 skills.

### Pitfall 2: Tauri `app.path()` vs `tauri::api::path`
**What goes wrong:** Tauri v2 changed the path API. `tauri::api::path::app_data_dir()` is Tauri v1. In Tauri v2 it is `app.path().app_data_dir()` (via `tauri::Manager` trait).
**Why it happens:** Many Stack Overflow answers reference v1 API.
**How to avoid:** Use `use tauri::Manager;` and call `app_handle.path().app_data_dir()`.
**Warning signs:** Compile error "no method named `app_data_dir` found".

### Pitfall 3: `listen()` in React called outside useEffect
**What goes wrong:** `listen()` called at component top level or inside render creates a new subscription every render cycle, causing memory leaks and duplicate event handling.
**Why it happens:** Developers new to async in React hooks.
**How to avoid:** Always wrap `listen()` calls in `useEffect(() => { ... return unlisten; }, [])` with empty dep array.
**Warning signs:** Events fire multiple times per step; progress bar jumps forward by >1 per event.

### Pitfall 4: Calibration screenshot BGRA vs RGBA byte order
**What goes wrong:** `game_capture::screenshot::capture_window` returns BGRA bytes (Windows BitBlt format), not RGBA. Creating an `image::ImageBuffer::<Rgba<u8>>` from BGRA bytes produces blue-tinted image.
**Why it happens:** Windows GDI DIB sections use BGRA channel order.
**How to avoid:** Swap B and R channels before creating the ImageBuffer, OR use `image::DynamicImage` with explicit channel mapping. Alternatively use `Bgra<u8>` if image crate supports it for the encoder path.
**Warning signs:** Screenshot overlay in calibration wizard looks blue/purple-tinted.

### Pitfall 5: Start button not disabled when calibration is missing
**What goes wrong:** User clicks Start before calibrating; `executor::run()` uses PLACEHOLDER coords and automation goes to wrong pixels.
**Why it happens:** Calibration check is a new app-startup concern that isn't part of the existing 5-state machine.
**How to avoid:** On app startup, `invoke("load_calibration")` and set `calibrated` state. Disable Start button if `calibrated === false`. Show `"请先校准坐标 / Please calibrate coordinates first"` warning.
**Warning signs:** Automation runs with placeholder coords, clicking in wrong positions.

### Pitfall 6: apply_complete event not listened to
**What goes wrong:** Automation finishes but UI stays in "Running" phase because only `apply_progress` events are listened to. User sees frozen progress bar.
**Why it happens:** Executor emits `apply_complete` as a separate event (not `apply_progress`).
**How to avoid:** Listen to `apply_complete` event in addition to `apply_progress`. On `apply_complete`, set `applyPhase` to `"Complete"` and clear progress.
**Warning signs:** Progress bar stays at last step count, buttons remain in Running state.

---

## Code Examples

### Variant Selector (conditional dropdown)
```typescript
// Source: React controlled select, Tauri variant index pattern
{buildPlan && buildPlan.variants.length > 1 && (
  <select
    className="variant-select"
    value={selectedVariant}
    onChange={(e) => setSelectedVariant(Number(e.target.value))}
  >
    {buildPlan.variants.map((v, i) => (
      <option key={i} value={i}>{v.name || `变体 ${i + 1} / Variant ${i + 1}`}</option>
    ))}
  </select>
)}
```

### Apply Controls Row
```typescript
// Source: CONTEXT.md locked decision
<div className="controls-row">
  <button
    className="btn-primary"
    onClick={handleStart}
    disabled={!buildPlan || !calibrated || applyPhase === "Running"}
  >
    {applyPhase === "Paused" ? "继续 / Resume" : "开始 / Start"}
  </button>
  <button
    className="btn-secondary"
    onClick={handlePause}
    disabled={applyPhase !== "Running"}
  >
    暂停 / Pause
  </button>
  <button
    className="btn-secondary"
    onClick={handleStop}
    disabled={applyPhase === "Idle" || applyPhase === "Complete"}
  >
    停止 / Stop
  </button>
</div>
```

### Bilingual Error Messages Map
```typescript
// Source: CONTEXT.md GUI-05 requirements
const ERROR_MESSAGES: Record<string, string> = {
  "No build plan loaded": "未加载构建 / No build loaded",
  "Game window not found": "游戏未找到 / Game not found",
  "unsafe state": "不安全状态 / Unsafe game state",
  "Emergency stop": "紧急停止已触发 / Emergency stop triggered",
  "Automation aborted": "自动化中止 / Automation failed",
};

function formatError(raw: string): string {
  for (const [key, msg] of Object.entries(ERROR_MESSAGES)) {
    if (raw.toLowerCase().includes(key.toLowerCase())) return msg;
  }
  return raw; // fallback: show raw error
}
```

### Skill Name Lookup Table
```typescript
// Source: CONTEXT.md — simple lookup, fallback to raw key
const SKILL_NAMES: Record<string, string> = {
  "Basic_Lunging_Strike": "冲刺打击",
  "Core_Whirlwind": "旋风斩",
  "Defensive_Iron_Skin": "铁甲",
  // ... add more as known
};

function displaySkillName(key: string): string {
  return SKILL_NAMES[key] ?? key;
}
```

### Calibration Load on Startup
```typescript
// Source: CONTEXT.md calibration requirement
useEffect(() => {
  invoke<CalibrationData | null>("load_calibration").then((data) => {
    setCalibrated(data !== null);
  }).catch(() => {
    setCalibrated(false);
  });
}, []);
```

---

## Backend Changes Required

This section is critical for planning — Phase 5 is not purely frontend.

### 1. Update `start_apply` to accept variant_index
**File:** `src-tauri/src/lib.rs`
**Change:** Add `variant_index: usize` parameter; pass to `executor::run()`.

### 2. Update `executor::run()` to use variant_index
**File:** `src-tauri/src/auto_applier/executor.rs`
**Change:** Accept `variant_index: usize`; replace `plan.variants.first()` with `plan.variants.get(variant_index)`.

### 3. Add CalibrationData type
**File:** `src-tauri/src/types.rs`
**Change:** Add `CalibrationData` struct (see Architecture Patterns above).

### 4. Add `load_calibration` Tauri command
**File:** `src-tauri/src/lib.rs`
**Change:** New `async fn load_calibration(app: AppHandle)` — read from appDataDir.

### 5. Add `save_calibration` Tauri command
**File:** `src-tauri/src/lib.rs`
**Change:** New `async fn save_calibration(app: AppHandle, data: CalibrationData)` — write to appDataDir.

### 6. Add `capture_game_screenshot` Tauri command (Windows only)
**File:** `src-tauri/src/lib.rs`
**Change:** New `fn capture_game_screenshot()` — reuse game_capture, return base64 PNG string.
**Note:** Requires `base64 = "0.22"` in Cargo.toml.

### 7. Register all new commands in `invoke_handler!`
**File:** `src-tauri/src/lib.rs`
**Change:** Add `load_calibration`, `save_calibration`, `capture_game_screenshot` to `tauri::generate_handler![]`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tauri::api::path::app_data_dir()` | `app.path().app_data_dir()` | Tauri v2 | Use Manager trait, not standalone function |
| `tauri::Event` re-exports | `@tauri-apps/api/event` `listen()` | Tauri v2 | Import from `@tauri-apps/api/event` not from `@tauri-apps/api` directly |
| `unlisten()` function-based | `listen()` returns `UnlistenFn` | Tauri v2 | Call the returned function to clean up |

**Deprecated/outdated:**
- `@tauri-apps/api/tauri`: replaced by `@tauri-apps/api/core` for `invoke` (Phase 3 already uses the v2 path)
- `tauri::api::path`: Tauri v1 — not used anywhere in this project

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (unit tests in-module) |
| Config file | `src-tauri/Cargo.toml` |
| Quick run command | `cd src-tauri && cargo test --lib 2>&1` |
| Full suite command | `cd src-tauri && cargo test 2>&1` |

Note: No frontend test framework is installed (`vitest`, `jest` are absent from `package.json`). Frontend logic is thin (state machines, event handling) — validated through TypeScript compilation (`tsc --noEmit`) and manual smoke test.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GUI-01 | Link input present, calibration warning shown when not calibrated | manual | `npm run build` (TypeScript compile) | ✅ src/App.tsx |
| GUI-02 | Build preview shows correct variant data | manual | `npm run build` | ✅ src/App.tsx |
| GUI-03 | Start/Pause/Stop invoke correct Tauri commands | manual + unit | `cargo test --lib executor` | ✅ executor.rs has tests |
| GUI-04 | Progress bar updates on apply_progress events | manual | `npm run build` | ✅ Wave 0 additions to App.tsx |
| GUI-05 | Error messages shown per failure type, bilingual | manual | `npm run build` | ❌ Wave 0: add error message tests |
| GUI-06 | UI stays responsive during automation | manual smoke | N/A — async Tauri inherently non-blocking | ✅ existing architecture |

Unit tests for new Rust commands:
- `load_calibration` / `save_calibration`: test with temp dir, verify round-trip JSON serialization
- `executor::run()` variant_index: test that `build_step_sequence` uses correct variant

### Sampling Rate
- **Per task commit:** `cd /home/zhlkkk/workspace/diablo4-tool/src-tauri && cargo test --lib 2>&1`
- **Per wave merge:** `cd /home/zhlkkk/workspace/diablo4-tool/src-tauri && cargo test 2>&1 && cd .. && npm run build 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/auto_applier/executor.rs` — add test for `build_step_sequence` using variant_index (not always variant 0)
- [ ] `src-tauri/src/lib.rs` — add unit test for `load_calibration` round-trip (temp dir pattern)
- [ ] TypeScript compiler check added to per-wave gate: `npm run build` ensures no type errors in new App.tsx state

---

## Open Questions

1. **Calibration screenshot BGRA channel order**
   - What we know: `capture_window` returns raw Windows BitBlt BGRA bytes; `image` crate `Rgba<u8>` is RGBA
   - What's unclear: Whether `image` crate v0.25 has a BGRA encoder or requires manual channel swap
   - Recommendation: Implementer should check `image::DynamicImage::from_bgra8()` or swap channels manually before encoding to PNG

2. **base64 dependency for screenshot**
   - What we know: Rust stdlib has no base64 encoder; `base64 = "0.22"` is the standard crate
   - What's unclear: Whether there is a lighter alternative already transitively available in the dependency tree
   - Recommendation: Add `base64 = "0.22"` to Cargo.toml; check `cargo tree` first to avoid version conflicts

3. **Calibration wizard click coordinate normalization**
   - What we know: Screenshot is captured at game resolution (e.g. 1920x1080); React renders it scaled to fit the app window (max 600px wide)
   - What's unclear: The scale factor between displayed screenshot pixels and actual game pixels must be applied when saving calibration coordinates
   - Recommendation: When user clicks on the overlay image, compute `actualX = clientX / displayedWidth * gameWidth` before storing in CalibrationData

---

## Sources

### Primary (HIGH confidence)
- `src/App.tsx` — direct inspection of current frontend (no uncertainty)
- `src-tauri/src/lib.rs` — direct inspection of all registered Tauri commands
- `src-tauri/src/types.rs` — direct inspection of ApplyPhase, BuildPlan, Variant types
- `src-tauri/src/auto_applier/executor.rs` — direct inspection of run(), pause(), resume()
- `src-tauri/src/auto_applier/coords.rs` — direct inspection of PLACEHOLDER constants
- `src-tauri/src/safety/mod.rs` — direct inspection of SafetyEvent enum
- `.planning/phases/05-gui-integration/05-CONTEXT.md` — locked user decisions
- `src-tauri/Cargo.toml` — dependency inventory (image, serde_json, reqwest, etc. all present)
- `src-tauri/capabilities/default.json` — current permissions (no fs plugin, confirms Rust-side I/O pattern)

### Secondary (MEDIUM confidence)
- Tauri v2 path API: `app.path().app_data_dir()` via `tauri::Manager` — based on training knowledge of Tauri v2 breaking changes from v1; project already uses other v2 patterns confirming version

### Tertiary (LOW confidence)
- `base64 = "0.22"` version: correct major version confirmed, patch may differ; check crates.io

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified by direct file inspection
- Architecture: HIGH — Tauri command patterns match existing code exactly; no new frameworks
- Pitfalls: HIGH — derived from direct code inspection of type mismatches and known API changes
- Calibration screenshot BGRA: MEDIUM — channel order is a known Windows GDI characteristic; image crate version behavior unconfirmed

**Research date:** 2026-03-16
**Valid until:** 2026-04-15 (stable stack; Tauri v2 API unlikely to change)
