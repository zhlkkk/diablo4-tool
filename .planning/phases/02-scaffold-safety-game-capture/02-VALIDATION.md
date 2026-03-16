---
phase: 2
slug: scaffold-safety-game-capture
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | SAFE-01 | unit | `cargo test safety` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | SAFE-02 | unit | `cargo test safety` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | SAFE-03 | unit | `cargo test safety` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | SAFE-04 | integration | `cargo test hotkey` | ❌ W0 | ⬜ pending |
| 02-01-05 | 01 | 1 | SAFE-05 | unit | `cargo test safety` | ❌ W0 | ⬜ pending |
| 02-01-06 | 01 | 1 | SAFE-06 | unit | `cargo test safety` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | CAPT-01 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | CAPT-02 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 1 | CAPT-03 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 1 | CAPT-04 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |
| 02-02-05 | 02 | 1 | CAPT-05 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |
| 02-02-06 | 02 | 1 | CAPT-06 | unit | `cargo test game_capture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/safety/tests.rs` — stubs for SAFE-01 through SAFE-06
- [ ] `src-tauri/src/game_capture/tests.rs` — stubs for CAPT-01 through CAPT-06
- [ ] cargo test infrastructure — comes with Rust, no install needed

*Existing infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| F10 hotkey stops automation when D4 has focus | SAFE-04 | Requires running game + system-wide hotkey | 1. Start D4, open skill tree. 2. Start automation. 3. Press F10. 4. Verify automation stops. |
| Pixel sampling detects skill tree screen | SAFE-01 | Requires running D4 at target resolution | 1. Open D4 skill tree at 1080p. 2. Run safety check. 3. Verify "safe" state detected. |
| Exclusive fullscreen warning | CAPT-04 | Requires D4 in exclusive fullscreen mode | 1. Set D4 to Fullscreen. 2. Start app. 3. Verify warning displayed. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
