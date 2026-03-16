---
phase: 05
slug: gui-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 05 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | TypeScript compiler (tsc) + cargo check |
| **Config file** | tsconfig.json, src-tauri/Cargo.toml |
| **Quick run command** | `cd src-tauri && cargo check -p diablo4-tool 2>&1 | tail -5` |
| **Full suite command** | `npx tsc --noEmit && cd src-tauri && cargo test -p diablo4-tool && cargo check -p diablo4-tool` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx tsc --noEmit` (frontend) or `cargo check` (backend)
- **After every plan wave:** Run full suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 1 | GUI-01, GUI-02 | compile | `npx tsc --noEmit` | N/A | pending |
| 05-01-02 | 01 | 1 | GUI-03 | compile | `npx tsc --noEmit` | N/A | pending |
| 05-02-01 | 02 | 1 | GUI-04, GUI-06 | compile+unit | `npx tsc --noEmit && cargo check -p diablo4-tool` | N/A | pending |
| 05-02-02 | 02 | 1 | GUI-05 | compile | `npx tsc --noEmit` | N/A | pending |
| 05-03-01 | 03 | 2 | — | compile+unit | `cargo test -p diablo4-tool && cargo check -p diablo4-tool` | W0 | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/Cargo.toml` — add `image` and `base64` dependencies for calibration screenshot
- [ ] Existing test infrastructure (tsc, cargo test) covers framework needs

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Build preview displays correctly with variant selector | GUI-02 | Visual layout assessment | Parse a multi-variant build, switch variants, verify preview updates |
| Progress bar updates in real-time during apply | GUI-04 | Requires live automation | Start apply on game, verify progress bar moves and status text updates |
| App window stays responsive during automation | GUI-06 | UI responsiveness assessment | Start apply, try scrolling/clicking during automation |
| Calibration wizard captures correct coordinates | — | Requires game window | Open calibration, click positions, verify saved JSON matches game UI |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
