---
phase: 01-research-spike
plan: 01
subsystem: research
tags: [d2core, web-parser, api-investigation, dom-scraping, architecture]

# Dependency graph
requires: []
provides:
  - "SPIKE-FINDINGS.md: empirical d2core.com investigation results"
  - "Architecture decision: dom-fallback (no API exists, bd= is client-side decoded)"
  - "DOM selectors for skill extraction: .skill-node with data-skill-id/rank/upgrades"
  - "2 bd= test vectors for Phase 3 PARSE-07 pinned tests"
affects:
  - "03-web-parser — architecture choice (dom-fallback vs direct-http)"
  - "Phase 3 PARSE-07 — pinned test data (bd=1QMw, bd=2p6t)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "dom-fallback: headless browser or bd= reverse-engineering required for skill extraction"
    - "bd= parameter: encodes full build state client-side, no server round-trip"

key-files:
  created:
    - ".planning/phases/01-research-spike/SPIKE-FINDINGS.md"
  modified: []

key-decisions:
  - "ARCHITECTURE_DECISION=dom-fallback: d2core.com has no function-planner-queryplan API call; bd= parameter decoded entirely client-side"
  - "Phase 3 primary approach: attempt to reverse-engineer bd= encoding in Rust before committing to headless browser (~150 MB dependency)"
  - "DOM selectors confirmed: .skill-node[data-skill-id/rank/upgrades] and .paragon-node[data-coord/type] inside .planner-container"

patterns-established:
  - "Spike findings documented before any implementation — architecture decisions are evidence-based"

requirements-completed: [SPIKE-01, SPIKE-02]

# Metrics
duration: 20min
completed: 2026-03-16
---

# Phase 1 Plan 01: d2core.com API Spike Summary

**d2core.com has no backend API for build data — bd= parameter is decoded client-side by JavaScript, requiring DOM scraping or bd= reverse-engineering for Phase 3 web_parser**

## Performance

- **Duration:** ~20 min (human investigation + authoring)
- **Started:** 2026-03-16T09:38:53Z
- **Completed:** 2026-03-16T09:42:00Z
- **Tasks:** 2 (1 human-action checkpoint + 1 auto)
- **Files modified:** 1

## Accomplishments

- Confirmed that d2core.com serves the planner as a fully client-side SPA — the assumed `function-planner-queryplan` Cloud Function is never called during page load
- Documented the DOM structure for skill and paragon nodes with exact CSS classes and data attributes needed for Phase 3 scraping
- Established architecture decision `dom-fallback` with recommendation to first attempt bd= parameter reverse-engineering as a lighter-weight alternative to headless Chromium

## Task Commits

Each task was committed atomically:

1. **Task 1: Live browser DevTools investigation** - human-action checkpoint (no commit — human performed investigation)
2. **Task 2: Author SPIKE-FINDINGS.md** - `a428488` (docs)

**Plan metadata:** (this commit — docs: complete plan)

## Files Created/Modified

- `.planning/phases/01-research-spike/SPIKE-FINDINGS.md` — Full spike findings: verdict, DOM structure, test vectors, architecture rationale

## Decisions Made

- `ARCHITECTURE_DECISION: dom-fallback` — No API exists; d2core.com decodes the `bd=` URL parameter using client-side JavaScript only. Phase 3 must either reverse-engineer the encoding or use a headless browser.
- Recommended Phase 3 strategy: attempt to decode `bd=` directly in Rust first (inspect JS bundle for decode algorithm, likely base64 + compression). Fall back to `chromiumoxide` only if encoding is opaque.
- DOM selectors for Phase 3: `.skill-node[data-skill-id]`, `.skill-node[data-rank]`, `.skill-node[data-upgrades]`, `.paragon-node[data-coord]`, `.paragon-node[data-type]`

## Deviations from Plan

The plan's `acceptance_criteria` required the `## API Endpoint` section to contain a URL with `cloudfunctions.net`. No such URL exists — this criterion was based on an unconfirmed hypothesis. The section is populated with the actual finding (no endpoint exists), which satisfies the spirit of the requirement even if it does not match the assumed positive case.

All other acceptance criteria are met: all 7 sections present, SKILLS_IN_API=NO, ARCHITECTURE_DECISION=dom-fallback, 2 test vectors documented.

## Issues Encountered

- Vector 2 (bd=2p6t) showed possible 404/incomplete behavior during investigation. Recorded as-is; Phase 3 should validate with a fresh confirmed-working bd= value before using as a pinned test fixture.

## User Setup Required

None — no external service configuration required. Phase 3 implementation choices may require installing Rust crates (`chromiumoxide` or `lz-string`/`base64` decoders) but no manual configuration steps.

## Next Phase Readiness

**Phase 3 (web_parser) is unblocked with the following known state:**
- Architecture: `dom-fallback` — must handle client-side rendered page, not a JSON API
- Recommended first attempt: reverse-engineer `bd=` encoding (inspect d2core.com JS bundle)
- Fallback: `chromiumoxide` headless browser (adds ~150 MB Chromium binary)
- CSS selectors documented and ready for implementation
- Test vector bd=1QMw (Paladin, Falling Star Rank 1) confirmed working

**Resolved blocker:** STATE.md blocker "d2core.com API may not include skill allocation data" is now resolved — skills are absent from any API (no API exists), architecture is confirmed `dom-fallback`.

---
*Phase: 01-research-spike*
*Completed: 2026-03-16*

## Self-Check: PASSED

- FOUND: .planning/phases/01-research-spike/SPIKE-FINDINGS.md
- FOUND: .planning/phases/01-research-spike/01-01-SUMMARY.md
- FOUND: commit a428488 (Task 2 — SPIKE-FINDINGS.md)
