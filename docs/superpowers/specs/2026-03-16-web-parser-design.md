# Web Parser Module Design

**Date:** 2026-03-16
**Status:** Approved
**Module:** `web_parser`
**Approach:** Direct HTTP to Tencent CloudBase API (Approach A)

## Overview

The `web_parser` module fetches and parses Diablo IV build data from d2core.com. It takes a `bd=` build link or raw ID, calls the Tencent CloudBase API directly via HTTP POST, and returns a typed `BuildPlan` struct containing skills, paragon boards, and equipped skills.

No browser, no DOM scraping, no base64 decoding. Pure HTTP + JSON.

## Key Discovery

d2core.com is a **uni-app** (Vue-based Chinese cross-platform framework) SPA hosted on **Tencent CloudBase** (腾讯云开发). The JS SDK makes POST requests to a TCB endpoint that was initially missed during DevTools investigation because:
1. The endpoint domain (`tcb-api.tencentcloudapi.com`) doesn't match Firebase patterns
2. The TCB SDK request format is not standard REST

Analysis of the 1.9MB JS bundle revealed the exact call chain and public credentials.

## API Details

### Endpoint

```
POST https://diablocore-4gkv4qjs9c6a0b40.ap-shanghai.tcb-api.tencentcloudapi.com/web?env=diablocore-4gkv4qjs9c6a0b40
Content-Type: application/json;charset=UTF-8
x-sdk-version: @cloudbase/js-sdk/1.0.0
```

### Request Body

```json
{
  "action": "functions.invokeFunction",
  "dataVersion": "2019-08-16",
  "env": "diablocore-4gkv4qjs9c6a0b40",
  "function_name": "function-planner-queryplan",
  "request_data": "{\"bd\":\"1QMw\",\"enableVariant\":true}"
}
```

Note: `request_data` is a JSON-encoded string (double-serialized). No auth token required for read operations.

### Response Structure

```
response.data.response_data  →  JSON string
  └── JSON.parse  →  { data: { _id, char, title, variants: [...] } }
```

Each variant contains:
- `skill: { "47": 3, "49": 3, ... }` — numeric skill ID → points allocated
- `skillOrder: [...]` — order of allocation (for replay)
- `equipSkills: [{ key, mods, rank }, ...]` — 6 active skill bar slots
- `paragon: { "Board_Name": { data, glyph, index, rotate }, ... }` — paragon boards
- `gear`, `pact`, `construct`, `mercenary`, `expertise` — other build data

### Public Credentials (reference only — NOT used in requests)

These are embedded in d2core's JS bundle for documentation purposes. They are **not included** in the HTTP request — read operations work without authentication. If the API begins requiring auth in the future, these values would need to be sent in the request body per TCB SDK conventions.

```
env: "diablocore-4gkv4qjs9c6a0b40"
appSign: "diablocore"
appSecret: { appAccessKeyId: 1, appAccessKey: "ed6fe96e6ca08acf392d360094a58477" }
```

## Data Flow

```
User pastes URL
    │
    ▼
extract_build_id("https://d2core.com/d4/planner?bd=1QMw")
    │                                          → "1QMw"
    ▼
D2CoreClient::call_tcb("function-planner-queryplan", {bd: "1QMw", enableVariant: true})
    │                                          → POST to TCB endpoint
    ▼
Double-parse: outer JSON → response_data string → inner JSON
    │                                          → { data: { char, variants, ... } }
    ▼
Deserialize into BuildPlan { id, char_class, title, variants }
    │
    ▼
Return to GUI for preview display
```

## Rust Data Structures

```rust
pub struct BuildPlan {
    pub id: String,              // "1QMw"
    pub char_class: String,      // "Paladin"
    pub title: String,           // build title
    pub variants: Vec<Variant>,
}

pub struct Variant {
    pub name: String,
    pub skill: HashMap<u32, u32>,        // skill_id → points (0-filtered)
    pub skill_order: Vec<u32>,           // allocation order for replay
    pub equip_skills: Vec<EquipSkill>,   // 6 active skill bar slots
    pub paragon: Vec<ParagonBoard>,      // sorted by index
}

pub struct EquipSkill {
    pub key: String,           // "druid_wolves"
    pub mods: Vec<String>,     // ["Brutal Wolf Pack", "Enhanced Wolf Pack"]
    pub rank: u32,             // 5
}

pub struct ParagonBoard {
    pub name: String,          // "Paragon_Druid_00"
    pub index: u32,            // board order position
    pub rotate: u32,           // 0-3 (0°/90°/180°/270°)
    #[serde(rename = "data")]
    pub nodes: Vec<String>,    // ["y_x", ...] coordinates on 21x21 grid
    pub glyph: Option<String>, // extracted from API's {"0": "glyph_name"} object
}
```

### Design Decisions

- `skill` uses `HashMap<u32, u32>` — API returns numeric IDs, not string keys
- `paragon` converts from API's `HashMap<String, BoardData>` to `Vec<ParagonBoard>` sorted by `index`
- `ParagonBoard.nodes` uses `#[serde(rename = "data")]` — API field is `data`, renamed for clarity
- `ParagonBoard.glyph` requires custom deserialization: API returns `{"0": "glyph_name"}` object, parsed to `Option<String>` by extracting the first value
- Zero-point skill entries filtered during deserialization
- All structs derive `serde::Deserialize` for automatic JSON parsing
- Unknown fields silently ignored (`serde` default) — allows API evolution without breaking the parser

## API Client

```rust
pub struct D2CoreClient {
    http: reqwest::Client,
    endpoint: String,
    env: String,
}

impl D2CoreClient {
    pub fn new() -> Self { /* hardcoded TCB constants */ }

    pub async fn fetch_build(&self, url_or_id: &str) -> Result<BuildPlan, ParserError> {
        let build_id = extract_build_id(url_or_id)?;
        let resp = self.call_tcb("function-planner-queryplan",
            json!({"bd": build_id, "enableVariant": true})
        ).await?;
        parse_build_response(resp)
    }

    async fn call_tcb(&self, func_name: &str, params: Value) -> Result<Value, ParserError> {
        // POST with double-serialized request_data
    }
}

pub fn extract_build_id(input: &str) -> Result<String, ParserError> {
    // Regex: extract bd= from URL, or accept raw 2-10 char alphanumeric ID
}
```

## Error Handling

```rust
pub enum ParserError {
    InvalidUrl(String),            // bd= extraction failed
    NetworkError(reqwest::Error),  // connection timeout, DNS failure
    ApiError { code: String, message: String }, // TCB returned error code
    BuildNotFound(String),         // bd= build doesn't exist
    BuildDeleted(String),          // build was deleted by author
    ParseError(String),            // JSON schema changed, deserialization failed
}
```

### Error Strategy

- **Network errors:** Surface as "d2core.com 不可用，请检查网络连接"
- **API errors:** Surface with TCB error code for debugging
- **Build not found / deleted:** Clear message "构建不存在或已被删除"
- **Parse errors:** Include the specific field name that failed — helps diagnose API schema changes

### Credential Management

TCB env ID and endpoint are hardcoded constants (matching d2core's JS bundle). No auth token needed for reads. If d2core changes env ID, a new app version is required. No automatic credential scraping — this is more reliable and secure.

### Timeouts

- Connect timeout: 10 seconds
- Total request timeout: 30 seconds
- On timeout: return `ParserError::NetworkError` with user message "d2core.com 请求超时"

## Testing Strategy

### Unit Tests (no network)

```
test_extract_build_id_from_full_url     "https://d2core.com/d4/planner?bd=1QMw" → "1QMw"
test_extract_build_id_from_raw_id       "1QMw" → "1QMw"
test_extract_build_id_invalid           "not-a-url" → Err(InvalidUrl)

test_parse_paladin_build                fixtures/1QMw.json → 4 variants, 35+ skills, 5 boards
test_parse_druid_build                  fixtures/1qHh.json → 6 equip skills, 41 skills, 5 boards
test_parse_deleted_build                fixtures/deleted.json → Err(BuildDeleted)
test_parse_empty_variant                variant with no skill/paragon → default empty values

test_paragon_sorted_by_index            boards sorted by index, not HashMap order
test_skill_zero_filtered                skill points = 0 entries removed
test_rotation_values                    rotate 0-3 maps correctly
```

### Integration Tests (network required, `#[ignore]` in CI)

```
test_live_api_call                      real HTTP call, verify API still works
test_live_error_handling                invalid ID returns correct error
```

### Fixture Management

- Save complete API responses to `tests/fixtures/1QMw.json` and `tests/fixtures/1qHh.json`
- Unit tests read from files, no network dependency
- Update fixture files when d2core API schema changes

## Verified Test Vectors

| Build ID | Class | Variants | Skills | Equip Skills | Paragon Boards | Status |
|----------|-------|----------|--------|--------------|----------------|--------|
| 1QMw | Paladin | 4 | 44 raw → 35 after zero-filter (71 pts) | 6 | 5 | ✓ 200 OK, 63KB |
| 1qHh | Druid | 1 | 45 raw → 41 after zero-filter (71 pts) | 6 | 5 | ✓ 200 OK |

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| d2core changes TCB env ID | Low | High — parser breaks | `ParserError::ApiError` with clear message; env ID re-extractable from JS bundle |
| API schema adds/removes fields | Medium | Low — serde ignores unknown fields | `#[serde(default)]` on optional fields; pinned test fixtures catch regressions |
| d2core adds auth requirement | Low | High — reads blocked | Fall back to headless browser approach (Approach B) as emergency path |
| Rate limiting | Low | Medium — burst requests blocked | Single request per build; add retry with backoff if needed |
| Skill numeric IDs change between seasons | Medium | Medium — wrong skill mapping | Skill ID → name mapping needs seasonal data file; out of scope for web_parser (consumer responsibility) |

## Dependencies

```toml
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
thiserror = "2.0"
regex = "1"
```

## File Structure

```
src-tauri/src/
  web_parser/
    mod.rs          — public API: D2CoreClient, fetch_build()
    client.rs       — TCB HTTP client, call_tcb()
    types.rs        — BuildPlan, Variant, EquipSkill, ParagonBoard
    error.rs        — ParserError enum
    extract.rs      — extract_build_id(), URL parsing
    parse.rs        — parse_build_response(), JSON → typed structs

tests/
  fixtures/
    1QMw.json       — pinned Paladin build response
    1qHh.json       — pinned Druid build response
    deleted.json    — deleted build response
  web_parser_test.rs — unit + integration tests
```
