# Pitfalls Research

**Domain:** Windows desktop game automation tool (Diablo IV, Rust + Tauri)
**Researched:** 2026-03-16
**Confidence:** MEDIUM — Core Warden behavior is well-documented via community; input injection flags verified via Win32 docs; Tauri pitfalls verified via official issues and docs. d2core.com encoding format is unverified (no public documentation found).

---

## Critical Pitfalls

### Pitfall 1: Diablo IV Is Always-Online — "Offline Mode" Does Not Exist

**What goes wrong:**
The PROJECT.md safety requirement states: "Safety module disables auto-apply mode if the game is detected as online." The implicit assumption is that an offline mode exists to automate into. It does not. Diablo IV is always-online by design; Blizzard confirmed there is no offline mode and there will not be one. Every session — including solo play and character select — is an authenticated online session routed through Battle.net servers.

**Why it happens:**
Developers familiar with Diablo II or Diablo III assume Diablo IV has a similar offline/offline-character option. Blizzard explicitly removed this in D4's design. The safety module as described cannot reliably distinguish "playing with others" from "solo private session" at the OS level; both require an active network connection to Blizzard's servers.

**How to avoid:**
Reframe what "safe to automate" means. The practical safety boundary is not online vs. offline but rather: is the character at the skill/paragon screen with no active game session in progress? Detection strategy must shift to: (1) checking that the game process is idle at a known-safe UI state (character screen, town, skill menu) rather than checking network connectivity, and (2) requiring the user to manually confirm they are in a safe state before applying. Do not rely on ping/network checks as the safety gate.

**Warning signs:**
- Project planning documents that describe "offline mode detection" as a feature
- Code that checks `ping battle.net` or monitors outbound connections as the safety predicate

**Phase to address:**
Safety module design phase (the phase that implements the online-detection safety gate). Must redefine the safety invariant before any automation code is written.

---

### Pitfall 2: SendInput Sets the LLMHF_INJECTED Flag — Warden Can Read It

**What goes wrong:**
The `enigo` crate (and any Win32 `SendInput`-based approach) injects synthetic mouse/keyboard events that Windows marks with the `LLMHF_INJECTED` flag at the low-level hook level (`WH_MOUSE_LL`, `WH_KEYBOARD_LL`). Anti-cheat software that installs such hooks — including Warden — can observe this flag on every injected event. An aggressive Warden policy can detect that synthetic clicks are occurring and flag or ban the account.

**Why it happens:**
Developers test on their own machines, observe no ban, and conclude the approach is safe. Warden enforcement is pattern-based and selective, so sporadic use often goes undetected initially. The risk accumulates with repeated automated sessions, especially if click timing is inhuman (perfectly uniform intervals, no micro-jitter, pixel-perfect positioning every time).

**How to avoid:**
- Add randomized sub-pixel jitter (1–3 px) to every click target coordinate
- Add randomized pre-click mouse movement paths (bezier curves, not straight lines)
- Add randomized timing delays between clicks (vary by ±10–30% of base delay)
- Apply the tool only at character select / skill assignment screens, not during combat or active gameplay
- Make the automation clearly operate on skill-assignment UI only, not game actions
- Avoid running the tool while the game's anti-cheat module is in its most active state (actively in a session/dungeon)

Kernel-level injection (`MouClassInputInjection`) is strictly off-limits — it is a direct ban trigger and violates ToS beyond ordinary automation.

**Warning signs:**
- Clicks use exact pixel coordinates with zero variation
- `thread::sleep(Duration::from_millis(500))` — exactly uniform timing throughout the sequence
- No mouse movement simulation between clicks (cursor teleports to target)

**Phase to address:**
Auto-applier module implementation phase. Jitter and humanization logic must be built in from the start, not retrofitted.

---

### Pitfall 3: DPI Scaling Causes All Click Coordinates to Be Wrong

**What goes wrong:**
Windows has two coordinate systems: physical pixels and logical (DPI-scaled) pixels. If the user runs Windows at 125%, 150%, or 200% display scaling (extremely common on modern laptops and 4K monitors), the captured window screenshot is in physical pixels but `SendInput` mouse coordinates must be in physical screen pixels too — however, window position queries via `GetWindowRect` return logical coordinates on DPI-unaware processes. The result: clicks land in the wrong location, shifted by exactly the DPI scale factor.

Additionally, if the Diablo IV game window is captured at one DPI context and click coordinates are computed in another, every coordinate is systematically wrong by the scaling ratio. This is a silent bug — the tool appears to work at 100% scale (the developer's machine) but fails for most real users.

**Why it happens:**
Rust and Win32 interop code must be explicitly declared DPI-aware via the application manifest (`SetProcessDpiAwareness` / `SetProcessDpiAwarenessContext`). Without this declaration, Windows virtualizes coordinates for the process. Tauri's WebView2 renderer has its own DPI context; code in the Rust backend that queries window positions may operate in a different DPI context than the code that captures or clicks.

**How to avoid:**
- Declare the process as Per-Monitor DPI Aware v2 in the application manifest (`DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`)
- Use `GetDpiForWindow` to query the target window's DPI and normalize all coordinates accordingly
- Test on at least 100%, 125%, and 150% Windows display scaling during development
- In the game capture module, verify that screenshot pixel dimensions match the window's actual physical pixel size before computing click targets

**Warning signs:**
- Clicks work perfectly on the developer's machine but land in the wrong location on testers' machines
- Coordinates are computed without any DPI lookup — raw `GetWindowRect` output fed directly into `SendInput`
- The application manifest does not declare `dpiAware` or `dpiAwareness`

**Phase to address:**
Game capture and auto-applier modules. DPI normalization must be implemented before any coordinate computation logic is written.

---

### Pitfall 4: The d2core.com Build Encoding Is Undocumented and Volatile

**What goes wrong:**
The entire tool depends on successfully reverse-engineering the `bd=` URL parameter from `d2core.com/d4/planner`. No official documentation for this encoding was found in public sources. If d2core.com changes their encoding format — which any web service can do at any time, with no notice — the web_parser module silently breaks. Users paste a valid link and the tool either produces garbage or crashes with a parse error.

**Why it happens:**
Third-party build planners are independently developed web services with no obligation to maintain stable encoding formats. Schema changes happen routinely when new seasons add skills, paragon nodes, or game mechanics.

**How to avoid:**
- Version-detect the encoding before attempting to decode: check for a version prefix or known structural signature in the `bd=` value
- Design the parser to return a typed error (`UnsupportedEncodingVersion`, `ParseFailed`) rather than panicking, with a user-visible message that explains the format may have changed
- Write integration tests that pin specific `bd=` values to expected decoded outputs — these tests will fail immediately when the site changes its format, giving early warning
- Monitor d2core.com changelogs or GitHub repository (if public) for encoding changes
- Consider displaying the raw decoded data to the user for verification before applying, so encoding drift is visible

**Warning signs:**
- Parser has no version check and assumes the encoding is immutable
- No pinned test vectors for the `bd=` decoder
- Silent fallback on parse error (e.g., applying a zeroed-out build without warning)

**Phase to address:**
Web parser module phase. Version-detection and error-typing must be part of the initial implementation, not an afterthought.

---

### Pitfall 5: Game Window Capture Fails When the Game Runs in Exclusive Fullscreen

**What goes wrong:**
When Diablo IV runs in exclusive fullscreen mode, `BitBlt`-based screen capture returns a black frame. The Windows Graphics Capture API (WGC, used by `windows-capture`) works for windowed and borderless-windowed modes but requires a capture session permission prompt when capturing certain protected windows. If the user's game is in exclusive fullscreen, every screenshot the tool takes is blank.

**Why it happens:**
Developers test in windowed or borderless-windowed mode (common for developers who alt-tab frequently). Many players run the game in exclusive fullscreen for performance. The capture API behavior differs silently between modes.

**How to avoid:**
- Use WGC (`windows-capture` crate) rather than `BitBlt` — WGC handles borderless windowed correctly
- At startup, detect the game's display mode and warn the user if exclusive fullscreen is active, instructing them to switch to borderless windowed before using the tool
- In the game capture module, validate that captured frames are non-blank (check that the frame is not uniformly black) and surface an explicit error if so

**Warning signs:**
- No test for black-frame capture output
- No user-facing guidance about required display mode
- Capture works in developer testing but fails in user reports

**Phase to address:**
Game capture module phase. Display mode detection and black-frame validation should be part of the initial capture implementation.

---

### Pitfall 6: Tauri Commands Blocking the Main Thread Freeze the UI

**What goes wrong:**
Tauri commands that perform long-running operations (game window capture, sequential click automation loops) without `async` will block Tauri's main thread, freezing the entire WebView2 frontend. The user sees a hung, unresponsive window with no progress feedback during the apply sequence.

**Why it happens:**
Rust commands in Tauri that are not declared `async` run on the main thread. The apply sequence — take screenshot, locate UI element, move mouse, click, wait for animation, repeat — can take 30–120 seconds. Synchronous execution of this loop makes the GUI completely unresponsive for that entire duration.

**How to avoid:**
- Declare all long-running Tauri commands as `async`
- Use `tokio::task::spawn_blocking` for CPU-intensive operations (image analysis, template matching)
- Emit progress events from the backend to the frontend using Tauri's event system (`app_handle.emit_all("apply_progress", ...)`) so the GUI can show step-by-step status
- Provide a cancellable apply loop: check a shared `AtomicBool` stop flag at each step so the user can abort via the stop button

**Warning signs:**
- `#[tauri::command]` functions without `async` that contain `thread::sleep` calls
- No progress events emitted during the apply sequence
- The stop button has no effect during automation (frontend can't receive events while main thread is blocked)

**Phase to address:**
GUI and auto-applier integration phase. Async architecture and event emission must be established before building the apply loop.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode click coordinates for a single resolution (e.g., 1920x1080) | Fast initial implementation | Breaks for every other resolution; forces complete rewrite of coordinate system | Never — resolution-adaptive design must be in from day one |
| Parse `bd=` with `unwrap()` instead of proper error types | Less boilerplate | Silent panics when encoding changes; no user-friendly error message | Never — parser errors must be typed and surfaced |
| Skip jitter/humanization on mouse clicks | Simpler code | Increases Warden detection risk for users; robotic timing is a detectable signal | Never — humanization is a safety feature, not polish |
| Use synchronous Tauri commands for automation loop | Simpler initial code | UI freezes; stop button non-functional; poor UX | Never for long-running operations |
| Rely on network connectivity check as the "offline" safety gate | Aligns with the stated requirement | D4 is always online; this check is always false and the gate never opens | Never — must redefine safety invariant |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `enigo` / `SendInput` | Using exact pixel-perfect coordinates and uniform timing | Add per-click jitter (±2px) and timing variation (±20%) to humanize input |
| `windows-capture` WGC | Not validating captured frame content | Check frame is non-blank before proceeding; surface display-mode error if black |
| Tauri IPC (frontend ↔ backend) | Sending large binary image data over IPC as base64 | Process images in Rust backend; send only extracted metadata (button positions, state) to frontend |
| `GetWindowRect` Win32 | Assuming logical coordinates equal physical pixels | Always query DPI for the target window and normalize; declare process as Per-Monitor DPI Aware |
| d2core.com URL parsing | Assuming `bd=` parameter is URL-decoded or base64 | Verify encoding assumption with test vectors from real URLs before writing the full parser |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Capturing full-screen screenshot every click step | High CPU/GPU usage; slow apply sequence | Capture a region of interest (ROI) around the target UI element instead of full screen | Immediately on any non-trivial apply sequence |
| Template matching on every captured frame in the UI event loop | CPU spikes, UI janks during apply | Run template matching only on-demand in a `spawn_blocking` task, not in a polling loop | When apply sequence runs more than a few seconds |
| Holding a game window handle across apply steps without checking validity | Handle becomes stale if user alt-tabs or resizes game | Re-query the window handle at the start of each major step | When user switches windows mid-apply |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Fetching and parsing d2core.com URLs without input validation | Malformed or malicious URLs could panic the parser or trigger unexpected behavior | Validate URL format (must match `d2core.com/d4/planner?bd=...`) before passing to decoder; use `url` crate for parsing |
| Logging decoded build data including character class/build metadata to disk | Unintended PII exposure or game account correlation | Keep logs to operational events (steps completed, errors); do not log decoded build content by default |
| Bundling Tauri app without configuring Content Security Policy for the WebView2 | Tauri's default CSP is permissive; a malicious build link could inject into the WebView if the frontend renders unsanitized data | Render build preview data as plain text or structured data only; never use `innerHTML` with user-supplied content |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Starting the apply sequence without showing the decoded build first | User discovers the wrong build was applied after the fact; no rollback | Always show a parsed build preview with "Apply" confirmation before touching the game |
| No progress indicator during the apply sequence | 30–90 second black wait looks like a crash | Emit per-step progress events: "Clicking skill 3/6", "Navigating paragon board"... |
| Failing silently when a UI element is not found at expected coordinates | Tool completes without applying all skills; user does not know | Surface explicit step-level errors: "Could not locate skill slot 4 — game UI may have changed" |
| Not telling the user what game state is required | Users run the tool in the middle of a dungeon and get confused why it doesn't work | Show a pre-flight checklist: "Is the skill assignment screen open?", "Is the game in solo mode?" |
| Applying the build and leaving the mouse cursor in the middle of the game screen | Jarring, professional tools clean up after themselves | Return cursor to a neutral position (e.g., center of tool window) after apply completes or fails |

---

## "Looks Done But Isn't" Checklist

- [ ] **d2core Parser:** Has test vectors pinned to specific `bd=` values — verify these tests exist and pass before shipping
- [ ] **Click Automation:** Verify jitter and timing variation are present — check that no two clicks in the sequence are pixel-identical or identically timed
- [ ] **DPI Scaling:** Test the full apply sequence on a machine with Windows display scaling set to 125% — not just the developer's 100%-scale monitor
- [ ] **Game Capture:** Test with game in exclusive fullscreen — verify that the tool detects a black frame and shows an error rather than attempting to click on a blank screen
- [ ] **Safety Gate:** Verify the safety invariant is correctly defined — the gate should block based on game UI state, not network connectivity
- [ ] **Stop Button:** Confirm the stop button terminates the automation loop mid-sequence — test by clicking stop after the first 2 skill clicks
- [ ] **Build Preview:** Verify the preview shows actual decoded skill names and paragon choices, not raw identifiers or indices
- [ ] **WebView2 Installer:** Test installation on a clean Windows machine without WebView2 pre-installed to verify the bundled bootstrapper works

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| d2core encoding format changes and parser breaks | LOW | Update decoder to new format; release patch; pinned test vectors will catch this quickly |
| DPI scaling bug found post-release | MEDIUM | Add `GetDpiForWindow` normalization layer; all coordinate math must be refactored through it |
| Warden flag raised (ban) on user account | HIGH (for user) | Cannot undo; prevention is the only strategy — humanization must be built in from day one |
| Safety gate wrong (allows automation when it shouldn't) | HIGH | Immediately disable apply functionality; require explicit user opt-in confirmation in all future versions |
| Tauri blocking command discovered post-release | LOW | Convert command to `async`; add `spawn_blocking` for CPU work; add progress events |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Diablo IV always-online / safety gate definition | Safety module design (early, before any automation code) | Safety gate correctly blocks at game-state level, not network-connectivity level |
| SendInput LLMHF_INJECTED flag / Warden detection | Auto-applier implementation | Code review: jitter and timing variation present; no uniform intervals or pixel-perfect targets |
| DPI scaling coordinate mismatch | Game capture + auto-applier | Integration test on 125% and 150% Windows display scaling |
| d2core.com encoding volatility | Web parser module | Pinned test vectors exist; parser returns typed errors, not panics |
| Exclusive fullscreen black capture | Game capture module | Black-frame detection test; user-visible error when capture fails |
| Tauri main thread blocking | GUI + auto-applier integration | Stop button works mid-sequence; progress events appear during apply |

---

## Sources

- Warden detection of injected input flags: [Warden (software) - Wowpedia](https://wowpedia.fandom.com/wiki/Warden_(software)), [How to Detect Input Injection - Anticheat Feature](https://guidedhacking.com/threads/how-to-detect-input-injection-anticheat-feature.20662/), [How blizzard detects AHK](https://www.ownedcore.com/forums/fps/overwatch-exploits-hacks/576716-how-blizzard-detects-ahk.html)
- Win32 SendInput LLMHF_INJECTED flag: [INPUT structure (winuser.h) - Win32 apps | Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-input)
- Blizzard ban policy for third-party tools: [PC Gamer - Blizzard permaban warning](https://www.pcgamer.com/blizzard-issues-dire-permaban-warning-to-diablo-4-players-using-any-game-modifying-software-and-calls-out-one-mod-especially/), [TurboHUD ban confirmation](https://www.gamespew.com/2023/07/blizzard-confirms-third-party-tool-turbohud-could-get-you-banned-from-diablo-4/)
- Diablo IV always-online: [PCGamesN - Is there a Diablo 4 offline mode?](https://www.pcgamesn.com/diablo-4/offline), [GameSpot - All versions online-only](https://www.gamespot.com/articles/diablo-4-doesnt-have-an-offline-mode-all-versions-/1100-6471115/)
- DPI scaling and coordinate systems: [Microsoft Learn - UI Automation and Screen Scaling](https://learn.microsoft.com/en-us/dotnet/framework/ui-automation/ui-automation-and-screen-scaling), [Microsoft Learn - DPI and device-independent pixels](https://learn.microsoft.com/en-us/windows/win32/learnwin32/dpi-and-device-independent-pixels), [Mouse input incorrectly scaled on high-DPI devices](https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/mouse-input-incorrectly-scaled)
- WGC vs BitBlt capture modes: [Screenshot-Detection-Bypass GitHub](https://github.com/xidenlz/Screenshot-Detection-Bypass), [Windows.Graphics.Capture Namespace - Microsoft Learn](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture)
- Tauri async blocking pitfalls: [Tauri - Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/), [Tauri + Rust = Speed, But Here's Where It Breaks Under Pressure](https://medium.com/@srish5945/tauri-rust-speed-but-heres-where-it-breaks-under-pressure-fef3e8e2dcb3), [Tauri + Async Rust Process](https://rfdonnelly.github.io/posts/tauri-async-rust-process/)
- Tauri WebView2 installation issues: [Tauri issue #4886 - WebView2 requirement has no fallback](https://github.com/tauri-apps/tauri/issues/4886), [Tauri Windows Installer docs](https://v2.tauri.app/distribute/windows-installer/)
- enigo crate: [enigo-rs/enigo - GitHub](https://github.com/enigo-rs/enigo)
- windows-capture crate: [NiiightmareXD/windows-capture - GitHub](https://github.com/NiiightmareXD/windows-capture)

---
*Pitfalls research for: Diablo IV Build Applier (Windows desktop game automation, Rust + Tauri)*
*Researched: 2026-03-16*
