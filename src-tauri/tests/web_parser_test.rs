// Integration tests for the web_parser module.
// All tests run offline using pinned fixture files — no network access required.
//
// Fixture files are in tests/fixtures/ (relative to src-tauri/).
// Run with: cargo test --test web_parser_test

use diablo4_tool_lib::web_parser::{extract_build_id, parse_build_response, ParserError};
use diablo4_tool_lib::BuildPlan;
use std::fs;

// ─── extract_build_id tests ───────────────────────────────────────────────────

#[test]
fn test_extract_build_id_from_full_url() {
    let result = extract_build_id("https://d2core.com/d4/planner?bd=1QMw");
    assert_eq!(result.unwrap(), "1QMw");
}

#[test]
fn test_extract_build_id_from_url_with_extra_params() {
    let result = extract_build_id("https://d2core.com/d4/planner?bd=1QMw&foo=bar");
    assert_eq!(result.unwrap(), "1QMw");
}

#[test]
fn test_extract_build_id_from_url_with_leading_params() {
    let result = extract_build_id("https://d2core.com/d4/planner?other=123&bd=1qHh&foo=bar");
    assert_eq!(result.unwrap(), "1qHh");
}

#[test]
fn test_extract_build_id_from_raw_id() {
    let result = extract_build_id("1QMw");
    assert_eq!(result.unwrap(), "1QMw");
}

#[test]
fn test_extract_build_id_from_raw_id_with_whitespace() {
    let result = extract_build_id("  1qHh  ");
    assert_eq!(result.unwrap(), "1qHh");
}

#[test]
fn test_extract_build_id_invalid() {
    let result = extract_build_id("not-a-valid-input");
    assert!(
        matches!(result, Err(ParserError::InvalidUrl(_))),
        "Expected InvalidUrl, got: {:?}",
        result
    );
}

#[test]
fn test_extract_build_id_empty() {
    let result = extract_build_id("");
    assert!(
        matches!(result, Err(ParserError::InvalidUrl(_))),
        "Expected InvalidUrl for empty string, got: {:?}",
        result
    );
}

// ─── parse_build_response tests — Paladin (1QMw) ─────────────────────────────

fn load_fixture(name: &str) -> serde_json::Value {
    let path = format!("tests/fixtures/{}", name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {} as JSON: {}", path, e))
}

#[test]
fn test_parse_paladin_build() {
    let fixture = load_fixture("1QMw.json");
    let result = parse_build_response(fixture, "1QMw");
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

    let build: BuildPlan = result.unwrap();
    assert_eq!(build.id, "1QMw");
    assert_eq!(build.char_class, "Paladin");
    assert_eq!(build.variants.len(), 4, "Expected 4 variants for 1QMw Paladin");

    let v0 = &build.variants[0];
    assert!(
        v0.skill.len() >= 35,
        "Expected 35+ skills (after zero filter) in variant[0], got {}",
        v0.skill.len()
    );
    assert_eq!(
        v0.equip_skills.len(),
        6,
        "Expected 6 equip skills in variant[0]"
    );
    assert_eq!(
        v0.paragon.len(),
        5,
        "Expected 5 paragon boards in variant[0]"
    );
}

#[test]
fn test_parse_druid_build() {
    let fixture = load_fixture("1qHh.json");
    let result = parse_build_response(fixture, "1qHh");
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);

    let build: BuildPlan = result.unwrap();
    assert_eq!(build.id, "1qHh");
    assert!(
        build.char_class.contains("Druid"),
        "Expected char_class to contain 'Druid', got: {}",
        build.char_class
    );

    let v0 = &build.variants[0];
    assert_eq!(
        v0.equip_skills.len(),
        6,
        "Expected 6 equip skills in Druid variant[0]"
    );

    // Verify druid_wolves is in equip skills
    let has_wolves = v0
        .equip_skills
        .iter()
        .any(|es| es.key.contains("druid_wolves"));
    assert!(has_wolves, "Expected druid_wolves in Druid equip skills");

    assert_eq!(
        v0.paragon.len(),
        5,
        "Expected 5 paragon boards in Druid variant[0]"
    );
}

#[test]
fn test_parse_deleted_build() {
    let fixture = load_fixture("deleted.json");
    let result = parse_build_response(fixture, "ZZZZ_NONEXISTENT");

    assert!(
        matches!(
            result,
            Err(ParserError::BuildNotFound(_)) | Err(ParserError::BuildDeleted(_))
        ),
        "Expected BuildNotFound or BuildDeleted for deleted build, got: {:?}",
        result
    );
}

// ─── Structural invariant tests ───────────────────────────────────────────────

#[test]
fn test_paragon_sorted_by_index() {
    let fixture = load_fixture("1QMw.json");
    let build = parse_build_response(fixture, "1QMw").unwrap();
    let v0 = &build.variants[0];

    for window in v0.paragon.windows(2) {
        assert!(
            window[0].index <= window[1].index,
            "Paragon boards not sorted by index: {} > {}",
            window[0].index,
            window[1].index
        );
    }
}

#[test]
fn test_skill_zero_filtered() {
    let fixture = load_fixture("1QMw.json");
    let build = parse_build_response(fixture, "1QMw").unwrap();
    let v0 = &build.variants[0];

    for (&skill_id, &points) in &v0.skill {
        assert!(
            points > 0,
            "Skill {} has zero points — should have been filtered out",
            skill_id
        );
    }
}

#[test]
fn test_glyph_extraction() {
    // At least one paragon board in 1QMw should have a glyph
    let fixture = load_fixture("1QMw.json");
    let build = parse_build_response(fixture, "1QMw").unwrap();
    let v0 = &build.variants[0];

    let has_glyph = v0.paragon.iter().any(|b| b.glyph.is_some());
    assert!(
        has_glyph,
        "Expected at least one paragon board with a glyph in 1QMw variant[0]"
    );
}

#[test]
fn test_skill_order_is_vec() {
    // skill_order can be empty (API returns null for many builds); verify it parses as Vec
    let fixture = load_fixture("1QMw.json");
    let build = parse_build_response(fixture, "1QMw").unwrap();
    let v0 = &build.variants[0];

    // skill_order is a Vec<u32> — may be empty if API returns null (valid state)
    // Verify the field exists and all entries are valid u32 (no panics)
    let _order: &Vec<u32> = &v0.skill_order;
    // Verify skill map is non-empty (proves parse succeeded)
    assert!(
        !v0.skill.is_empty(),
        "Expected non-empty skill map in variant[0]"
    );
}

#[test]
fn test_druid_paragon_sorted_by_index() {
    let fixture = load_fixture("1qHh.json");
    let build = parse_build_response(fixture, "1qHh").unwrap();
    let v0 = &build.variants[0];

    for window in v0.paragon.windows(2) {
        assert!(
            window[0].index <= window[1].index,
            "Druid paragon boards not sorted: {} > {}",
            window[0].index,
            window[1].index
        );
    }
}

// ─── Network tests (ignored in CI) ───────────────────────────────────────────

#[tokio::test]
#[ignore] // Requires network — run with: cargo test -- --ignored
async fn test_live_api_call() {
    let client = diablo4_tool_lib::web_parser::D2CoreClient::new();
    let result = client.fetch_build("1QMw").await;
    assert!(result.is_ok(), "Live API call failed: {:?}", result);

    let build = result.unwrap();
    assert_eq!(build.id, "1QMw");
    assert_eq!(build.char_class, "Paladin");
}
