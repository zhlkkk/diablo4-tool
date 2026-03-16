---
phase: 03-web-parser
plan: 01
subsystem: api
tags: [rust, reqwest, serde_json, tauri, tencent-cloudbase, web-parser]

# Dependency graph
requires:
  - phase: 02-scaffold-safety-game-capture
    provides: AppState with Mutex pattern, thin Tauri command pattern, types.rs BuildPlan structs

provides:
  - web_parser Rust module with D2CoreClient, ParserError, extract_build_id as public API
  - parse_build_link Tauri command (async, accepts URL or raw ID, stores result in AppState)
  - AppState.build_plan field for downstream auto-applier consumption

affects:
  - 03-02 (frontend URL input and build preview uses parse_build_link command)
  - 04-auto-applier (reads AppState.build_plan to drive automation)

# Tech tracking
tech-stack:
  added:
    - reqwest 0.12 with json feature (HTTP client)
    - regex 1 (URL/ID extraction)
  patterns:
    - Double-deserialization: outer TCB JSON → parse response_data string → inner JSON
    - Glyph extraction: convert {"0": "name"} object to Option<String>
    - Zero-value skill filtering during parse (not at storage layer)
    - Paragon boards sorted by index field (API returns HashMap, output is Vec)

key-files:
  created:
    - src-tauri/src/web_parser/error.rs — ParserError enum with 6 variants, Chinese user messages
    - src-tauri/src/web_parser/extract.rs — extract_build_id (URL + raw ID support)
    - src-tauri/src/web_parser/client.rs — D2CoreClient with TCB endpoint, fetch_build, call_tcb
    - src-tauri/src/web_parser/parse.rs — parse_build_response with full type mapping and unit tests
  modified:
    - src-tauri/Cargo.toml — added reqwest and regex dependencies
    - src-tauri/src/types.rs — added build_plan: Option<BuildPlan> to AppState
    - src-tauri/src/web_parser/mod.rs — public API re-exports
    - src-tauri/src/lib.rs — parse_build_link command + generate_handler registration

key-decisions:
  - "D2CoreClient hardcodes TCB endpoint and env ID — matches d2core.com JS bundle, no auth token needed for reads"
  - "request_data is double-serialized (JSON string inside JSON) — required by TCB SDK wire format"
  - "ParserError messages are Chinese-only (target users) — technical detail available via debug format"
  - "No cfg(windows) guards on web_parser — pure HTTP, works cross-platform"
  - "BuildDeleted and BuildNotFound are separate variants for future granularity despite same user message"

patterns-established:
  - "web_parser follows thin command pattern: parse_build_link delegates to D2CoreClient, no business logic in command"
  - "Parser returns ALL variants in BuildPlan — downstream consumers choose which variant to apply"

requirements-completed: [PARSE-01, PARSE-02, PARSE-03, PARSE-04, PARSE-06]

# Metrics
duration: 3min
completed: 2026-03-16
---

# Phase 3 Plan 01: Web Parser Module Summary

**Rust web_parser module using reqwest + serde_json to fetch d2core.com builds via Tencent CloudBase TCB API with double-deserialization, zero-skill filtering, and paragon board sorting**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-16T12:10:27Z
- **Completed:** 2026-03-16T12:13:37Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Complete web_parser module with 5 submodule files compiling under `cargo check`
- D2CoreClient sends correct POST to TCB endpoint with double-serialized `request_data` and custom headers
- parse_build_response does double-deserialization, filters zero-value skills, sorts paragon boards by index, and extracts glyph from `{"0": "name"}` API format
- parse_build_link Tauri command wired and registered — stores BuildPlan in AppState for Phase 4 auto-applier

## Task Commits

Each task was committed atomically:

1. **Task 1: Create web_parser submodules and add dependencies** - `5e5a119` (feat)
2. **Task 2: Wire parse_build_link Tauri command** - `40154bd` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `src-tauri/Cargo.toml` — Added reqwest 0.12 (json feature) and regex 1
- `src-tauri/src/types.rs` — Added `build_plan: Option<BuildPlan>` to AppState struct and `new()`
- `src-tauri/src/web_parser/error.rs` — ParserError with InvalidUrl, NetworkError, ApiError, BuildNotFound, BuildDeleted, ParseError; Chinese user messages
- `src-tauri/src/web_parser/extract.rs` — extract_build_id: regex `bd=([A-Za-z0-9]+)` from URL or raw 2-10 char alphanumeric ID
- `src-tauri/src/web_parser/client.rs` — D2CoreClient with hardcoded TCB constants, fetch_build, call_tcb (double-serialized request_data)
- `src-tauri/src/web_parser/parse.rs` — parse_build_response with full type mapping and inline unit tests
- `src-tauri/src/web_parser/mod.rs` — Public re-exports: D2CoreClient, ParserError, extract_build_id
- `src-tauri/src/lib.rs` — parse_build_link async Tauri command + generate_handler registration

## Decisions Made

- No cfg(windows) guard on parse_build_link — web_parser has no Win32 dependencies
- D2CoreClient::new() hardcodes TCB env ID and endpoint matching d2core's JS bundle; read operations require no auth token
- request_data field in TCB request body is a JSON string (double-serialized), not an object
- ParserError variants BuildDeleted and BuildNotFound have same user message but are separate for future error handling granularity

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. `cargo check` passed on first attempt after implementing all modules.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- web_parser module is ready for frontend integration (Phase 3 Plan 02)
- parse_build_link Tauri command available for the URL input UI component
- AppState.build_plan field ready for Phase 4 auto-applier consumption
- No blockers

---
*Phase: 03-web-parser*
*Completed: 2026-03-16*
