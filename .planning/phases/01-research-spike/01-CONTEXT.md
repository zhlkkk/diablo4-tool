# Phase 1: Research Spike - Context

**Gathered:** 2026-03-16
**Status:** Complete (REVISED after JS bundle analysis)

<domain>
## Phase Boundary

Empirically verify whether d2core.com's API includes skill allocation data alongside paragon data, and document the exact API endpoint, request format, and response JSON schema.

</domain>

<decisions>
## Implementation Decisions

### API Discovery (REVISED)
- d2core.com uses **Tencent CloudBase** (腾讯云开发), NOT Firebase
- API endpoint: `https://diablocore-4gkv4qjs9c6a0b40.ap-shanghai.tcb-api.tencentcloudapi.com/web`
- Function: `function-planner-queryplan` with params `{bd, enableVariant: true}`
- **Skills ARE in the API response** — `variants[].skill` (numeric ID → points) + `variants[].equipSkills` (equipped skills with mods)
- No auth token required for read operations
- Original DevTools investigation was incorrect — the TCB SDK endpoint was missed

### Architecture Decision: direct-http (REVISED from dom-fallback)
- Direct HTTP POST with `reqwest` — verified with live 200 OK responses
- ~50ms latency, typed JSON, zero browser dependency
- Design spec: `docs/superpowers/specs/2026-03-16-web-parser-design.md`

### Claude's Discretion
- Which specific bd= build IDs to use for additional testing
- Whether to include curl commands in documentation

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### API details
- `.planning/phases/01-research-spike/SPIKE-FINDINGS.md` — Verified TCB API endpoint, request format, full response schema, test vectors
- `docs/superpowers/specs/2026-03-16-web-parser-design.md` — Full web_parser module design: data structures, error handling, testing strategy

### Research
- `.planning/research/STACK.md` — Rust crate recommendations (reqwest, serde, etc.)
- `.planning/research/ARCHITECTURE.md` — Module architecture and dependency chain

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project

### Established Patterns
- None — Phase 1

### Integration Points
- SPIKE-FINDINGS.md feeds Phase 3 (web_parser) architecture
- Test vectors (bd=1QMw, bd=1qHh) become pinned test fixtures for PARSE-07
- Design spec defines Rust data structures for all downstream modules

</code_context>

<specifics>
## Specific Ideas

- Tencent CloudBase JS SDK source code reveals the exact `callFunction` protocol
- Public app credentials in JS bundle: env=diablocore-4gkv4qjs9c6a0b40
- Response uses double-serialization: `response_data` is a JSON string inside JSON
- Glyph field in paragon is an object `{"0": "name"}`, needs custom deserialization

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-research-spike*
*Context gathered: 2026-03-16 (revised after JS bundle analysis)*
