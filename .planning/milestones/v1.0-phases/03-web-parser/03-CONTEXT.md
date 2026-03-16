# Phase 3: Web Parser - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

`web_parser::fetch_and_decode()` returns a typed `BuildPlan` from a pasted d2core.com link via direct HTTP POST to Tencent CloudBase API. Includes pinned test vectors, typed error variants, a summary method for GUI preview, and a Tauri command to expose parsing to the frontend. The parser stores the result in AppState.

</domain>

<decisions>
## Implementation Decisions

### Variant handling
- Auto-select first variant (variants[0]) for preview and automation
- Parser returns ALL variants in BuildPlan (future-proof), but downstream consumers use only the first
- For single-variant builds, skip any variant selection UI and go straight to preview
- No variant picker UI in this phase (defer to Phase 5 or v2 if needed)

### Build preview
- Compact summary format: class, title, 6 equipped skills with upgrade count, total skill points, paragon board count with glyph names
- Parser provides a `BuildPlan::to_summary()` method (or `Display` impl) returning structured summary data — frontend just renders it
- Skill names shown as raw API keys (e.g., `druid_wolves`), not human-readable names. Numeric IDs shown as `Skill #47 (3 pts)`. Full name mapping deferred to Phase 5 or v2

### Error UX & language
- All user-facing error messages in Chinese only (target users are Chinese, d2core.com is a Chinese site)
- Technical detail (TCB error codes, HTTP status, raw response snippets) logged to console/debug — not shown to user
- API change/expiry: show `d2core API 已变更，请更新应用` — no auto-recovery, user updates the app
- No auto-scraping of JS bundle credentials, no headless browser fallback

### Tauri integration
- Tauri command name: `parse_build_link` — async, takes `{ url: String }`, returns `Result<BuildPlan, String>`
- Store parsed BuildPlan in AppState: add `build_plan: Option<BuildPlan>` field (follows existing GameState pattern)
- Always re-fetch from API on each parse (no caching) — API calls are ~50ms, always fresh
- cfg(windows)/cfg(not(windows)) guards not needed for web_parser (no Win32 dependency, works cross-platform)

### Claude's Discretion
- Exact summary formatting and Display implementation details
- reqwest client configuration (User-Agent string, connection pool settings)
- Internal module file organization within web_parser/
- Exact regex pattern for bd= extraction
- Whether to use thiserror or manual Error impl for ParserError

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### API details & architecture
- `.planning/phases/01-research-spike/SPIKE-FINDINGS.md` — Verified TCB API endpoint, request format, full response JSON schema, test vectors (bd=1QMw, bd=1qHh)
- `docs/superpowers/specs/2026-03-16-web-parser-design.md` — Full web_parser module design: D2CoreClient, data structures, error handling (ParserError enum), testing strategy, file structure, dependency list

### Existing types
- `src-tauri/src/types.rs` — BuildPlan, Variant, EquipSkill, ParagonBoard structs already defined (lines 54-86), AppState struct (lines 101-115) needs build_plan field added

### Prior decisions
- `.planning/phases/01-research-spike/01-CONTEXT.md` — Architecture decision: direct-http (revised from dom-fallback), TCB API details, public credentials
- `.planning/phases/02-scaffold-safety-game-capture/02-CONTEXT.md` — Tauri v2 scaffold, thin command pattern, cfg guards pattern

### Requirements
- `.planning/REQUIREMENTS.md` — PARSE-01 through PARSE-07 (all Phase 3 requirements)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `types.rs`: BuildPlan, Variant, EquipSkill, ParagonBoard already defined with serde derives — parser deserializes directly into these
- `types.rs`: AppState struct with Mutex pattern — add `build_plan: Option<BuildPlan>` field
- `lib.rs`: Thin Tauri command pattern (delegate to module functions, no business logic in handlers) — follow same pattern for `parse_build_link`
- `web_parser/mod.rs`: Stub exists, ready for implementation

### Established Patterns
- cfg(windows)/cfg(not(windows)) guards on platform-specific code — NOT needed for web_parser (pure HTTP, cross-platform)
- Tauri commands return `Result<T, String>` — follow this for `parse_build_link`
- AppState managed via `Mutex<AppState>` passed as `tauri::State`

### Integration Points
- `lib.rs` invoke_handler: add `parse_build_link` to `generate_handler![]` macro
- `types.rs` AppState: add `build_plan: Option<BuildPlan>` field
- `Cargo.toml`: add reqwest, tokio, thiserror, regex dependencies
- Test fixtures: `tests/fixtures/1QMw.json`, `tests/fixtures/1qHh.json`, `tests/fixtures/deleted.json`

</code_context>

<specifics>
## Specific Ideas

- Double-deserialization: API response wraps `response_data` as a JSON string inside JSON — parse twice
- Glyph field needs custom deserialization: API returns `{"0": "glyph_name"}` object, extract to `Option<String>`
- Zero-point skill entries (value = 0) should be filtered during deserialization
- Paragon boards should be sorted by `index` field (API returns as HashMap, convert to sorted Vec)
- Design spec provides exact file structure: `web_parser/{mod.rs, client.rs, types.rs, error.rs, extract.rs, parse.rs}`

</specifics>

<deferred>
## Deferred Ideas

- Variant picker UI (let user choose among multiple variants) — Phase 5 or v2
- Skill ID → human-readable name mapping table — Phase 5 or v2
- Auto-scrape JS bundle for credential recovery — v2 (PARSE-V2 scope)
- Support for other build planner sites (maxroll.gg, d4builds.gg) — v2 (PARSE-V2-01)
- Build history / previously applied builds — v2 (PARSE-V2-02)

</deferred>

---

*Phase: 03-web-parser*
*Context gathered: 2026-03-16*
