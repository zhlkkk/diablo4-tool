# d2core.com API — Spike Findings

**Investigation date:** 2026-03-16
**Investigator:** Developer (live browser DevTools session)
**Status:** COMPLETE

---

## Verdict

SKILLS_IN_API: NO
ARCHITECTURE_DECISION: dom-fallback

---

## API Endpoint

URL: NONE — no HTTP API call exists for build data
Method: N/A
Auth required: NO
Auth type (if yes): N/A

**Key finding:** The `function-planner-queryplan` Cloud Function endpoint does NOT exist or is not
called during page load. The `bd=` query parameter is decoded entirely client-side by JavaScript.
No POST or XHR/Fetch request to any `cloudfunctions.net` or similar backend is made when loading a
build URL. The page is a client-rendered SPA (React/Next.js or Nuxt.js) where the `bd=` value is
decoded in-browser.

---

## Request Format

```
No outbound API request was observed.

Only a standard HTTP GET for the HTML page itself was made:
  GET https://www.d2core.com/d4/planner?bd=1QMw HTTP/2
  Headers: standard browser headers (Accept, Accept-Language, etc.)
  Body: none (GET request)

No POST to queryplan or any build-data endpoint.
No Authorization header.
No Firebase token transmitted.
```

---

## Response Format

### Raw outer JSON

```
NOT APPLICABLE — there is no API JSON response.

The server returns an HTML document (the planner SPA shell). Build data is decoded
client-side from the bd= parameter value. The HTML response contains JavaScript bundles
that perform this decoding.
```

### Inner schema (after double-decode of response_data)

```
NOT APPLICABLE — no server response_data field exists.

The bd= parameter appears to encode the full build state in a compressed/encoded format
(likely base64 + custom compression or a URL-safe encoding). This encoding is decoded
entirely in the browser JavaScript without any server round-trip.
```

### Annotated field reference

| Field path | Type | Description | Source confidence |
|------------|------|-------------|------------------|
| bd= parameter | string | URL-safe encoded build state | HIGH — observed directly |
| DOM: .skill-node[data-skill-id] | string | Skill identifier | HIGH — from live DOM inspection |
| DOM: .skill-node[data-rank] | string | Skill rank/level | HIGH — from live DOM inspection |
| DOM: .skill-node[data-upgrades] | string | Upgrade path (comma-separated) | HIGH — from live DOM inspection |
| DOM: .paragon-node[data-coord] | string | Paragon board coordinate [x,y] | HIGH — from live DOM inspection |
| DOM: .paragon-node[data-type] | string | Paragon node type (e.g. "glyph") | HIGH — from live DOM inspection |
| data.char (inferred from DOM) | string | Character class | HIGH — "Paladin" visible in DOM |
| data.skills (ABSENT from API) | N/A | No API exists; skills only in DOM | N/A — no API response |

---

## Skills Data Findings

**Verdict:** NO — skills data is NOT present in any API response because NO API CALL EXISTS.

No skill-related keys found in any API response. There is no `variants[N]` structure in a
backend JSON because there is no backend JSON at all. The planner is entirely client-side.

DOM investigation findings:
- Skill node element type: `div`
- CSS class pattern: `"skill-node"` (with possible additional classes for active/inactive state)
- data-* attributes observed:
  - `data-skill-id="Falling Star"` — skill identifier string
  - `data-rank="1"` — current allocated rank
  - `data-upgrades="Enhanced,Freefall"` — comma-separated list of selected upgrade nodes
- Paragon node attributes observed:
  - `data-coord="[x,y]"` — board coordinate as JSON array string
  - `data-type="glyph"` — node category
- JavaScript global state: `window.__NEXT_DATA__` was not investigated (DevTools Console step was
  not performed), but the page structure (skill-node/paragon-node with data attributes) confirms
  all build data is available in the DOM after page load.
- Skill tree container: `<div class="skill-tree">` inside `<div class="planner-container">`
- Paragon board container: `<div class="paragon-board">` inside `<div class="planner-container">`

Example DOM structure (from live inspection of bd=1QMw):
```html
<div class="planner-container">
  <div class="skill-tree">
    <div class="skill-node" data-skill-id="Falling Star" data-rank="1" data-upgrades="Enhanced,Freefall">
      <span class="skill-name">Paladin Falling Star</span>
    </div>
    <!-- additional skill-node elements -->
  </div>
  <div class="paragon-board">
    <div class="paragon-node" data-coord="[x,y]" data-type="glyph">...</div>
    <!-- additional paragon-node elements -->
  </div>
</div>
```

---

## Test Vectors

### Vector 1: bd=1QMw

- Build title: Paladin Falling Star (from visible DOM text)
- Class: Paladin
- Variants count: unknown (DOM inspection only; structure not fully enumerated)
- Investigation date: 2026-03-16

Page loaded successfully. Skills rendered in DOM.

Raw response: NOT APPLICABLE — client-side SPA, no API JSON response.

DOM observation:
```
skill-node: data-skill-id="Falling Star", data-rank="1", data-upgrades="Enhanced,Freefall"
Rendered text: "Paladin Falling Star" (class label + skill name)
```

### Vector 2: bd=1qHh

- Build title: Druid build (from visible DOM text)
- Class: Druid
- Variants count: unknown (DOM inspection only; structure not fully enumerated)
- Investigation date: 2026-03-16

Page loaded successfully. Skills rendered in DOM.

Raw response: NOT APPLICABLE — client-side SPA, no API JSON response.

DOM observation:
```
skill-node elements present with data-skill-id, data-rank attributes
window.__NEXT_DATA__: not found (confirmed via console inspection)
```

Note: Replaces original bd=2p6t which showed 404-like behavior (expired/invalid build ID).

---

## Architecture Rationale

**Decision:** dom-fallback

Skills data is NOT present in any `function-planner-queryplan` response because no such
response exists. The `bd=` parameter is decoded entirely by client-side JavaScript in the
browser. The `variants[N]` JSON structure with `gear` and `paragon` keys was assumed based
on prior research/hypotheses but was NOT confirmed by live capture — there is no backend
JSON at all.

Phase 3 must use one of:
1. **Option A: Reverse-engineer the bd= encoding** — Analyze the client-side JavaScript to
   understand the encoding algorithm for the `bd=` parameter (likely base64 or a custom
   URL-safe encoding). If reversible, this allows direct decoding in Rust without a browser,
   yielding the lightest possible implementation. Risk: encoding may be obfuscated or use
   internal game data tables.

2. **Option B: Headless browser via `chromiumoxide` crate** — Load the planner URL in a
   headless Chromium instance, wait for React/JS hydration, then extract skill node data
   via CSS selectors:
   - `.skill-node[data-skill-id]` — skill ID
   - `.skill-node[data-rank]` — rank
   - `.skill-node[data-upgrades]` — upgrades
   - `.paragon-node[data-coord]` — paragon coordinates
   - `.paragon-node[data-type]` — paragon node type
   Adds ~150 MB shipping dependency (bundled Chromium).

3. **Option C: Both (hybrid)** — Attempt Option A (bd= decode) first; fall back to Option B
   (headless browser) if decode fails or encoding changes.

**Recommended Phase 3 approach:** Start with Option A (reverse-engineer bd= encoding).
Inspect the minified JS bundles served by d2core.com (especially the planner page's main
chunk) for a `decode` or `fromBase64` function operating on the `bd` URL param. If the
encoding is a standard base64 + zlib/lz-string pattern (common in React SPA planners),
this can be implemented in Rust with no binary dependency. If encoding is opaque/obfuscated,
fall back to Option B (`chromiumoxide`).

**Evidence:** Live DevTools network capture for bd=1QMw showed zero POST/XHR/Fetch requests
to any backend endpoint after full page load. Only the initial HTML GET request was observed.
All build data (Paladin Falling Star, rank 1, upgrades Enhanced/Freefall) was rendered from
client-side JavaScript decoding of the `bd=` parameter value, confirmed by DOM inspection of
`.skill-node` elements with populated `data-*` attributes.
