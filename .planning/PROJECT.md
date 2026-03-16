# Diablo4 Build Applier

## What This Is

A Windows desktop tool that takes a d2core.com/d4/planner build link, fetches the build data via Tencent CloudBase API, and automatically applies skills and paragon board selections to a Diablo IV character through safe, humanized UI automation. Built with Rust + Tauri 2 for a lightweight native experience.

## Core Value

Automatically apply a planned build to a Diablo IV character from a single pasted link — safely, without memory reading, and only when the game is in a safe UI state.

## Requirements

### Validated

- ✓ User can paste a d2core.com/d4/planner?bd=XXXX link and see the decoded build (skills + paragon) — v1.0
- ✓ App reverse-engineers the bd= parameter to extract skill allocations and paragon board choices — v1.0
- ✓ App captures the Diablo IV game window and detects current resolution — v1.0
- ✓ Safety module gates automation on game-UI-state (skill/paragon screen detection) — v1.0
- ✓ Unit tests exist for web_parser and game_capture modules — v1.0
- ✓ App applies skills to character via resolution-adaptive UI click automation — v1.0
- ✓ App applies paragon board choices via resolution-adaptive UI click automation — v1.0
- ✓ Resolution-adaptive click mapping adjusts to any supported game resolution — v1.0
- ✓ GUI displays parsed build preview before applying — v1.0
- ✓ GUI provides start/stop/pause controls for the apply process — v1.0
- ✓ Real-time per-step progress display during automation — v1.0
- ✓ Calibration wizard for capturing UI coordinate positions — v1.0
- ✓ F10 emergency stop hotkey halts all automation immediately — v1.0
- ✓ Click humanization (jitter, timing variation) to avoid detection — v1.0
- ✓ Bilingual Chinese/English error messages — v1.0
- ✓ GitHub Actions CI (type-check, clippy, cargo test) on push/PR to main — v1.0
- ✓ Tagged releases produce draft GitHub Release with Windows installer via tauri-action — v1.0

### Active

- [ ] Dry-run mode that shows what clicks would be made without executing
- [ ] Support for additional build planner sites (maxroll.gg, d4builds.gg)
- [ ] Build history — remember previously applied builds
- [ ] Skill refund automation (reset before applying new build)
- [ ] Auto-detect build link from clipboard

### Out of Scope

- Memory reading or injection — violates game ToS and triggers Warden anti-cheat
- Online/multiplayer automation — safety module explicitly prevents this
- Build creation or editing — this tool only applies builds from d2core links
- Mobile or cross-platform support — Windows only (Diablo IV PC)

## Context

Shipped v1.0 with ~3,246 LOC across Rust (2,631) + TypeScript (537) + YAML (78).
Tech stack: Rust, Tauri 2, React 19, Vite 6, enigo (mouse control), image, reqwest, windows-capture.
4-module backend architecture: web_parser, game_capture, safety, auto_applier.
CI/CD: GitHub Actions CI (ubuntu frontend + windows rust) and Release workflow (tauri-action NSIS installer).
Calibration wizard captures 5 UI positions; coordinate scaling normalizes to 1080p baseline.
Known risk: exact paragon board pixel coordinates require empirical measurement per resolution.

## Constraints

- **Tech stack**: Rust + Tauri — performance, safety, small binary
- **Anti-cheat safety**: Zero memory access, zero injection, UI automation only
- **Platform**: Windows only (Diablo IV PC client)
- **Safety invariant**: Game-UI-state gating (not network) — D4 is always-online
- **Testing**: Unit tests required for every module; offline fixtures, no network in CI

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Tauri over Electron | Smaller binary, better performance, Rust safety guarantees | ✓ Good — v1.0 |
| UI automation over memory injection | Anti-cheat safety, ToS compliance | ✓ Good — v1.0 |
| Game-UI-state safety invariant (not network) | D4 is always-online; offline mode doesn't exist | ✓ Good — v1.0 |
| 4-module architecture (web_parser, game_capture, safety, auto_applier) | Clear separation of concerns, testable units | ✓ Good — v1.0 |
| Direct TCB API over DOM scraping | d2core.com JS reveals TCB endpoint; no headless browser needed | ✓ Good — v1.0 |
| Double-serialized request_data format | TCB SDK wire format requires JSON string inside JSON body | ✓ Good — v1.0 |
| Chinese-only error messages | Target user base is Chinese | ✓ Good — v1.0 |
| Calibration wizard (5 click points) | Empirical coordinate capture; scales via resolution factor | ✓ Good — v1.0 |
| cfg(windows) guards on Win32 code | Enables compilation and testing on Linux/WSL | ✓ Good — v1.0 |
| scale_from_calibration normalizes to 1080p baseline | Supports calibration at any resolution | ✓ Good — v1.0 |

---
*Last updated: 2026-03-16 after v1.0 milestone*
