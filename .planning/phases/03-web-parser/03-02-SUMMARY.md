---
phase: 03-web-parser
plan: 02
subsystem: testing
tags: [rust, fixtures, unit-tests, serde_json, web_parser, cargo-test]

# Dependency graph
requires:
  - phase: 03-01
    provides: "web_parser module with parse_build_response, extract_build_id, ParserError"

provides:
  - "Pinned fixture: 1QMw.json (Paladin, 4 variants, 35 skills, 5 boards, 6 equip skills)"
  - "Pinned fixture: 1qHh.json (Druid, 1 variant, 41 skills, 5 boards, 6 equip skills)"
  - "Pinned fixture: deleted.json (API errMsg error response)"
  - "15 unit tests for extract_build_id, parse_build_response, structural invariants"
  - "Integration test runner: cargo test --test web_parser_test"

affects: [03-03, future-phases]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Load fixture files with fs::read_to_string(\"tests/fixtures/{name}\")"
    - "Use matches!() macro for error variant checks: assert!(matches!(result, Err(ParserError::InvalidUrl(_))))"
    - "Paragon sort invariant tested with .windows(2) sliding window"
    - "Zero-filter invariant tested with HashMap.values().all(|&v| v > 0)"

key-files:
  created:
    - src-tauri/tests/fixtures/1QMw.json
    - src-tauri/tests/fixtures/1qHh.json
    - src-tauri/tests/fixtures/deleted.json
    - src-tauri/tests/web_parser_test.rs
  modified:
    - src-tauri/src/web_parser/mod.rs
    - src-tauri/src/web_parser/parse.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "parse_build_response exported via pub use in web_parser/mod.rs for integration test access"
  - "web_parser module exported as pub mod in lib.rs for integration test visibility"
  - "skillOrder is null in live API responses — test verifies Vec<u32> type, not non-emptiness"
  - "[Rule 1 - Bug] parse.rs now checks errMsg before data field — returns BuildNotFound not ParseError for API errors"

patterns-established:
  - "Integration tests in src-tauri/tests/ use diablo4_tool_lib:: import path"
  - "Load fixture with fs::read_to_string(path) + serde_json::from_str — no network dependency"
  - "Network-dependent tests marked #[ignore] for CI safety"

requirements-completed: [PARSE-07]

# Metrics
duration: 4min
completed: 2026-03-16
---

# Phase 3 Plan 02: Test Fixtures and Unit Tests Summary

**15 offline unit tests against pinned live API fixtures (1QMw Paladin, 1qHh Druid, deleted) covering extract, parse, structural invariants, and error paths**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-03-16T12:17:16Z
- **Completed:** 2026-03-16T12:21:15Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Captured 3 live API fixtures (68KB Paladin, 36KB Druid, 200B deleted error) via curl POST to TCB endpoint
- Created 15 unit tests covering extract_build_id (7 tests), parse_build_response (4 tests), structural invariants (3 tests), and live API call (1 ignored)
- Fixed bug in parse.rs: errMsg response now returns BuildNotFound instead of ParseError

## Task Commits

Each task was committed atomically:

1. **Task 1: Capture live API responses as pinned test fixtures** - `98321e9` (feat)
2. **Task 2: Write unit tests for web_parser module** - `81291bf` (test)

**Plan metadata:** (created in this commit)

## Files Created/Modified

- `src-tauri/tests/fixtures/1QMw.json` — Pinned Paladin build API response (68KB, 4 variants, 35 non-zero skills, 5 boards)
- `src-tauri/tests/fixtures/1qHh.json` — Pinned Druid build API response (36KB, 1 variant, 41 non-zero skills, 5 boards)
- `src-tauri/tests/fixtures/deleted.json` — Deleted build error response: `{"errMsg":"数据不存在"}`
- `src-tauri/tests/web_parser_test.rs` — 244-line integration test file, 15 tests, all passing offline
- `src-tauri/src/web_parser/mod.rs` — Added `pub use parse::parse_build_response` for test access
- `src-tauri/src/web_parser/parse.rs` — Fixed errMsg handling (BuildNotFound vs ParseError)
- `src-tauri/src/lib.rs` — Changed `mod web_parser` to `pub mod web_parser` for integration test visibility

## Decisions Made

- `parse_build_response` needs to be `pub use`-d from `web_parser/mod.rs` to be accessible from integration tests — plan's import path required it
- `pub mod web_parser` required in `lib.rs` for integration tests to access the module at all
- `skillOrder` is null (not an array) in live API responses for both test vectors — test updated to verify Vec<u32> type without asserting non-emptiness

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed parse.rs errMsg handling returns wrong error variant**
- **Found during:** Task 1 (inspecting deleted fixture response)
- **Issue:** `deleted.json` API response is `{"errMsg":"数据不存在"}` — no `data` field. Parser returned `ParseError("missing 'data' field...")` instead of `BuildNotFound`
- **Fix:** Added errMsg check before `data` extraction: if `inner.get("errMsg")` is present, return `BuildNotFound(build_id + errMsg)`
- **Files modified:** `src-tauri/src/web_parser/parse.rs`
- **Verification:** `test_parse_deleted_build` passes, asserting `BuildNotFound | BuildDeleted`
- **Committed in:** `98321e9` (Task 1 commit)

**2. [Rule 2 - Missing Critical] Exported parse_build_response and web_parser module**
- **Found during:** Task 2 (writing integration tests)
- **Issue:** `parse_build_response` was private; `web_parser` module was `mod` (private) in lib.rs — integration tests couldn't access either
- **Fix:** Added `pub use parse::parse_build_response` to `web_parser/mod.rs`; changed `mod web_parser` to `pub mod web_parser` in `lib.rs`
- **Files modified:** `src-tauri/src/web_parser/mod.rs`, `src-tauri/src/lib.rs`
- **Verification:** All 15 tests compile and pass
- **Committed in:** `81291bf` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing export)
**Impact on plan:** Both fixes necessary for correctness and test compilation. No scope creep.

## Issues Encountered

- `skillOrder` field is null in both live API responses — the API apparently doesn't populate this field for the test vectors. Test was updated to check Vec<u32> type rather than asserting non-empty. This is valid API behavior (optional field).

## User Setup Required

None — no external service configuration required. All tests run offline using pinned fixtures.

## Next Phase Readiness

- 15 offline unit tests confirm web_parser module correctness: extract, parse, sort, filter, error paths all verified
- Fixtures pinned and committed — CI will never need network access for these tests
- Plan 03 (frontend URL input) can proceed with confidence that backend parsing is correct

---
*Phase: 03-web-parser*
*Completed: 2026-03-16*
