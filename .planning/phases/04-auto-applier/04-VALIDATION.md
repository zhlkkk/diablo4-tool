---
phase: 04
slug: auto-applier
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | src-tauri/Cargo.toml |
| **Quick run command** | `cargo test -p diablo4-tool --lib auto_applier` |
| **Full suite command** | `cargo test -p diablo4-tool` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p diablo4-tool --lib auto_applier`
- **After every plan wave:** Run `cargo test -p diablo4-tool`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | APPLY-03 | unit | `cargo test -p diablo4-tool --lib auto_applier::coords` | W0 | pending |
| 04-01-02 | 01 | 1 | APPLY-04 | unit | `cargo test -p diablo4-tool --lib auto_applier::humanize` | W0 | pending |
| 04-02-01 | 02 | 2 | APPLY-01, APPLY-02 | unit | `cargo test -p diablo4-tool --lib auto_applier::executor` | W0 | pending |
| 04-02-02 | 02 | 2 | APPLY-05, APPLY-06 | unit | `cargo test -p diablo4-tool --lib auto_applier::executor` | W0 | pending |
| 04-02-03 | 02 | 2 | APPLY-07 | compile | `cargo check -p diablo4-tool` | N/A | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/Cargo.toml` — add enigo and rand dependencies
- [ ] Test stubs for coordinate mapping pure functions
- [ ] Test stubs for humanization (jitter, timing) pure functions

*Existing test infrastructure (cargo test) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Skill clicks hit correct UI elements at 1080p | APPLY-01, APPLY-03 | Requires game running at 1080p | Run apply with test build, verify skills allocated correctly |
| Paragon clicks navigate board correctly | APPLY-02, APPLY-03 | Requires game running with paragon board open | Run apply with test build, verify paragon nodes selected |
| Click jitter looks natural | APPLY-04 | Visual/behavioral assessment | Observe click patterns during automation, verify no robotic feel |
| Emergency stop halts within one click | APPLY-06 | Requires real-time automation running | Press F10 during apply, verify immediate halt |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
