# Phase 1: Research Spike - Research

**Researched:** 2026-03-16
**Domain:** d2core.com API investigation — empirical verification of skills data availability
**Confidence:** HIGH (investigation methodology), MEDIUM (expected API structure from Diablo4Companion), LOW (skills data presence — unverified, that is the point of this phase)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### API investigation method
- Use browser DevTools network inspection on d2core.com/d4/planner pages
- Inspect the `function-planner-queryplan` Cloud Function call (identified by Diablo4Companion source code)
- Capture full request headers, URL pattern, and response JSON
- Test with multiple known build IDs (bd= values) to confirm schema consistency

#### Skills data fallback strategy
- If d2core API response includes skills data: use direct HTTP API call (simplest path)
- If d2core API does NOT include skills data: fall back to scraping the planner page DOM for skill node elements
- If DOM scraping needed: document the CSS selectors and DOM structure for skill nodes
- Record which approach is needed so Phase 3 can architect accordingly (direct HTTP vs headless browser/DOM parsing)

#### Spike output format
- Structured markdown document in the phase directory
- Must include: exact API endpoint URL, required request headers, full JSON response example
- Must include: typed schema definition showing all fields (paragon nodes, gear, and skills if present)
- Must include: architecture decision — "direct HTTP" or "DOM fallback" with rationale
- Must include: at least 2 sample bd= values with their decoded responses for test vectors

### Claude's Discretion
- Which specific bd= build IDs to test with
- How many sample builds are sufficient to confirm schema stability
- Whether to include curl commands or just document the endpoint

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SPIKE-01 | Developer can confirm whether d2core.com API includes skill allocation data by inspecting live network traffic | Investigation methodology documented below; tooling (browser DevTools + Network tab) requires no installation |
| SPIKE-02 | Developer has documented the exact d2core API endpoint, request format, and response JSON schema for builds | Known partial schema from Diablo4Companion; gap list defined; capture procedure detailed below |
</phase_requirements>

---

## Summary

Phase 1 is a pure empirical investigation — no production code is written. Its entire value is answering one binary question: does the d2core.com `function-planner-queryplan` Cloud Function response include skill allocation data? The answer gates the entire web_parser architecture in Phase 3. If skills are present in the API, Phase 3 is a straightforward direct-HTTP implementation. If absent, Phase 3 requires either DOM scraping (fragile, needs headless browser) or a secondary API discovery effort.

The d2core API structure is partially known from Diablo4Companion (josdemmers/Diablo4Companion), which successfully parses the same endpoint for gear and paragon data. The C# source code in `D2CoreBuildJson.cs` and `BuildsManagerD2Core.cs` reveals the response shape but only deserializes `gear` and `paragon` fields — skills are absent from that implementation, but we cannot determine whether this is because (a) Diablo4Companion simply ignores skills fields that are present, or (b) skills fields do not exist in the response at all. Both interpretations are consistent with the source code. Phase 1 resolves this ambiguity.

**Primary recommendation:** Open a known d2core.com build URL in Chrome/Firefox, open DevTools Network tab, filter for `function-planner-queryplan`, reload the page, capture the full response body, and inspect the complete JSON for any skills-related keys. This takes approximately 30 minutes and eliminates the #1 architectural uncertainty for the entire project.

---

## Investigation Methodology

This section provides the exact procedure the developer must follow to satisfy SPIKE-01 and SPIKE-02.

### Step 1: Obtain Sample Build URLs

Find at least 2 public d2core.com build links. Sources:
- The Diablo4Companion wiki has a known-good build link used for testing
- Any build shared publicly on Reddit r/diablo4 or D4 Discord
- The URL format is: `https://www.d2core.com/d4/planner?bd=XXXX` where XXXX is a 4-6 character alphanumeric ID

Ideal samples to collect:
- One build that prominently features skill allocation (a popular meta build where skill choices are critical)
- One build from a different class (Barbarian vs. Sorcerer vs. Druid, etc.)
- At least one build with paragon data visible on the planner page

Note each `bd=` value down — these become the test vectors for PARSE-07 in Phase 3.

### Step 2: Capture the API Call in Browser DevTools

For each build URL:

1. Open Chrome or Firefox
2. Navigate to `https://www.d2core.com/d4/planner?bd=XXXX`
3. Before navigating, open DevTools (F12) and go to the **Network** tab
4. Enable "Preserve log" to avoid losing requests on page load
5. Reload the page (Ctrl+R or Cmd+R)
6. In the Network filter bar, search for: `function-planner-queryplan`
7. Alternatively search for `queryplan` or filter by XHR/Fetch type and look for a POST to a Firebase/GCP endpoint

The request should appear as a POST to a URL matching the pattern:
```
https://<region>-<project>.cloudfunctions.net/function-planner-queryplan
```
or via Firebase Callable Functions:
```
https://<region>-diablocore.cloudfunctions.net/function-planner-queryplan
```
or through a Firebase `web?env=diablocore` routing endpoint.

### Step 3: Extract Request Details

Click the captured request in DevTools. Record ALL of the following in the spike output document:

**Request tab:**
- Full URL (including any query parameters)
- Method (POST expected)
- All request headers (Content-Type, Authorization/auth token if present, Origin, Referer)
- Request body / payload (raw JSON)

**Response tab:**
- Full response body as raw JSON (copy entire text)
- Response headers (Content-Type, Cache-Control)
- HTTP status code

**CRITICAL: Do not truncate the response.** Copy the entire JSON including all nested keys. The skills data, if present, may be in an unexpected key name (`skills`, `skilltree`, `skill_allocations`, `skillpoints`, `active_skills`, etc.).

### Step 4: Inspect the Response JSON for Skills Data

Parse the copied JSON (paste into https://jsonformatter.org/ or similar). Look for:

**Known fields (from Diablo4Companion source):**
```json
{
  "requestId": "...",
  "data": {
    "response_data": "{\"data\":{
      \"_id\": \"...\",
      \"char\": \"...\",
      \"title\": \"...\",
      \"variants\": [
        {
          \"name\": \"...\",
          \"gear\": { ... },
          \"paragon\": { ... }
        }
      ]
    }}"
  }
}
```

Note: `response_data` is a JSON-encoded string inside JSON — double-parse it.

**Look for any of these keys at any nesting level:**
- `skills`
- `skill`
- `skilltree`
- `skillAllocations`
- `skill_allocations`
- `skillPoints`
- `activeSkills`
- `passiveSkills`
- `buildSkills`
- Any key containing "skill" (case-insensitive)

**Answer the key question:** Are skills present in the `variants[N]` object alongside `gear` and `paragon`?

### Step 5: If Skills Are Absent — DOM Investigation

If no skill data appears in the API response, open DevTools on the same planner page and inspect the rendered DOM:

1. Switch to the **Elements** tab
2. Find the skill tree section of the page visually
3. Right-click on individual skill nodes and select "Inspect"
4. Record:
   - The HTML element type for a skill node (likely `div` or `button`)
   - The CSS class name pattern
   - Whether skill name/rank is stored in a `data-*` attribute
   - Whether skill point count is readable in the DOM

Example DOM structure to document if found:
```html
<div class="skill-node active" data-skill-name="Fireball" data-points="5">
  <span class="skill-rank">5/5</span>
</div>
```

Also check: window JavaScript state. In the browser console, try:
```javascript
// Check if skills data is in a global React/Vue/Angular state
window.__REACT_STATE__
window.__NEXT_DATA__
window.__NUXT__
// Or look for a global app store
window.store
window.app
// Try Vue DevTools or React DevTools panel if available
```

### Step 6: Test Reproducibility with Multiple Build IDs

Repeat Steps 2–5 for at least one additional `bd=` value of a different class. Confirm:
- The API endpoint URL is the same
- The request headers are the same
- The response JSON schema is the same (field names are consistent, even if values differ)

Schema inconsistency between builds would indicate versioned or class-specific API formats — document any differences.

### Step 7: Check for Authentication Requirements

Examine the request headers captured in Step 3:
- Is there an `Authorization: Bearer ...` header?
- Is there a session cookie being sent?
- Is there a Firebase ID token in the request body?

If YES: the direct-HTTP approach requires replicating that auth flow. Document the token source (cookie, localStorage, etc.) and whether it is short-lived (expiry matters for a long-lived desktop app).

If NO (public unauthenticated endpoint): confirm this is the case. Diablo4Companion operates without credentials, so unauthenticated is expected but must be verified.

---

## Known API Structure (Pre-Investigation Baseline)

The following is known from Diablo4Companion source code analysis (MEDIUM confidence). Use it as a starting point, not ground truth.

### Endpoint Pattern

**Base URL:** Firebase Cloud Functions endpoint — exact hostname varies by Firebase project region
**Known from Diablo4Companion:** Contains `web?env=diablocore` in the HTTP client configuration
**Function name:** `function-planner-queryplan`

From the C# source:
```csharp
// BuildsManagerD2Core.cs
private const string FunctionName = "function-planner-queryplan";
private const string BaseUrl = "https://us-central1-diablocore.cloudfunctions.net";
// or similar regional endpoint
```

**Action:** Verify the exact hostname from the live network capture. Firebase project IDs and regional endpoints can change.

### Request Format (Inferred)

```json
POST https://<endpoint>/function-planner-queryplan
Content-Type: application/json

{
  "data": {
    "buildId": "XXXX"
  }
}
```

Firebase Callable Functions always wrap the payload in `{ "data": { ... } }`.

**Action:** Confirm the exact request body key name (`buildId`, `bd`, `id`, `planId`, etc.) from the live capture.

### Response Format (Partially Known)

```json
{
  "result": {
    "response_data": "<JSON-encoded string>"
  }
}
```

The `response_data` value is a JSON-encoded string (not an object) — must be JSON-parsed a second time. The inner object structure:

```json
{
  "data": {
    "_id": "1MWy",
    "char": "Necromancer",
    "title": "Build title here",
    "_createTime": 1703123456789,
    "_updateTime": 1703987654321,
    "variants": [
      {
        "name": "Variant 1",
        "gear": {
          "helm": { "itemType": "...", "key": "...", "mods": [], "sockets": [], "type": "..." },
          "chest": { ... },
          "gloves": { ... },
          "boots": { ... },
          "pants": { ... },
          "amulet": { ... },
          "ring1": { ... },
          "ring2": { ... },
          "weapon1h1": { ... },
          "offhand": { ... }
        },
        "paragon": {
          "<BoardName>": {
            "data": ["10_10", "11_10", "10_11"],
            "glyph": { "0": "<GlyphName>" },
            "index": 0,
            "rotate": 0
          }
        }
      }
    ]
  }
}
```

**CRITICAL gap:** The `variants[N]` object above may also contain a `skills` or similar key that Diablo4Companion never deserializes. Phase 1 exists to verify this.

### Paragon Data Schema Detail

For the paragon fields (HIGH confidence from Diablo4Companion):
- `data`: Array of `"y_x"` formatted strings indicating activated node positions on the 21x21 grid
- `glyph`: Object mapping slot index to glyph name; `"0"` is center socket
- `index`: Board order (0-based); determines which board is placed where
- `rotate`: Integer 0–3 representing 90° rotation increments (0=0°, 1=90°, 2=180°, 3=270°)

---

## Architecture Decision Framework

The spike must conclude with one of two architecture decisions for Phase 3:

### Decision A: Direct HTTP (Preferred)

**Condition:** Skills data IS present in the `function-planner-queryplan` response

**Architecture:** `web_parser` makes a single `reqwest` POST to the Cloud Function endpoint, JSON-parses the `response_data` string, and deserializes into a `BuildPlan` struct. No browser dependency. ~50 lines of Rust.

**Risk:** d2core API change breaks parser silently. Mitigated by pinned test vectors + typed error variants.

### Decision B: DOM Fallback

**Condition:** Skills data is NOT in the API response

**Architecture:** `web_parser` must either:
1. Embed a headless browser (e.g., via `fantoccini` or `chromiumoxide` crate) to load the planner page and extract DOM state, OR
2. Identify a secondary API call made by the page that DOES contain skills data (additional DevTools investigation required)

**Tradeoffs for DOM approach:**
- Requires shipping a WebDriver binary or Chromium dependency (~150 MB)
- Slower (page load vs. API call)
- More fragile to d2core frontend updates
- But: is the only option if skills are not in the main API response

**Stack additions if DOM fallback needed:**
| Library | Version | Purpose |
|---------|---------|---------|
| chromiumoxide | 0.7.x | Headless Chrome via CDP, async Rust |
| fantoccini | 0.21.x | WebDriver client (requires separate geckodriver/chromedriver) |

Recommend `chromiumoxide` over `fantoccini` if DOM fallback is required — it uses Chrome DevTools Protocol directly without a separate driver process, and Diablo4Companion's approach (intercepting the CDP network traffic) validates this pattern.

---

## Common Pitfalls in This Investigation

### Pitfall 1: Truncating the Response Body

**What goes wrong:** Copying only the visible portion of the response in DevTools, missing deeply nested fields.

**Why it happens:** Large JSON responses render truncated by default in the DevTools Preview pane.

**How to avoid:** Use the **Response** tab (not Preview) and copy the raw text. Or right-click the request → "Copy" → "Copy response" to get the full body.

### Pitfall 2: Missing the Double-JSON-Decode

**What goes wrong:** Inspecting the outer JSON and concluding the schema only has `requestId` and `data.response_data`, not realizing `response_data` is itself a JSON string.

**How to avoid:** In the DevTools console, run `JSON.parse(response.data.response_data)` on the captured response to see the inner object. Or paste the `response_data` string value into a JSON formatter.

### Pitfall 3: Confusing Firebase REST API vs. Callable Functions

**What goes wrong:** Trying to `curl` the endpoint directly and getting 404s because Firebase Callable Functions require the `{ "data": {...} }` wrapper and specific headers.

**How to avoid:** Use the exact request captured from DevTools as the template for any manual reproduction. The `Content-Type: application/json` header and the `{ "data": {...} }` wrapper are both required.

### Pitfall 4: Expired Build IDs

**What goes wrong:** Choosing an old bd= value from a stale URL and receiving a 404 or empty response.

**How to avoid:** Use recently shared builds (within the past few months). The d2core database may prune old inactive builds. Test that the build actually loads in the browser before trying to capture the API call.

### Pitfall 5: Missing Auth Token Capture

**What goes wrong:** The request shows an `Authorization` header but the developer ignores it, concluding the endpoint is unauthenticated.

**How to avoid:** Inspect ALL headers in the captured request. If an auth header is present, document: the header name, token format (JWT? Firebase ID token?), where it originates (auto-set by Firebase SDK? Manual from localStorage?), and its TTL/expiry behavior.

---

## Spike Output Document Specification

The deliverable of Phase 1 is a file at `.planning/phases/01-research-spike/SPIKE-FINDINGS.md`.

That document MUST contain the following sections (the planner enforces these as requirements):

```markdown
## Verdict
SKILLS_IN_API: YES | NO | PARTIAL
ARCHITECTURE_DECISION: direct-http | dom-fallback | dom-fallback-secondary-api

## API Endpoint
URL: <exact URL from network capture>
Method: POST | GET
Auth required: YES | NO
Auth type (if yes): <Firebase ID token | session cookie | other>

## Request Format
Headers:
  Content-Type: ...
  Authorization: ... (if present)
Body:
  <exact JSON>

## Response Format
<full JSON response — not truncated>
Inner schema after double-decode:
  <annotated JSON showing all field names and types>

## Skills Data Findings
<Is there a skills/skilltree/skillAllocations key in variants[N]? What is its structure?>
<If absent: what is the exact DOM structure for skill nodes on the planner page?>

## Test Vectors
### Vector 1: bd=XXXX
- Build title: ...
- Class: ...
- Variants count: N
- Full response: <base64-encoded or raw JSON stored in .planning/research/test-vectors/>

### Vector 2: bd=YYYY
- Build title: ...
- Class: ...
- Variants count: N
- Full response: ...

## Architecture Rationale
<Why direct-http OR why dom-fallback was chosen, with specific evidence from the API inspection>
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | None required for Phase 1 — this is a manual investigation |
| Config file | N/A |
| Quick run command | N/A — output is a markdown document, not executable code |
| Full suite command | N/A |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPIKE-01 | Skills data presence confirmed via live API inspection | manual-only | N/A — inherently manual browser DevTools work | N/A |
| SPIKE-02 | API endpoint URL, headers, and response schema documented | manual-only | N/A — inherently manual network capture | N/A |

Both requirements are manual-only because the investigation requires a live browser session, network access to d2core.com, and human interpretation of response JSON. No automated test can substitute for this empirical verification.

### Wave 0 Gaps

None — no test infrastructure is created in Phase 1. The deliverable is a markdown document (`SPIKE-FINDINGS.md`), not code. Test vectors captured here become the pinned test data for PARSE-07 in Phase 3, but that test scaffolding is Wave 0 work for Phase 3, not Phase 1.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Selenium + WebDriver for d2core data extraction | Direct Firebase Callable Function HTTP call (Diablo4Companion v2+) | ~2023 | Eliminates browser dependency if API includes all needed data |
| Manual bd= base64 decode assumption | bd= is a database key, not encoded data | Community verified 2023 | Parser does NOT need base64 or flate2 to decode the URL parameter; just pass the bd= value to the API |

**Deprecated/outdated:**
- Assumption that `bd=` is a base64/LZString-encoded blob: WRONG. It is a short alphanumeric database key. The `base64` and `flate2` crates in STACK.md are for potential response body encoding, not URL parameter decoding.

---

## Open Questions

1. **Does `variants[N]` contain a skills key not deserialized by Diablo4Companion?**
   - What we know: Diablo4Companion only defines `gear` and `paragon` fields in `D2CoreBuildDataVariantJson`
   - What's unclear: Whether this is because (a) d2core does not include skills in the API, or (b) Diablo4Companion simply never needed skills and didn't bother to deserialize them
   - Recommendation: Phase 1 investigation resolves this directly; assume nothing until the live response is inspected

2. **Does the endpoint require Firebase authentication?**
   - What we know: Diablo4Companion reads d2core builds — implies either no auth, or the companion handles auth transparently via Chrome/DevTools interception
   - What's unclear: Whether a standalone HTTP call to the Firebase function succeeds without an auth token
   - Recommendation: Capture both the request headers and whether auth tokens appear in the DevTools capture; if auth is required, document the token acquisition flow

3. **Is the `response_data` value always a JSON-encoded string, or can it be a JSON object directly?**
   - What we know: Diablo4Companion source code parses it as a string requiring deserialization
   - What's unclear: Whether this is an API design choice or an artifact of an older API version
   - Recommendation: Verify in the live capture; if it changed to direct JSON object, the decoder becomes simpler

4. **Are there multiple API versions or endpoint patterns active simultaneously?**
   - What we know: Firebase projects can have regional endpoints; the exact URL may differ by geography
   - What's unclear: Whether the endpoint URL varies by user region, or is globally consistent
   - Recommendation: Document the exact URL from the capture; note your geographic location so downstream callers can compare

---

## Sources

### Primary (HIGH confidence)
- [josdemmers/Diablo4Companion](https://github.com/josdemmers/Diablo4Companion) — D2CoreBuildJson.cs and BuildsManagerD2Core.cs; partial d2core API schema (gear + paragon); confirmed `function-planner-queryplan` endpoint name
- `.planning/research/FEATURES.md` — Project research synthesizing Diablo4Companion findings; identifies skills data gap as #1 blocker
- `.planning/research/SUMMARY.md` — Cross-domain research summary; bd= is a database key (not encoded), confirmed from community + Diablo4Companion source

### Secondary (MEDIUM confidence)
- [HackMD — Diablo4 d2core build links reference](https://hackmd.io/@littlex/SJzt2OEdh) — community confirming bd= is a short alphanumeric ID
- [Diablo4Companion wiki — How to import and export builds](https://github.com/josdemmers/Diablo4Companion/wiki/How-to-import-and-export-builds) — confirms planner import workflow; implies API access pattern

### Tertiary (LOW confidence)
- General Firebase Callable Functions documentation pattern — request/response wrapper structure (`{ "data": {...} }`) is Firebase SDK standard; applies by inference to this endpoint

---

## Metadata

**Confidence breakdown:**
- Investigation methodology: HIGH — browser DevTools network capture is a standard, well-documented technique; no uncertainty
- Expected API structure (gear + paragon fields): MEDIUM — derived from Diablo4Companion C# source, a real implementation but may not reflect full response
- Skills data presence: LOW — this is the unknown this phase exists to resolve
- Firebase endpoint pattern: MEDIUM — standard Firebase Callable Functions format applies; exact URL must be verified live

**Research date:** 2026-03-16
**Valid until:** Findings are valid until d2core.com changes their API. Given the endpoint has been stable long enough for Diablo4Companion to ship against it, 2026-06-16 is a reasonable stability estimate. Pin test vectors immediately after capture.
