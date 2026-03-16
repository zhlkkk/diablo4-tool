# Stack Research

**Domain:** Windows desktop automation tool (Rust + Tauri)
**Researched:** 2026-03-16
**Confidence:** HIGH (core Tauri/Rust stack), MEDIUM (window capture, mouse automation)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.85+ (stable) | Backend language | Required by PROJECT.md; zero-cost abstractions, memory safety without GC, excellent Windows FFI |
| Tauri | 2.10.3 | Desktop app shell + IPC bridge | Stable as of Oct 2024; v2 is current release line; 11M+ crates.io downloads; smaller binary than Electron; Rust backend with web UI |
| TypeScript + Vite | 5.x / Vite 6.x | Frontend build | Official Tauri template; hot-reload dev experience; typed IPC contract with tauri-api |
| windows | 0.61.x | Win32 API bindings | Microsoft's official Rust-for-Windows crate; code-generated from Windows metadata; use 0.61.x to match windows-capture dependency |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| windows-capture | 1.5.0 | Game window capture + frame data | Capturing the Diablo IV window for resolution detection; uses WinRT Graphics Capture API (modern, no DWM flicker) |
| enigo | 0.6.x | Mouse click automation | Simulating resolution-adaptive UI clicks; uses Windows SendInput under the hood; cross-platform if needed later |
| base64 | 0.22.1 | URL-safe base64 decode | Decoding the `bd=` parameter from d2core URLs; `URL_SAFE` engine built-in |
| flate2 | 1.1.x | DEFLATE/zlib decompression | Decompressing the decoded bd= payload if it is zlib/deflate compressed (very likely for compact URL encoding) |
| serde | 1.0.x | Serialization/deserialization | Parsing JSON build data returned from URL decoding; serde_json for the parsed struct |
| serde_json | 1.0.x | JSON parsing | Deserializing skill/paragon data after base64+decompress; standard de facto Rust JSON library |
| tokio | 1.x | Async runtime | Tauri's async commands run on tokio by default; needed for async window detection and click sequencing |
| thiserror | 2.0.x | Ergonomic error types | Each module (web_parser, game_capture, auto_applier) needs typed errors; thiserror generates Display + Error impls |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo-tauri (tauri-cli) | Build, dev server, bundle | `cargo install tauri-cli` — provides `cargo tauri dev` and `cargo tauri build` |
| Vite | Frontend hot-reload dev server | Configured via `vite.config.ts`; Tauri injects the devServer URL automatically |
| cargo test | Unit test runner | Each module gets a `#[cfg(test)]` block; `cargo test` runs them all |
| cargo clippy | Lint | Catch unsafe patterns early; configure `deny(unsafe_code)` outside the windows FFI modules |

## Installation

```bash
# Tauri CLI
cargo install tauri-cli

# Frontend dependencies (run in frontend/ dir)
npm install

# Rust dependencies are managed via Cargo.toml — no manual install needed
# Key entries in Cargo.toml:
```

```toml
[dependencies]
tauri = { version = "2.10", features = ["devtools"] }
windows-capture = "1.5"
enigo = "0.6"
base64 = "0.22"
flate2 = "1.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
thiserror = "2.0"

[dependencies.windows]
version = "0.61"
features = [
  "Win32_UI_WindowsAndMessaging",
  "Win32_System_Threading",
  "Win32_Foundation",
]
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| windows-capture 1.5 | win-screenshot | win-screenshot uses older BitBlt/PrintWindow APIs — works but misses DWM-composited frames and is slower. Use only if the WinRT Graphics Capture API is unavailable (Windows < 10 1803). |
| enigo 0.6 | rdev | rdev is a listener-first library; its send_event API is less ergonomic for scripted click sequences. Use rdev only if you also need to intercept global input events (not required here). |
| Tauri 2.x | Electron | Electron ships a 100 MB+ Chromium bundle; Tauri produces a ~4 MB installer and shares the system WebView. Electron only if you need Node.js APIs directly or target Windows 7. |
| base64 0.22 | base64-url | base64-url is a thin wrapper around the base64 crate. Use it only if you find the engine API of base64 0.22 too verbose — it adds no capability for this use case. |
| flate2 1.1 | lz4 / zstd | Only use if the bd= format turns out to use a non-deflate compression algorithm. Verify by inspecting decoded bd= bytes (0x78 header = zlib, 0x1f 0x8b = gzip). |
| Vanilla TS (no framework) | React/Vue/Svelte | For this app the GUI is a simple form + status panel. Adding a framework adds bundle size and complexity for no gain. Use React only if the paragon board visualizer becomes interactive enough to need component trees. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| winapi crate | Unmaintained since ~2020; superseded by windows-rs (Microsoft official). The API surface is identical but windows-rs is generated from live Windows metadata. | `windows` crate 0.61+ |
| windows-sys without windows | windows-sys gives raw unsafe bindings with no safe wrappers. Fine for internal use but requires extensive unsafe blocks. Only use it if you need the absolute minimal dependency. | `windows` crate with feature flags |
| screenshot-rs | Last commit 2017; uses X11-era APIs on Windows; does not support DWM composition. Will produce black frames on modern Windows. | `windows-capture` 1.5 |
| PyAutoGUI / AutoHotkey via subprocess | Shelling out to Python or AHK for mouse automation introduces a process-boundary latency of 100-200ms per click and makes the binary non-self-contained. | `enigo` in-process |
| Memory reading (ReadProcessMemory) | Violates Diablo IV ToS and triggers Warden anti-cheat. Architecturally banned per PROJECT.md. | UI automation via SendInput only |
| Any crate that injects DLLs | DLL injection triggers Warden. Use only standard user-space Win32 calls. | Win32 UI APIs through `windows` crate |

## Stack Patterns by Variant

**If bd= parameter uses gzip compression (header 0x1f 0x8b):**
- Use `flate2::read::GzDecoder` instead of `ZlibDecoder`
- Both are in the same flate2 crate — no extra dependency

**If bd= parameter uses custom binary encoding (not JSON after decompress):**
- Add `byteorder` crate for little/big-endian primitive parsing
- Build a custom bit-reader for packed skill/paragon flags
- This needs to be determined empirically by decoding a real bd= sample before implementing web_parser

**If resolution-adaptive click mapping requires image recognition (OCR of UI elements):**
- Add `image` crate 0.25 for frame decoding from windows-capture output
- Flag as MEDIUM complexity increase — avoid unless coordinate lookup tables prove insufficient

**If online detection needs network-state inspection:**
- Use `windows` crate `Win32_Networking_WinSock` feature to check active TCP connections to Blizzard's IP ranges
- Simpler alternative: check if process is `Diablo IV.exe` AND window title does NOT contain offline mode text
- Even simpler: require the user to manually confirm offline mode before enabling automation (safest, v1 appropriate)

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| windows-capture 1.5.0 | windows 0.61.3 | windows-capture 1.5 pins `windows = "^0.61.3"` — do NOT use windows 0.62 in the same build without upgrading windows-capture |
| tauri 2.10.3 | windows 0.61 | Tauri's own PR #13038 updated its internal windows dep to 0.61; keep your explicit windows dep at 0.61 for consistency |
| enigo 0.6.x | tokio 1.x | enigo is sync by default; wrap in `tokio::task::spawn_blocking` when calling from async Tauri commands |
| flate2 1.1 | no known conflicts | Pure Rust backend (miniz_oxide); no system library linkage required |

## Sources

- https://v2.tauri.app/blog/tauri-20/ — Tauri 2.0 stable release announcement (Oct 2024), HIGH confidence
- https://github.com/tauri-apps/tauri/releases — Latest Tauri version confirmed as 2.10.3, HIGH confidence
- https://github.com/NiiightmareXD/windows-capture — windows-capture capabilities and API details, MEDIUM confidence
- https://crates.io/crates/windows-capture — version 1.5.0, dependencies on windows ^0.61.3, MEDIUM confidence
- https://github.com/enigo-rs/enigo — enigo cross-platform input simulation, MEDIUM confidence (latest 0.6.1 per docs.rs search result)
- https://docs.rs/base64/latest — base64 0.22.1, URL_SAFE engine, HIGH confidence
- https://docs.rs/crate/flate2/latest — flate2 1.1.9, DEFLATE/zlib/gzip support, HIGH confidence
- https://github.com/microsoft/windows-rs/releases — windows crate 0.62.2 is latest; 0.61 preferred for windows-capture compat, HIGH confidence
- https://v2.tauri.app/develop/calling-rust/ — Tauri async command pattern, HIGH confidence

---
*Stack research for: Diablo IV Build Applier — Windows desktop automation (Rust + Tauri)*
*Researched: 2026-03-16*
