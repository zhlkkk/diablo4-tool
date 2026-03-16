# d2core.com API — Spike Findings

**Investigation date:** 2026-03-16
**Investigator:** Automated JS bundle analysis + live API verification
**Status:** COMPLETE (REVISED — original DevTools investigation was incorrect)

---

## Verdict

SKILLS_IN_API: YES
ARCHITECTURE_DECISION: direct-http

---

## API Endpoint

URL: `https://diablocore-4gkv4qjs9c6a0b40.ap-shanghai.tcb-api.tencentcloudapi.com/web?env=diablocore-4gkv4qjs9c6a0b40`
Method: POST
Auth required: NO (public app credentials embedded in JS bundle)
Platform: Tencent CloudBase (腾讯云开发), NOT Firebase

**Key finding:** The original DevTools investigation incorrectly concluded no API exists. The d2core.com
SPA uses the Tencent CloudBase JS SDK which makes POST requests to a non-obvious TCB endpoint.
The API call was likely missed because:
1. The endpoint domain (`tcb-api.tencentcloudapi.com`) doesn't match Firebase patterns searched for
2. The request may have been filtered out or fired before "Preserve log" was enabled
3. The TCB SDK request format is not a standard REST call

Analysis of the 1.9MB JS bundle (`/assets/index-CxYwtc2P.js`) revealed the exact call chain:
`queryPlan({bd, enableVariant}) → cloudRequest({funcName}) → cloudbase.callFunction() → POST to TCB`

**Public app credentials (embedded in JS bundle):**
- env: `diablocore-4gkv4qjs9c6a0b40`
- appSign: `diablocore`
- appSecret: `{appAccessKeyId: 1, appAccessKey: "ed6fe96e6ca08acf392d360094a58477"}`

---

## Request Format

```
POST https://diablocore-4gkv4qjs9c6a0b40.ap-shanghai.tcb-api.tencentcloudapi.com/web?env=diablocore-4gkv4qjs9c6a0b40
Content-Type: application/json;charset=UTF-8
x-sdk-version: @cloudbase/js-sdk/1.0.0

{
  "action": "functions.invokeFunction",
  "dataVersion": "2019-08-16",
  "env": "diablocore-4gkv4qjs9c6a0b40",
  "function_name": "function-planner-queryplan",
  "request_data": "{\"bd\":\"1QMw\",\"enableVariant\":true}"
}
```

Note: `request_data` is a JSON-encoded string (double-serialized). No auth token required for read operations.

---

## Response Format

### Raw outer JSON

```json
{
  "data": {
    "response_data": "{\"data\":{\"_id\":\"1QMw\",\"char\":\"Paladin\",\"title\":\"...\",\"variants\":[...]}}"
  },
  "requestId": "..."
}
```

### Inner schema (after JSON.parse of response_data)

```json
{
  "data": {
    "_id": "1QMw",
    "char": "Paladin",
    "title": "【琉璃】S12圣骑士开荒构筑分享",
    "_createTime": 1773169200000,
    "_updateTime": 1773169200000,
    "variants": [
      {
        "name": "variant name",
        "skill": { "6": 0, "16": 1, "47": 3, "49": 3, ... },
        "skillOrder": [...],
        "equipSkills": [
          {
            "key": "druid_wolves",
            "mods": ["Brutal Wolf Pack", "Enhanced Wolf Pack"],
            "rank": 5
          }
        ],
        "gear": { "<slot>": { "itemType": "...", "key": "...", "mods": [...] } },
        "paragon": {
          "<board_name>": {
            "data": ["y_x", ...],
            "glyph": { "0": "<glyph_name>" },
            "index": 0,
            "rotate": 0
          }
        },
        "pact": {},
        "construct": {},
        "witch": {},
        "bossPower": {},
        "seasonPower": {},
        "mercenary": {},
        "expertise": []
      }
    ]
  }
}
```

### Annotated field reference

| Field path | Type | Description | Source confidence |
|------------|------|-------------|------------------|
| data._id | string | Build ID (same as bd= param) | HIGH — verified |
| data.char | string | Character class ("Paladin", "Druid", etc.) | HIGH — verified |
| data.title | string | Build title (Chinese text) | HIGH — verified |
| data.variants[] | array | Build variants (multiple loadouts) | HIGH — verified |
| variants[].skill | object | Skill allocations: numeric skill ID → point count | HIGH — verified |
| variants[].skillOrder | array | Order of skill allocation (for replay) | HIGH — verified |
| variants[].equipSkills | array | Equipped active skills with mods and rank | HIGH — verified |
| variants[].equipSkills[].key | string | Skill key identifier (e.g. "druid_wolves") | HIGH — verified |
| variants[].equipSkills[].mods | array | Selected skill upgrades/runes | HIGH — verified |
| variants[].equipSkills[].rank | number | Skill rank/level | HIGH — verified |
| variants[].paragon | object | Paragon boards keyed by board name | HIGH — verified |
| variants[].paragon.<board>.data | array | Node coordinates on 21x21 grid ("y_x" format) | HIGH — verified |
| variants[].paragon.<board>.glyph | object | Glyph assignments | HIGH — verified |
| variants[].paragon.<board>.index | number | Board order position | HIGH — verified |
| variants[].paragon.<board>.rotate | number | Board rotation (0=0°, 1=90°, 2=180°, 3=270°) | HIGH — verified |
| variants[].gear | object | Equipment per slot | HIGH — verified |
| variants[].pact | object | Pact selections | HIGH — verified |
| variants[].construct | object | Construct data | HIGH — verified |
| variants[].mercenary | object | Mercenary configuration | HIGH — verified |
| variants[].expertise | array | Expertise selections | HIGH — verified |

---

## Skills Data Findings

**Verdict:** YES — skills data IS present in the API response.

Each variant contains three skill-related fields:
1. **`skill`** — Object mapping numeric skill IDs to point allocations (e.g., `{"47": 3, "49": 3, "56": 5}`)
2. **`skillOrder`** — Array defining the order skills were allocated (for replay in correct dependency order)
3. **`equipSkills`** — Array of equipped active skills with `key` (skill identifier), `mods` (selected upgrades), and `rank`

**Example from bd=1qHh (Druid):**
```json
{
  "equipSkills": [
    {"key": "druid_wolves", "mods": ["Brutal Wolf Pack", "Enhanced Wolf Pack"], "rank": 5},
    {"key": "druid_hurricane", "mods": ["Savage Hurricane", "Enhanced Hurricane"], "rank": 1},
    {"key": "druid_debilitating_roar", "mods": ["Innate Debilitating Roar", "Enhanced Debilitating Roar"], "rank": 3},
    {"key": "druid_petrify", "mods": ["Prime Petrify", "Supreme Petrify"], "rank": 1},
    {"key": "druid_maul", "mods": ["Enhanced Maul"], "rank": 1},
    {"key": "druid_shred", "mods": ["Enhanced Shred"], "rank": 1}
  ]
}
```

**Note:** The `skill` object uses numeric IDs that correspond to d2core's internal skill database. The `equipSkills` array uses human-readable `key` identifiers. Both are needed: `skill` for total point allocations, `equipSkills` for the 6 active skill bar slots with upgrade paths.

---

## Test Vectors

### Vector 1: bd=1QMw

- Build title: 【琉璃】S12圣骑士开荒构筑分享
- Class: Paladin
- Variants count: 4
- Investigation date: 2026-03-16

API call verified: 200 OK, 63KB response.

```
Skills: 44 skill allocations in variant 0
EquipSkills: active skill bar data with mods
Paragon: 5 boards with full node data
```

### Vector 2: bd=1qHh

- Build title: S6_德鲁伊_Tankzh_同伴_狼群BD_深坑112层通关
- Class: Druid
- Variants count: 1+
- Investigation date: 2026-03-16

API call verified: 200 OK.

```
Skills: 45 skill allocations in variant 0
EquipSkills: 6 equipped skills (wolves, hurricane, debilitating_roar, petrify, maul, shred)
Paragon: 5 boards with full node data
```

---

## Architecture Rationale

**Decision:** direct-http

Skills data, paragon data, and all build information is fully available via a direct HTTP POST to the
Tencent CloudBase API. No authentication token is required for read operations. The request format
is simple JSON with double-serialized `request_data`.

**Phase 3 approach:** Direct HTTP with `reqwest` + `serde_json`
1. Extract `bd=` value from pasted URL
2. POST to TCB endpoint with `function-planner-queryplan` function name
3. Parse outer response → JSON.parse `response_data` → extract `data`
4. Deserialize into typed Rust structs: `BuildPlan { char, variants: Vec<Variant> }`
5. Each `Variant` contains `skill`, `skillOrder`, `equipSkills`, `paragon`, `gear`

**Why NOT dom-fallback:** The original investigation incorrectly concluded no API exists. The Tencent
CloudBase SDK uses a non-standard POST endpoint that was missed in the DevTools network tab. Now that
the exact endpoint and request format are known and verified with live 200 OK responses, direct HTTP
is dramatically simpler, faster (~50ms vs 2-5s), and produces typed JSON instead of fragile DOM scraping.

**Risk:** If d2core.com changes the TCB environment ID or rotates the appSecret, the direct HTTP approach
breaks. Mitigation: the parser should detect non-200 responses and surface a clear "d2core API changed"
error. The app credentials are public (embedded in the JS bundle) and can be re-extracted by fetching
the main JS chunk.

**Evidence:** Live curl calls to the TCB endpoint returned 200 OK with complete build data including
skills, paragon, gear, and all metadata for both test vectors (bd=1QMw Paladin, bd=1qHh Druid).
