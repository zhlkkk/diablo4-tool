# Phase 2: Scaffold + Safety + Game Capture - Research

**Researched:** 2026-03-16
**Domain:** Tauri v2 scaffold, Win32 window capture, DPI normalization, safety state machine, global hotkeys
**Confidence:** HIGH (Tauri scaffold, Win32 APIs), MEDIUM (pixel sampling approach, exclusive fullscreen detection)

## Summary

Phase 2 establishes the greenfield Tauri v2 project with three functional modules: shared types (`types.rs`), a safety module that gates automation on game-UI-state detection via pixel sampling, and a game capture module that finds the Diablo IV window, detects resolution, handles DPI, and captures screenshots. The project uses a single crate with module folders (not a Cargo workspace).

Key research findings: (1) Tauri has an official `tauri-plugin-global-shortcut` that handles system-wide hotkeys natively -- this is preferable to `rdev` since it integrates with Tauri's event system and avoids spawning a separate blocking listener thread. (2) For screenshot capture, `PrintWindow` with `PW_RENDERFULLCONTENT` is the right choice for single-frame pixel sampling -- it avoids the yellow border issue of Windows Graphics Capture API and works with borderless windowed DirectX apps. (3) Tauri apps on Windows are already DPI-aware, but we still need `GetDpiForWindow` to normalize coordinates for the game window specifically. (4) Exclusive fullscreen detection uses window style heuristics (`WS_POPUP` without `WS_THICKFRAME`) combined with window rect vs monitor rect comparison.

**Primary recommendation:** Use `tauri-plugin-global-shortcut` for F10 hotkey (not `rdev`), `PrintWindow` for screenshots (not `windows-capture` crate), and the `windows` crate 0.61 for all Win32 API calls.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Pixel sampling at known screen coordinates to detect skill tree / paragon board screens
- Re-check game state before every click action (maximum safety, ~1-5ms overhead per check)
- On safety failure: immediate halt + emit safety_abort event to GUI + log reason. User must manually restart when game is back in safe state
- Safety events displayed in GUI status area only (no log file)
- Global hotkey: F10 (system-wide, works even when D4 has focus)
- No GUI-only stop button needed at this phase (GUI is Phase 5)
- After emergency stop, track progress so user can resume from where it stopped
- Find D4 window via Win32 FindWindowW using known window class ("D3 Main Window Class"), fallback to EnumWindows + title match
- Support 1080p, 1440p, and 4K resolutions at launch with calibrated coordinate maps
- Detect exclusive fullscreen and warn user to switch to borderless windowed; block automation in exclusive fullscreen
- DPI normalization via GetDpiForWindow (Per-Monitor DPI Aware v2 manifest)
- Tauri v2 (current stable)
- Single crate with module folders: mod web_parser, mod game_capture, mod auto_applier, mod safety
- React frontend for the webview (Tauri official React template)
- Shared types module (types.rs or models.rs) at crate root for BuildPlan, GameState, AppState

### Claude's Discretion
- Screenshot capture API choice (BitBlt/PrintWindow vs Windows Graphics Capture API) -- pick based on compatibility and simplicity
- Exact pixel coordinates for safe-state detection calibration points
- React project setup details (Vite config, folder structure)
- Internal module file organization within each mod folder

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SAFE-01 | Safety module detects whether Diablo IV is in a safe UI state for automation (skill tree screen, paragon board screen) | Pixel sampling via PrintWindow screenshot + coordinate lookup tables per resolution |
| SAFE-02 | Safety module refuses to start automation if game is not in a recognized safe state | SafetyState enum with gate function returning Result; checked before executor starts |
| SAFE-03 | Safety module re-checks game state before each click step | `assert_safe_state()` call in executor loop; ~1-5ms overhead per PrintWindow capture |
| SAFE-04 | Safety module provides immediate emergency stop (hotkey) that halts all automation | `tauri-plugin-global-shortcut` with F10 key; sets `AtomicBool` cancel flag |
| SAFE-05 | Safety module logs all automation decisions for user transparency | Tauri event emission (`safety_check`, `safety_abort`, `emergency_stop`) to frontend |
| SAFE-06 | Unit tests verify safety module correctly identifies safe vs unsafe states | Test with pre-captured screenshot fixtures; mock pixel data for each resolution |
| CAPT-01 | App detects whether Diablo IV process is running and finds the game window handle | `FindWindowW` with class "D3 Main Window Class", fallback `EnumWindows` + title match |
| CAPT-02 | App detects the current game resolution from the window | `GetClientRect` on game HWND returns client area dimensions |
| CAPT-03 | App handles DPI scaling correctly | `GetDpiForWindow` on game HWND; normalize coordinates: `physical_px = logical_px * dpi / 96` |
| CAPT-04 | App detects if game is in exclusive fullscreen and warns user | Window style check (`WS_POPUP` without `WS_THICKFRAME`) + window rect covers entire monitor |
| CAPT-05 | App can capture a screenshot of the game window for state detection | `PrintWindow` with `PW_RENDERFULLCONTENT` flag; returns BGRA bitmap buffer |
| CAPT-06 | Unit tests verify resolution detection and DPI normalization logic | Pure functions for coordinate math tested with synthetic inputs; no HWND needed |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.10+ | Desktop app shell, IPC, window management | Official stable v2; Rust backend with web UI |
| tauri-plugin-global-shortcut | 2.3+ | System-wide F10 hotkey for emergency stop | Official Tauri plugin; uses OS-native RegisterHotKey; works when app lacks focus |
| windows | 0.61.x | Win32 API bindings (FindWindowW, GetClientRect, GetDpiForWindow, PrintWindow, etc.) | Microsoft's official Rust crate; matches Tauri 2.10 internal dependency |
| serde | 1.0 | Serialization for shared types | De facto standard for Rust serialization |
| serde_json | 1.0 | JSON for Tauri IPC | Required for Tauri command return types |
| tokio | 1.x | Async runtime | Tauri's built-in async runtime |
| thiserror | 2.0 | Typed error enums | Ergonomic error types per module |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| image | 0.25 | Decode/manipulate screenshot bitmaps for pixel sampling | Converting raw BGRA bitmap from PrintWindow to indexable pixel buffer |
| tauri-build | 2.x | Build script for Tauri (code generation, manifest embedding) | Required in build-dependencies |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tauri-plugin-global-shortcut` | `rdev` crate | rdev requires a dedicated blocking thread (`std::thread::spawn` with `rdev::listen`); harder to integrate with Tauri state; tauri-plugin-global-shortcut is native to the framework |
| `PrintWindow` (direct Win32) | `windows-capture` crate (WinRT Graphics Capture API) | windows-capture adds yellow border on Win10, requires continuous capture session setup for single frames, and is designed for video streaming not one-shot screenshots. PrintWindow is simpler for periodic single-frame capture |
| `PrintWindow` (direct Win32) | `win-screenshot` crate | win-screenshot wraps BitBlt/PrintWindow but adds unnecessary abstraction. Direct `windows` crate calls give us full control |
| `image` crate | Raw byte indexing | image crate handles BGRA->RGBA conversion and provides safe pixel access; raw indexing is error-prone with stride calculations |

**Installation (Cargo.toml):**
```toml
[dependencies]
tauri = { version = "2", features = ["devtools"] }
tauri-plugin-global-shortcut = "2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
thiserror = "2.0"
image = "0.25"

[dependencies.windows]
version = "0.61"
features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_HiDpi",
    "Win32_Graphics_Gdi",
]

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

**Frontend (package.json):**
```bash
npm create tauri-app@latest -- --template react-ts
# Then add:
npm install @tauri-apps/plugin-global-shortcut
```

## Architecture Patterns

### Recommended Project Structure
```
diablo4-tool/
├── package.json
├── index.html
├── vite.config.ts
├── tsconfig.json
├── src/                          # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   └── ...
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json          # global-shortcut permissions
    ├── build.rs
    └── src/
        ├── main.rs               # #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
        ├── lib.rs                 # Tauri builder, state init, command registration, plugin setup
        ├── types.rs               # BuildPlan, GameState, AppState, SafetyState, Resolution
        ├── safety/
        │   ├── mod.rs             # pub API: check_safe_state(), SafetyState enum
        │   ├── detector.rs        # Pixel sampling logic, coordinate maps per resolution
        │   └── hotkey.rs          # Emergency stop setup (integrates with tauri-plugin-global-shortcut)
        ├── game_capture/
        │   ├── mod.rs             # pub API: find_game_window(), capture_screenshot(), get_resolution()
        │   ├── window.rs          # FindWindowW, EnumWindows fallback, exclusive fullscreen check
        │   ├── screenshot.rs      # PrintWindow capture, BGRA bitmap handling
        │   └── dpi.rs             # GetDpiForWindow, coordinate normalization
        ├── web_parser/            # Stub mod.rs only -- implemented in Phase 3
        │   └── mod.rs
        └── auto_applier/          # Stub mod.rs only -- implemented in Phase 4
            └── mod.rs
```

### Pattern 1: Thin Command, Fat Module
**What:** Tauri `#[tauri::command]` functions are one-liners that delegate to module functions. All logic lives in plain Rust modules with no Tauri dependency in their public API.
**When to use:** Always. Keeps modules unit-testable without a Tauri runtime.
**Example:**
```rust
// In lib.rs — command handler
#[tauri::command]
fn get_game_state(state: tauri::State<'_, Mutex<AppState>>) -> Result<GameState, String> {
    let game_state = game_capture::detect_game_state().map_err(|e| e.to_string())?;
    let mut s = state.lock().unwrap();
    s.game_state = Some(game_state.clone());
    Ok(game_state)
}
```

### Pattern 2: Safety as a Pure Function on Screenshot Data
**What:** The safety check function takes a raw pixel buffer + resolution as input and returns a `SafetyState` enum. It has no Win32 dependency itself -- the capture module provides the pixels.
**When to use:** For all safe-state checks. Enables unit testing with fixture images.
**Example:**
```rust
// safety/detector.rs
pub fn detect_safe_state(pixels: &[u8], width: u32, height: u32) -> SafetyState {
    let resolution = Resolution::from_dimensions(width, height);
    let sample_points = get_sample_points(&resolution);
    for point in &sample_points {
        let pixel = get_pixel(pixels, width, point.x, point.y);
        if !point.matches(pixel) {
            return SafetyState::Unsafe { reason: format!("Pixel mismatch at ({}, {})", point.x, point.y) };
        }
    }
    SafetyState::Safe(DetectedScreen::SkillTree) // or ParagonBoard
}
```

### Pattern 3: Emergency Stop via AtomicBool + Tauri Global Shortcut
**What:** F10 hotkey sets an `Arc<AtomicBool>` cancel flag. The executor loop checks this flag before each click. The hotkey handler also emits a Tauri event to update the frontend.
**When to use:** For all cancel/emergency-stop flows.
**Example:**
```rust
// In lib.rs setup
use tauri_plugin_global_shortcut::{Code, Shortcut, ShortcutState, GlobalShortcutExt};

let cancel_flag = app_state.cancel_flag.clone();
let app_handle = app.handle().clone();
app.handle().plugin(
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |_app, shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                cancel_flag.store(true, Ordering::SeqCst);
                let _ = app_handle.emit("emergency_stop", ());
            }
        })
        .build(),
)?;
let f10 = Shortcut::new(None, Code::F10);
app.global_shortcut().register(f10)?;
```

### Anti-Patterns to Avoid
- **Business logic in command handlers:** Untestable without full Tauri runtime. Move all logic to module functions.
- **Blocking the async runtime:** Never call `std::thread::sleep()` or blocking Win32 calls inside async commands. Use `tokio::task::spawn_blocking()` for Win32 calls.
- **Single safety check at start only:** Game state can change mid-automation. Re-check before every click.
- **Hardcoded pixel coordinates:** Must use per-resolution lookup tables or proportional coordinates.
- **Using `windows-capture` crate for single-frame screenshots:** Overkill; designed for continuous video capture; adds yellow border on Win10.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Global system hotkeys | Custom `SetWindowsHookEx` / `RegisterHotKey` wrapper | `tauri-plugin-global-shortcut` | Handles platform differences, integrates with Tauri lifecycle, auto-unregisters on app exit |
| DPI detection | Manual registry reading or `GetSystemMetrics` | `GetDpiForWindow` via `windows` crate | Per-monitor DPI requires the per-window API, not system-level; `GetSystemMetrics` returns scaled values |
| Bitmap pixel access | Manual stride/offset calculation on raw bytes | `image` crate's `ImageBuffer` | Handles BGRA/RGBA conversion, bounds checking, stride alignment automatically |
| Tauri project scaffold | Manual file creation | `npm create tauri-app@latest -- --template react-ts` | Generates correct `tauri.conf.json`, `build.rs`, `main.rs`/`lib.rs` split, capabilities directory |
| Window enumeration fallback | Manual `EnumWindows` + callback | `windows` crate's `EnumWindows` with closure | Still manual, but the `windows` crate provides safe-ish wrappers for the callback pattern |

**Key insight:** The `windows` crate provides type-safe bindings for all needed Win32 APIs. There is no need for `winapi` (unmaintained) or `windows-sys` (raw unsafe only). The `windows` crate with feature flags is the standard approach.

## Common Pitfalls

### Pitfall 1: PrintWindow Returns Black/Empty Image
**What goes wrong:** `PrintWindow` returns all-black pixels for some windows, especially hardware-accelerated ones.
**Why it happens:** Without `PW_RENDERFULLCONTENT` flag (value `2`), PrintWindow uses the old rendering path that doesn't capture DirectX/OpenGL content.
**How to avoid:** Always pass `PW_RENDERFULLCONTENT` (0x00000002) as the `nFlags` parameter. This forces the window to render its full content into the provided DC.
**Warning signs:** All pixels are (0, 0, 0, 0) or (0, 0, 0, 255) in the captured buffer.

### Pitfall 2: DPI Mismatch Between App and Game Window
**What goes wrong:** Coordinates calculated for the game window are wrong because the app's DPI context differs from the game's.
**Why it happens:** Tauri's webview runs at the system's DPI, but the Diablo IV window may be on a different monitor with a different DPI scale factor.
**How to avoid:** Always call `GetDpiForWindow(game_hwnd)` to get the game window's actual DPI, not the app's DPI. Normalize ALL coordinates: `physical = logical * game_dpi / 96`.
**Warning signs:** Clicks land in wrong positions only on high-DPI displays.

### Pitfall 3: FindWindowW Fails After Game Restart
**What goes wrong:** The stored HWND becomes invalid after Diablo IV restarts.
**Why it happens:** Window handles are process-specific. When D4 restarts, the old HWND is stale.
**How to avoid:** Re-discover the HWND before each capture/check cycle using `FindWindowW`. Cache it in `GameState` but re-validate with `IsWindow()` before use. If invalid, re-discover.
**Warning signs:** Win32 calls return error codes or zero values with a stale HWND.

### Pitfall 4: Exclusive Fullscreen Detection False Positives
**What goes wrong:** Borderless windowed mode is misidentified as exclusive fullscreen.
**Why it happens:** Both modes produce a window that covers the entire monitor. Simple "window size == screen size" check is insufficient.
**How to avoid:** Check window style bits: exclusive fullscreen typically has `WS_POPUP` WITHOUT `WS_THICKFRAME` or `WS_CAPTION`. Borderless windowed has different style combinations. Additionally, Diablo IV on DX12 does not truly support exclusive fullscreen (DX12 fullscreen is always "borderless fullscreen optimized"), so the detection is mainly about warning users who selected "Fullscreen" in D4 settings vs "Windowed" or "Windowed (Fullscreen)".
**Warning signs:** Users report false warnings on borderless windowed mode.

### Pitfall 5: Pixel Sampling Coordinates Drift Between Game Patches
**What goes wrong:** Previously correct pixel sample points no longer detect the skill tree or paragon board screens.
**Why it happens:** Blizzard patches may move UI elements or change colors.
**How to avoid:** Use multiple redundant sample points per screen type (not just one). Sample distinctive, stable UI chrome (borders, panel backgrounds) rather than content areas that change. Store coordinates in a data structure that can be updated without code changes.
**Warning signs:** Safety module rejects valid safe states after a game patch.

### Pitfall 6: tauri-plugin-global-shortcut Not Added to Capabilities
**What goes wrong:** Hotkey registration fails silently or errors at runtime.
**Why it happens:** Tauri v2 requires explicit capability permissions for all plugins.
**How to avoid:** Add `"global-shortcut:allow-register"`, `"global-shortcut:allow-unregister"`, `"global-shortcut:allow-is-registered"` to `src-tauri/capabilities/default.json`.
**Warning signs:** `app.global_shortcut().register()` returns an error about missing permissions.

## Code Examples

### Finding the Diablo IV Window
```rust
// game_capture/window.rs
// Source: windows crate docs (microsoft.github.io/windows-docs-rs)
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::Win32::Foundation::HWND;

pub fn find_diablo_window() -> Result<HWND, CaptureError> {
    let class_name: Vec<u16> = "D3 Main Window Class\0".encode_utf16().collect();
    let hwnd = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) };
    if hwnd.0 == std::ptr::null_mut() {
        // Fallback: EnumWindows + title match
        return find_by_title("Diablo IV");
    }
    Ok(hwnd)
}
```

### Getting Resolution and DPI
```rust
// game_capture/dpi.rs
// Source: windows crate docs
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::Foundation::{HWND, RECT};

pub fn get_game_resolution(hwnd: HWND) -> Result<(u32, u32), CaptureError> {
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect)? };
    Ok(((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32))
}

pub fn get_game_dpi(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
}

/// Normalize a logical coordinate to physical pixels for the game window's DPI
pub fn normalize_coord(logical: u32, game_dpi: u32) -> u32 {
    (logical as f64 * game_dpi as f64 / 96.0).round() as u32
}
```

### Capturing a Screenshot with PrintWindow
```rust
// game_capture/screenshot.rs
// Source: Win32 GDI docs + windows crate bindings
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::PrintWindow;
use windows::Win32::Foundation::HWND;

const PW_RENDERFULLCONTENT: u32 = 0x00000002;

pub fn capture_window(hwnd: HWND, width: u32, height: u32) -> Result<Vec<u8>, CaptureError> {
    unsafe {
        let hdc_window = GetDC(hwnd);
        let hdc_mem = CreateCompatibleDC(hdc_window);
        let hbm = CreateCompatibleBitmap(hdc_window, width as i32, height as i32);
        let old_bm = SelectObject(hdc_mem, hbm);

        // PW_RENDERFULLCONTENT forces DX content rendering
        let success = PrintWindow(hwnd, hdc_mem, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT));

        // Read pixel data from bitmap
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let buf_size = (width * height * 4) as usize;
        let mut buffer = vec![0u8; buf_size];
        GetDIBits(hdc_mem, hbm, 0, height, Some(buffer.as_mut_ptr().cast()), &mut bmi, DIB_RGB_COLORS);

        // Cleanup
        SelectObject(hdc_mem, old_bm);
        DeleteObject(hbm);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_window);

        if success.as_bool() {
            Ok(buffer) // BGRA format
        } else {
            Err(CaptureError::PrintWindowFailed)
        }
    }
}
```

### Detecting Exclusive Fullscreen
```rust
// game_capture/window.rs
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::{HWND, RECT};

pub fn is_exclusive_fullscreen(hwnd: HWND) -> bool {
    unsafe {
        let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let has_popup = (style & WS_POPUP.0) != 0;
        let has_frame = (style & WS_THICKFRAME.0) != 0;
        let has_caption = (style & WS_CAPTION.0) != 0;

        if !has_popup || has_frame || has_caption {
            return false; // Windowed or borderless windowed
        }

        // Check if window covers entire monitor
        let mut win_rect = RECT::default();
        GetWindowRect(hwnd, &mut win_rect).ok();
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(monitor, &mut mi);

        win_rect.left == mi.rcMonitor.left
            && win_rect.top == mi.rcMonitor.top
            && win_rect.right == mi.rcMonitor.right
            && win_rect.bottom == mi.rcMonitor.bottom
    }
}
```

### Pixel Sampling for Safe-State Detection
```rust
// safety/detector.rs

/// Sample point definition: a pixel coordinate + expected color range
pub struct SamplePoint {
    pub x: u32,
    pub y: u32,
    pub expected_r: (u8, u8), // (min, max) range
    pub expected_g: (u8, u8),
    pub expected_b: (u8, u8),
}

impl SamplePoint {
    pub fn matches(&self, pixel: [u8; 4]) -> bool {
        // Buffer is BGRA format from PrintWindow
        let (b, g, r) = (pixel[0], pixel[1], pixel[2]);
        r >= self.expected_r.0 && r <= self.expected_r.1
            && g >= self.expected_g.0 && g <= self.expected_g.1
            && b >= self.expected_b.0 && b <= self.expected_b.1
    }
}

/// Get pixel at (x, y) from BGRA buffer with given width
pub fn get_pixel(buffer: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * width + x) * 4) as usize;
    [buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3]]
}
```

### Shared Types
```rust
// types.rs
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resolution {
    Res1080p, // 1920x1080
    Res1440p, // 2560x1440
    Res4K,    // 3840x2160
}

impl Resolution {
    pub fn from_dimensions(w: u32, h: u32) -> Option<Self> {
        match (w, h) {
            (1920, 1080) => Some(Self::Res1080p),
            (2560, 1440) => Some(Self::Res1440p),
            (3840, 2160) => Some(Self::Res4K),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyState {
    Safe(DetectedScreen),
    Unsafe { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectedScreen {
    SkillTree,
    ParagonBoard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub window_found: bool,
    pub resolution: Option<Resolution>,
    pub raw_width: u32,
    pub raw_height: u32,
    pub dpi: u32,
    pub is_exclusive_fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplyPhase {
    Idle,
    Running { step: usize, total: usize },
    Paused { step: usize, total: usize },
    Complete,
    Aborted { reason: String },
}

pub struct AppState {
    pub game_state: Option<GameState>,
    pub apply_phase: ApplyPhase,
    pub cancel_flag: Arc<AtomicBool>,
    // build_plan will be added in Phase 3
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `winapi` crate | `windows` crate 0.61+ | 2020-2021 | `winapi` is unmaintained; `windows` is Microsoft-official, code-generated from Windows metadata |
| BitBlt for screenshots | PrintWindow with PW_RENDERFULLCONTENT | Windows 8.1+ | BitBlt fails on hardware-accelerated windows; PrintWindow with the flag forces content rendering |
| System DPI (GetSystemMetrics) | Per-Monitor DPI (GetDpiForWindow) | Windows 10 1607+ | Correct DPI for multi-monitor setups where monitors have different scale factors |
| Custom hotkey hooks (SetWindowsHookEx) | tauri-plugin-global-shortcut | Tauri v2 (2024) | Framework-integrated, handles lifecycle, cross-platform if needed |
| `rdev` for global input | `tauri-plugin-global-shortcut` for hotkeys | Tauri v2 (2024) | rdev requires a blocking listener thread; Tauri plugin is native to the framework |

**Deprecated/outdated:**
- `winapi` crate: Unmaintained since ~2020; use `windows` crate instead
- `screenshot-rs`: Last commit 2017; produces black frames on modern Windows
- `windows-sys` alone: Raw unsafe bindings without safe wrappers; only use if minimizing dependency size is critical

## Open Questions

1. **Exact pixel coordinates for skill tree / paragon board detection**
   - What we know: Skill tree and paragon board screens have distinctive UI chrome (borders, backgrounds, specific colors)
   - What's unclear: The exact (x, y) coordinates and expected color values at each resolution -- these require empirical measurement from the running game
   - Recommendation: Phase 2 implementation should define the `SamplePoint` data structure and resolution lookup table, but use PLACEHOLDER coordinates. A calibration task (possibly Phase 2 Wave 0 or early implementation) requires running D4 at each resolution and recording pixel values. Mark sample points as `todo!()` with a clear comment.

2. **PrintWindow reliability with Diablo IV specifically**
   - What we know: PrintWindow with PW_RENDERFULLCONTENT works for most DX11/DX12 borderless windowed games
   - What's unclear: Whether Diablo IV's specific rendering pipeline (DX12, anti-cheat overlay) causes any issues
   - Recommendation: Implement PrintWindow as the primary capture method. If it returns black frames, add a fallback to Desktop Duplication API (DXGI). Test early.

3. **DX12 "exclusive fullscreen" vs "fullscreen optimized"**
   - What we know: DirectX 12 does not support true exclusive fullscreen. D4's "Fullscreen" setting is actually "borderless fullscreen optimized" under the hood.
   - What's unclear: Whether window style bits still differ between D4's "Fullscreen" and "Windowed (Fullscreen)" settings
   - Recommendation: Implement the window-style heuristic. If D4's "Fullscreen" mode is indistinguishable from borderless, the detection may need to check the D4 settings file or simply warn if the window style has `WS_POPUP` without standard frame decorations.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust test framework) |
| Config file | None needed -- uses `#[cfg(test)]` modules |
| Quick run command | `cd src-tauri && cargo test` |
| Full suite command | `cd src-tauri && cargo test -- --include-ignored` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SAFE-01 | Detect safe UI state from pixel data | unit | `cd src-tauri && cargo test safety::detector::tests -x` | Wave 0 |
| SAFE-02 | Refuse automation in unsafe state | unit | `cd src-tauri && cargo test safety::tests::test_gate_unsafe -x` | Wave 0 |
| SAFE-03 | Re-check before each click | unit | `cd src-tauri && cargo test safety::tests::test_recheck -x` | Wave 0 |
| SAFE-04 | F10 emergency stop halts automation | integration | Manual -- requires Tauri runtime + hotkey registration | Manual-only: requires OS-level hotkey hook |
| SAFE-05 | Log automation decisions as events | unit | `cd src-tauri && cargo test safety::tests::test_event_emission -x` | Wave 0 |
| SAFE-06 | Unit tests for safe/unsafe detection | unit | `cd src-tauri && cargo test safety::detector::tests -x` | Wave 0 |
| CAPT-01 | Find game window handle | unit+integration | `cd src-tauri && cargo test game_capture::window::tests -x` | Wave 0 |
| CAPT-02 | Detect game resolution | unit | `cd src-tauri && cargo test game_capture::dpi::tests::test_resolution -x` | Wave 0 |
| CAPT-03 | DPI normalization | unit | `cd src-tauri && cargo test game_capture::dpi::tests::test_normalize -x` | Wave 0 |
| CAPT-04 | Detect exclusive fullscreen | unit | `cd src-tauri && cargo test game_capture::window::tests::test_fullscreen -x` | Wave 0 |
| CAPT-05 | Screenshot capture | integration | Manual -- requires D4 running | Manual-only: requires game window |
| CAPT-06 | Resolution + DPI unit tests | unit | `cd src-tauri && cargo test game_capture::dpi::tests -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test`
- **Per wave merge:** `cd src-tauri && cargo test && cargo clippy -- -D warnings`
- **Phase gate:** Full suite green + `cargo build` succeeds

### Wave 0 Gaps
- [ ] `src-tauri/src/safety/detector.rs` -- test module with fixture-based pixel sampling tests (SAFE-01, SAFE-06)
- [ ] `src-tauri/src/safety/mod.rs` -- test module for gate function (SAFE-02, SAFE-03)
- [ ] `src-tauri/src/game_capture/window.rs` -- test module for window finding logic (CAPT-01, CAPT-04)
- [ ] `src-tauri/src/game_capture/dpi.rs` -- test module for resolution detection and DPI normalization (CAPT-02, CAPT-03, CAPT-06)
- [ ] `src-tauri/tests/fixtures/` -- screenshot fixture data (small byte arrays) for pixel sampling tests
- [ ] Tauri project scaffold itself -- `npm create tauri-app@latest` must run first

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Project Structure](https://v2.tauri.app/start/project-structure/) -- scaffold layout, main.rs/lib.rs split
- [Tauri v2 Create Project](https://v2.tauri.app/start/create-project/) -- `create-tauri-app` templates
- [Tauri v2 Configuration Reference](https://v2.tauri.app/reference/config/) -- tauri.conf.json schema
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) -- F10 hotkey registration, capabilities
- [windows crate FindWindowW](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.FindWindowW.html) -- API signature
- [windows crate GetClientRect](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.GetClientRect.html) -- API signature
- [windows crate GetDpiForWindow](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/HiDpi/fn.GetDpiForWindow.html) -- DPI detection API
- [Win32 Setting Default DPI Awareness](https://learn.microsoft.com/en-us/windows/win32/hidpi/setting-the-default-dpi-awareness-for-a-process) -- Per-Monitor V2 manifest
- [Win32 Capturing an Image](https://learn.microsoft.com/en-us/windows/win32/gdi/capturing-an-image) -- BitBlt/PrintWindow pattern

### Secondary (MEDIUM confidence)
- [rdev crate](https://docs.rs/rdev/) -- evaluated but NOT recommended (use tauri-plugin-global-shortcut instead)
- [windows-capture crate](https://crates.io/crates/windows-capture/1.0.17) -- evaluated but NOT recommended for this use case (yellow border, overkill for single-frame)
- [OBS Forum: BitBlt vs Graphics Capture](https://obsproject.com/forum/threads/for-capture-method-whats-the-difference-between-bitblt-and-windows-graphics-capture.127687/) -- capture method comparison
- [Application manifest for Per Monitor V2](https://gist.github.com/emoacht/7e5a026080aeb7eb1b9316f5fe7628da) -- manifest XML format

### Tertiary (LOW confidence)
- DX12 exclusive fullscreen behavior -- based on general DX12 documentation and forum discussions; D4-specific behavior needs empirical verification

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Tauri v2 official docs, windows crate official docs, well-established patterns
- Architecture: HIGH -- follows patterns from ARCHITECTURE.md research, validated against Tauri v2 project structure docs
- Pitfalls: MEDIUM -- Win32 capture pitfalls are well-documented; D4-specific behavior (pixel coordinates, DX12 rendering) needs empirical validation
- Safety module: MEDIUM -- architecture is sound but pixel coordinates require game-running calibration

**Research date:** 2026-03-16
**Valid until:** 2026-04-16 (stable domain; Tauri v2 is mature, Win32 APIs are frozen)
