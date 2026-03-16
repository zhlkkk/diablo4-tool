---
phase: 1
slug: research-spike
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-16
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual verification (research spike — no code produced) |
| **Config file** | none |
| **Quick run command** | `test -f .planning/phases/01-research-spike/SPIKE-FINDINGS.md` |
| **Full suite command** | `test -f .planning/phases/01-research-spike/SPIKE-FINDINGS.md && grep -q "## Verdict" .planning/phases/01-research-spike/SPIKE-FINDINGS.md` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `test -f .planning/phases/01-research-spike/SPIKE-FINDINGS.md`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | SPIKE-01 | manual + file check | `grep -q "skills" .planning/phases/01-research-spike/SPIKE-FINDINGS.md` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | SPIKE-02 | manual + file check | `grep -q "## API Endpoint" .planning/phases/01-research-spike/SPIKE-FINDINGS.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] No test infrastructure needed — this is a manual research spike
- [ ] Output document template (SPIKE-FINDINGS.md) defined in plan

*Existing infrastructure covers all phase requirements via file existence checks.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Skills data presence in API | SPIKE-01 | Requires live browser DevTools inspection | Open d2core.com planner URL, inspect network tab for queryplan response, check for skills key |
| API schema documentation | SPIKE-02 | Requires human judgment on completeness | Verify SPIKE-FINDINGS.md has endpoint URL, headers, full JSON example, and architecture decision |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 1s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
