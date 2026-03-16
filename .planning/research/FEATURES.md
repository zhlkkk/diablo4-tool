# Feature Research

**Domain:** Diablo IV build applier / desktop automation tool
**Researched:** 2026-03-16
**Confidence:** MEDIUM — d2core.com has no public documentation; API structure inferred from Diablo4Companion source code (josdemmers/Diablo4Companion). Ban-risk findings are HIGH confidence from official Blizzard sources.

---

## What the d2core.com `bd=` Parameter Actually Is

The `bd=` URL parameter (e.g., `https://www.d2core.com/d4/planner?bd=1MWy`) is a **short alphanumeric build ID**, not a base64- or LZString-encoded blob. It is a database key used to look up build data from d2core.com's backend.

When a browser loads that URL, the page fires an XHR/fetch to `web?env=diablocore` with a Cloud Function called `function-planner-queryplan`. The response JSON structure is:

```
{
  "requestId": "...",
  "data": {
    "response_data": "{\"data\":{
      \"_id\": \"1MWy\",
      \"char\": \"Paladin\",
      \"title\": \"...\",
      \"_createTime\": <unix ms>,
      \"_updateTime\": <unix ms>,
      \"variants\": [
        {
          \"name\": \"...\",
          \"gear\": { <slot>: { itemType, key, mods[], sockets[], type } },
          \"paragon\": {
            <board_name>: {
              \"data\": [\"y_x\", ...],   // node coordinates on 21x21 grid
              \"glyph\": { \"0\": \"<glyph_name>\" },
              \"index\": <board_order>,
              \"rotate\": 0|1|2|3         // 0=0°, 1=90°, 2=180°, 3=270°
            }
          }
        }
      ]
    }}"
  }
}
```

**Critical finding: Skills/skill tree allocations are NOT present** in the JSON structure parsed by Diablo4Companion. The `variants` object contains only `gear` and `paragon` keys. Either:
1. d2core.com stores skill data in a separate API call not captured by the companion app, OR
2. d2core.com encodes skill data embedded in the planner page JavaScript (not in the API JSON), OR
3. Skills are tracked client-side in the `bd=` page's frontend state and never serialized to the backend API

**This is the #1 technical risk for this project.** The tool must reverse-engineer whether skill data exists in the API and, if not, parse it from the page DOM or client state.

Source: [josdemmers/Diablo4Companion — BuildsManagerD2Core.cs](https://github.com/josdemmers/Diablo4Companion) (HIGH confidence, real implementation code)

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Parse d2core.com build link | Core premise of the tool — user pastes one link, tool does the rest | HIGH | `bd=` is a build ID requiring an API call or headless browser; API may require Selenium/DevTools to intercept (as Diablo4Companion does) |
| Display decoded build preview before applying | Users must verify what will be applied before automation begins | MEDIUM | GUI shows skill allocations + paragon boards with node count; prevents surprises |
| Detect Diablo IV game window | Cannot apply if game is not running; resolution matters for click coordinates | MEDIUM | Windows API (`FindWindow`, `GetWindowRect`); Diablo IV window class known |
| Resolution-adaptive click mapping | D4 UI element positions change with resolution (1080p vs 1440p vs 4K) | HIGH | Must compute UI positions as fractions of window size or maintain per-resolution coordinate tables |
| Apply skill points to skill tree | Primary user goal — allocates skill points in correct order respecting unlock dependencies | HIGH | In-game skill tree has dependency chains (cannot allocate node before its prerequisite); ordering matters |
| Apply paragon board choices | Secondary user goal — selects boards, rotations, nodes, glyphs | HIGH | Paragon board UI is complex: board selection wheel, rotation controls, individual node clicks across 21x21 grid |
| Online/offline safety check | Blizzard's EULA prohibits automation in online play; risk of permanent ban | HIGH | Must detect if game is in online mode (character select screen state, battle.net connectivity) and refuse to proceed |
| Start/stop controls in GUI | User needs to interrupt automation if something goes wrong | LOW | Simple UI state machine; stop should halt immediately |
| Progress feedback during apply | Multi-minute automation process; user must see what step is running | LOW | Progress bar or step log in GUI |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Zero-dependency parsing (no headless browser) | If d2core API can be called directly via HTTP, avoid Selenium entirely — faster, lighter, more reliable | HIGH | Diablo4Companion uses Chrome + DevTools to intercept; a direct HTTP call to the Cloud Function endpoint would eliminate the browser dependency. Requires reverse-engineering the exact endpoint and auth headers |
| "Refund all first" automation step | Automates the skill reset (Refund All + confirm) and paragon reset before applying new build | MEDIUM | Eliminates manual pre-step; user doesn't have to remember to reset. Must handle gold cost of respecs |
| Dry-run / preview mode | Show exactly which UI elements will be clicked (highlighted coordinates on screenshot) without actually clicking | MEDIUM | Builds trust; user sees the plan before execution |
| Per-build variant support | d2core stores multiple variants per build; let user select which variant to apply | LOW | The `variants[]` array can have N entries; UI dropdown to select |
| Pause and resume | Stop mid-apply, manually fix something in-game, then resume from where it left off | HIGH | Requires state tracking of which steps completed; complex to implement reliably |
| Build history / recently applied | Track which builds were applied to which character so user can re-apply without re-pasting the link | LOW | Local JSON store; minimal UI |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Online mode automation | "Why can't it just apply when I'm online too?" | Blizzard Warden flags automation while online; permanent ban risk; explicitly against EULA | Enforce offline-only with clear user messaging about why |
| Memory reading for state detection | More reliable way to detect current game state (which screen, online/offline) | Triggers Warden anti-cheat; violates ToS; exactly what the project constraints forbid | Use window title/screenshot-based detection only |
| Build editor / creation | "Just let me edit the skill points in the app too" | Massively increases scope; duplicates what d2core already does well | Link to d2core.com for build editing; this tool is apply-only |
| Support for Maxroll / D4Builds / Mobalytics links | "These are more popular planners" | Each has a different URL encoding scheme requiring separate reverse-engineering work; triples parser complexity | d2core-only for v1; other planners in v2+ if validated |
| Headless / background operation | "Let it run while I do something else" | Requires the game to be in focus for UI automation; background operation would automate a live game session | Require explicit user-triggered apply with game window in foreground |
| Auto-detect build from clipboard | "Watch the clipboard and apply automatically when I copy a d2core link" | Reduces user intent signal; could apply wrong build accidentally | Require explicit paste + confirm gesture |
| Gear / item farming automation | Adjacent use case but completely different risk profile | Continuous automation is core botting behavior; near-certain ban | Out of scope; this tool's value is one-time build application only |

---

## Feature Dependencies

```
[Parse d2core.com build link]
    └──requires──> [HTTP/API access to d2core.com backend OR headless browser]
    └──requires──> [Skill data exists in API response]  <-- UNVERIFIED, research gap

[Display build preview]
    └──requires──> [Parse d2core.com build link]

[Apply skill points]
    └──requires──> [Parse d2core.com build link]
    └──requires──> [Detect Diablo IV game window]
    └──requires──> [Resolution-adaptive click mapping]
    └──requires──> [Online/offline safety check passes]

[Apply paragon boards]
    └──requires──> [Parse d2core.com build link]
    └──requires──> [Detect Diablo IV game window]
    └──requires──> [Resolution-adaptive click mapping]
    └──requires──> [Online/offline safety check passes]
    └──requires after──> [Apply skill points]  (paragon unlocks after level 50)

[Online/offline safety check]
    └──requires──> [Detect Diablo IV game window]

[Resolution-adaptive click mapping]
    └──requires──> [Detect Diablo IV game window]  (need window rect for scaling)

[Zero-dependency parsing (direct HTTP)]
    └──enhances──> [Parse d2core.com build link]
    └──conflicts──> [Headless browser parsing approach]

["Refund all first" automation]
    └──enhances──> [Apply skill points]
    └──enhances──> [Apply paragon boards]
    └──requires──> [Online/offline safety check passes]
```

### Dependency Notes

- **Apply paragon boards requires apply skill points (sequentially):** Paragon unlocks at level 50; the tool should always apply skills first, then paragon. This also matches the in-game flow.
- **Apply skill/paragon requires safety check:** Safety check is a gate, not an optional step. It must pass before any UI automation begins.
- **Resolution-adaptive mapping requires window detection:** The click coordinate calculation is `screen_position = window_origin + (relative_fraction * window_size)`. Cannot compute without knowing window bounds.
- **Skill data in API is unverified:** If d2core.com does NOT include skill allocations in the `function-planner-queryplan` response, the `[Apply skill points]` feature may require a different parsing approach (page DOM, secondary API, or manual skill input). This is a project blocker.

---

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the concept.

- [ ] **Parse d2core.com build link (bd= → API JSON)** — core of everything; if this doesn't work reliably nothing else does. Must resolve the skills data gap first.
- [ ] **Display build preview before applying** — safety net; user confirms what will happen
- [ ] **Online/offline safety check** — non-negotiable; protect users from bans
- [ ] **Apply skill points** — first half of build application; simpler UI than paragon
- [ ] **Apply paragon boards (boards + nodes)** — second half; completes the build application
- [ ] **Resolution-adaptive click mapping** — required for the tool to work across any PC setup
- [ ] **Start/stop controls** — bare minimum UX for an automation tool
- [ ] **Per-build variant selection** — d2core builds commonly have multiple variants; user must choose

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **"Refund all first" automation** — add when users report friction from having to manually reset before applying
- [ ] **Dry-run / preview mode** — add when users express trust concerns about what will be clicked
- [ ] **Progress feedback (step log)** — add when first user feedback mentions confusion during apply

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Support for other build planners (Maxroll, D4Builds)** — defer until d2core support is proven solid and there's user demand for other sources
- [ ] **Pause and resume** — complex state tracking; defer unless users hit the need often
- [ ] **Build history** — nice QoL but adds storage/UI complexity; defer

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Parse d2core.com link (bd= → API data) | HIGH | HIGH | P1 |
| Skills data extraction (unverified in API) | HIGH | HIGH (research required) | P1 — blocker |
| Online/offline safety check | HIGH | MEDIUM | P1 |
| Detect game window + resolution | HIGH | MEDIUM | P1 |
| Resolution-adaptive click mapping | HIGH | HIGH | P1 |
| Apply skill points | HIGH | HIGH | P1 |
| Apply paragon boards | HIGH | HIGH | P1 |
| Build preview GUI | HIGH | MEDIUM | P1 |
| Start/stop controls | MEDIUM | LOW | P1 |
| Per-build variant selection | MEDIUM | LOW | P1 |
| "Refund all first" automation | MEDIUM | MEDIUM | P2 |
| Dry-run / preview mode | MEDIUM | MEDIUM | P2 |
| Progress feedback | LOW | LOW | P2 |
| Build history | LOW | LOW | P3 |
| Pause and resume | LOW | HIGH | P3 |
| Other planner support | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | Diablo4Companion (josdemmers) | Paragon AI (d4-paragon.com) | This Tool |
|---------|-------------------------------|----------------------------|-----------|
| Parse d2core.com builds | YES — via Selenium/DevTools | NO | YES — target: direct HTTP |
| Skills data | NOT parsed from d2core | NO | Must verify/implement |
| Paragon board display | YES — overlay only, no apply | Algorithmic optimization only | YES — auto-apply |
| In-game UI automation | NO | NO | YES — core feature |
| Online safety enforcement | N/A (read-only overlay) | N/A (no automation) | YES — offline-only gate |
| Gear/affix tracking | YES (primary feature) | NO | NO (out of scope) |
| Resolution adaptation | N/A | N/A | YES — required |
| Windows desktop app | YES (.NET WPF) | NO (web only) | YES (Rust + Tauri) |

**Key insight:** No existing tool does in-game build application via UI automation. The closest is Diablo4Companion, which parses d2core builds but only for gear/loot-filter purposes (no apply). This tool occupies an unoccupied niche.

---

## Critical Research Gap

**Skills allocation in d2core.com API is unverified.**

The `D2CoreBuildDataVariantJson` entity in Diablo4Companion only defines `gear` and `paragon` fields. There is no `skills` or `skilltree` field. This means either:

1. Skills data IS in the raw JSON but Diablo4Companion ignores it (possible — the app only needs gear/paragon for its loot-filter use case)
2. Skills data is NOT in the API response and is only in the page's frontend state

**Action required before implementing the parser:** Load a live d2core.com planner URL with browser devtools open, capture the `function-planner-queryplan` response body, and inspect the full JSON for skill fields. This is a 30-minute investigation that determines the architecture of the `web_parser` module.

If skills are absent from the API, the fallback approach is page DOM parsing (identify skill nodes in the rendered HTML) — more fragile but possible.

---

## Sources

- [josdemmers/Diablo4Companion GitHub](https://github.com/josdemmers/Diablo4Companion) — source code; D2CoreBuildJson.cs and BuildsManagerD2Core.cs reveal the d2core API structure and JSON schema (HIGH confidence)
- [Diablo4Companion — How to import and export builds wiki](https://github.com/josdemmers/Diablo4Companion/wiki/How-to-import-and-export-builds) — user-facing import workflow (MEDIUM confidence)
- [Blizzard — Stance on Unauthorized Game Modifying Software](https://www.icy-veins.com/d4/news/blizzards-stance-on-unauthorized-game-modifying-software-in-diablo-iv/) — official policy on automation (HIGH confidence)
- [Blizzard forums — Got banned for autohotkey?](https://us.forums.blizzard.com/en/d4/t/got-banned-for-autohotkey/52417) — community evidence of ban enforcement (MEDIUM confidence)
- [d4-paragon.com](https://d4-paragon.com/) — Paragon AI tool; confirms no existing tool does in-game paragon automation (MEDIUM confidence)
- [HackMD — Diablo4 d2core build links reference](https://hackmd.io/@littlex/SJzt2OEdh) — community collection confirming `bd=` is a short ID, not encoded data (MEDIUM confidence)
- [Maxroll D4Planner](https://maxroll.gg/d4/planner) — competitor reference for build planner feature set (HIGH confidence)
- [D4Builds.gg](https://d4builds.gg/build-planner/) — competitor reference for build planner feature set (HIGH confidence)

---

*Feature research for: Diablo IV Build Applier (Rust + Tauri, Windows)*
*Researched: 2026-03-16*
