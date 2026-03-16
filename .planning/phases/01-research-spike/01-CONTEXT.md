# Phase 1: Research Spike - Context

**Gathered:** 2026-03-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Empirically verify whether d2core.com's API includes skill allocation data alongside paragon data, and document the exact API endpoint, request format, and response JSON schema. No production code is written in this phase — output is a spike document that gates the web_parser architecture in Phase 3.

</domain>

<decisions>
## Implementation Decisions

### API investigation method
- Use browser DevTools network inspection on d2core.com/d4/planner pages
- Inspect the `function-planner-queryplan` Cloud Function call (identified by Diablo4Companion source code)
- Capture full request headers, URL pattern, and response JSON
- Test with multiple known build IDs (bd= values) to confirm schema consistency

### Skills data fallback strategy
- If d2core API response includes skills data: use direct HTTP API call (simplest path)
- If d2core API does NOT include skills data: fall back to scraping the planner page DOM for skill node elements
- If DOM scraping needed: document the CSS selectors and DOM structure for skill nodes
- Record which approach is needed so Phase 3 can architect accordingly (direct HTTP vs headless browser/DOM parsing)

### Spike output format
- Structured markdown document in the phase directory
- Must include: exact API endpoint URL, required request headers, full JSON response example
- Must include: typed schema definition showing all fields (paragon nodes, gear, and skills if present)
- Must include: architecture decision — "direct HTTP" or "DOM fallback" with rationale
- Must include: at least 2 sample bd= values with their decoded responses for test vectors

### Claude's Discretion
- Which specific bd= build IDs to test with
- How many sample builds are sufficient to confirm schema stability
- Whether to include curl commands or just document the endpoint

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### d2core API structure
- `.planning/research/FEATURES.md` — Documents d2core API structure findings from Diablo4Companion source code (D2CoreBuildDataVariantJson entity)
- `.planning/research/STACK.md` — Stack recommendations including base64/flate2 for potential encoding
- `.planning/research/ARCHITECTURE.md` — Module dependency chain showing web_parser feeds into all downstream phases

### Research summary
- `.planning/research/SUMMARY.md` — Synthesized findings across all research dimensions, including the skills data gap flagged as P1 blocker

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no code exists yet

### Established Patterns
- None — this is Phase 1

### Integration Points
- Spike output directly informs Phase 3 (web_parser) architecture decisions
- Test vectors from this spike become the pinned test data for PARSE-07

</code_context>

<specifics>
## Specific Ideas

- Diablo4Companion (C# open source project) already reverse-engineered the d2core API for gear and paragon — its source code is a reference for the endpoint pattern
- The `function-planner-queryplan` Cloud Function name and `web?env=diablocore` URL pattern are known from that project
- Research found that bd= is a short build ID (database key), NOT encoded build data — the API returns the actual build JSON

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-research-spike*
*Context gathered: 2026-03-16*
