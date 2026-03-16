# Diablo4 Build Applier

## What This Is

A Windows desktop tool that takes a d2core.com/d4/planner build link, reverse-engineers the encoded build data (skills + paragon board), captures the Diablo IV game window, and automatically applies the build to the player's character through safe UI automation. Built with Rust + Tauri for a lightweight native experience.

## Core Value

Automatically apply a planned build to a Diablo IV character from a single pasted link — safely, without memory reading, and only when the game is offline.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can paste a d2core.com/d4/planner?bd=XXXX link and see the decoded build (skills + paragon)
- [ ] App reverse-engineers the bd= parameter to extract skill allocations and paragon board choices
- [ ] App captures the Diablo IV game window and detects current resolution
- [ ] App applies skills to character via resolution-adaptive UI click automation
- [ ] App applies paragon board choices via resolution-adaptive UI click automation
- [ ] Safety module disables auto-apply mode if the game is detected as online
- [ ] Resolution-adaptive click mapping adjusts to any supported game resolution
- [ ] Unit tests exist for each module (web_parser, game_capture, auto_applier, gui)
- [ ] GUI displays parsed build preview before applying
- [ ] GUI provides start/stop controls for the apply process

### Out of Scope

- Memory reading or injection — violates game ToS and triggers anti-cheat
- Online/multiplayer automation — safety module explicitly prevents this
- Build creation or editing — this tool only applies builds from d2core links
- Mobile or cross-platform support — Windows only (Diablo IV PC)
- Supporting other build planner sites — d2core.com only for v1

## Context

- Diablo IV uses Blizzard's Warden anti-cheat system; any memory read/write or injection will trigger detection
- Safe approach: UI automation only (simulated mouse clicks), which mimics human input
- d2core.com encodes builds in a bd= URL parameter that contains skill tree and paragon board data in a compressed/encoded format
- The game must be in offline mode (or at character select) for safe automation — applying builds during online play risks account action
- Tauri provides a lightweight Rust backend with a web-based frontend, ideal for this use case
- Resolution detection is critical because Diablo IV UI element positions change with resolution

## Constraints

- **Tech stack**: Rust + Tauri — performance, safety, small binary
- **Anti-cheat safety**: Zero memory access, zero injection, UI automation only
- **Platform**: Windows only (Diablo IV PC client)
- **Online safety**: Must detect online state and refuse to automate
- **Testing**: Unit tests required for every module

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + Tauri over Electron | Smaller binary, better performance, Rust safety guarantees | — Pending |
| UI automation over memory injection | Anti-cheat safety, ToS compliance | — Pending |
| Offline-only enforcement | Protect users from account bans | — Pending |
| 4-module architecture (web_parser, game_capture, auto_applier, gui) | Clear separation of concerns, testable units | — Pending |

---
*Last updated: 2026-03-16 after initialization*
