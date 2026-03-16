use std::collections::HashMap;

use serde_json::Value;

use crate::types::{BuildPlan, EquipSkill, ParagonBoard, Variant};

use super::error::ParserError;

/// Parse the raw TCB API response into a typed BuildPlan.
///
/// The API uses double-deserialization:
/// 1. Outer JSON: `{ "data": { "response_data": "<json string>" } }`
/// 2. Inner JSON: `{ "data": { "_id": ..., "char": ..., "variants": [...] } }`
pub fn parse_build_response(resp: Value, build_id: &str) -> Result<BuildPlan, ParserError> {
    // Step 1: Extract outer response_data string
    let response_data_str = resp
        .get("data")
        .and_then(|d| d.get("response_data"))
        .and_then(|r| r.as_str())
        .ok_or_else(|| {
            ParserError::ParseError(format!(
                "missing data.response_data in API response for build {}",
                build_id
            ))
        })?;

    // Step 2: Parse the inner JSON string
    let inner: Value = serde_json::from_str(response_data_str).map_err(|e| {
        ParserError::ParseError(format!(
            "failed to parse response_data as JSON for build {}: {}",
            build_id, e
        ))
    })?;

    // Step 3: Extract inner data object
    // If the API returns an errMsg (e.g. "数据不存在"), the build was not found
    if let Some(err_msg) = inner.get("errMsg").and_then(|e| e.as_str()) {
        return Err(ParserError::BuildNotFound(format!(
            "{}: {}",
            build_id, err_msg
        )));
    }

    let data = inner.get("data").ok_or_else(|| {
        ParserError::ParseError(format!(
            "missing 'data' field in parsed response for build {}",
            build_id
        ))
    })?;

    // If data is null or has no _id, the build doesn't exist
    if data.is_null() || data.get("_id").is_none() {
        return Err(ParserError::BuildNotFound(build_id.to_string()));
    }

    // Map top-level fields
    let id = data["_id"]
        .as_str()
        .unwrap_or(build_id)
        .to_string();

    let char_class = data["char"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let title = data["title"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Parse variants array
    let variants_json = data["variants"].as_array().ok_or_else(|| {
        ParserError::ParseError(format!(
            "missing or non-array 'variants' for build {}",
            build_id
        ))
    })?;

    let variants: Vec<Variant> = variants_json
        .iter()
        .map(|v| parse_variant(v, build_id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(BuildPlan {
        id,
        char_class,
        title,
        variants,
    })
}

fn parse_variant(v: &Value, build_id: &str) -> Result<Variant, ParserError> {
    let name = v["name"].as_str().unwrap_or("").to_string();

    // Parse skill HashMap<u32, u32>, filtering out zero-value entries
    let skill = parse_skill_map(&v["skill"]);

    // Parse skillOrder as Vec<u32>
    let skill_order: Vec<u32> = v["skillOrder"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default();

    // Parse equipSkills
    let equip_skills: Vec<EquipSkill> = v["equipSkills"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|es| parse_equip_skill(es))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Parse paragon: object keyed by board name → convert to sorted Vec<ParagonBoard>
    let paragon = parse_paragon(&v["paragon"], build_id)?;

    Ok(Variant {
        name,
        skill,
        skill_order,
        equip_skills,
        paragon,
    })
}

/// Parse skill object: {"47": 3, "6": 0, ...} → HashMap<u32, u32> with zero values filtered out
fn parse_skill_map(val: &Value) -> HashMap<u32, u32> {
    let mut map = HashMap::new();
    if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            if let (Ok(skill_id), Some(points)) = (k.parse::<u32>(), v.as_u64()) {
                let points = points as u32;
                if points != 0 {
                    map.insert(skill_id, points);
                }
            }
        }
    }
    map
}

fn parse_equip_skill(es: &Value) -> EquipSkill {
    let key = es["key"].as_str().unwrap_or("").to_string();
    let mods: Vec<String> = es["mods"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let rank = es["rank"].as_u64().unwrap_or(0) as u32;
    EquipSkill { key, mods, rank }
}

/// Parse paragon: {"Board_Name": {data, glyph, index, rotate}} → sorted Vec<ParagonBoard>
fn parse_paragon(val: &Value, _build_id: &str) -> Result<Vec<ParagonBoard>, ParserError> {
    let mut boards: Vec<ParagonBoard> = Vec::new();

    if let Some(obj) = val.as_object() {
        for (name, board_data) in obj {
            let index = board_data["index"].as_u64().unwrap_or(0) as u32;
            let rotate = board_data["rotate"].as_u64().unwrap_or(0) as u32;
            let nodes: Vec<String> = board_data["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|n| n.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let glyph = extract_glyph(&board_data["glyph"]);

            boards.push(ParagonBoard {
                name: name.clone(),
                index,
                rotate,
                nodes,
                glyph,
            });
        }
    }

    // Sort by index ascending
    boards.sort_by_key(|b| b.index);

    Ok(boards)
}

/// Extract glyph from API's {"0": "glyph_name"} object → Option<String>
/// If object is empty, null, or missing, returns None.
fn extract_glyph(val: &Value) -> Option<String> {
    if let Some(obj) = val.as_object() {
        // Extract first value from the object (key is always "0")
        obj.values()
            .next()
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_skill_zero_filtered() {
        let val = json!({"6": 0, "47": 3, "49": 1});
        let map = parse_skill_map(&val);
        assert!(!map.contains_key(&6));
        assert_eq!(map[&47], 3);
        assert_eq!(map[&49], 1);
    }

    #[test]
    fn test_extract_glyph_present() {
        let val = json!({"0": "Exploit"});
        assert_eq!(extract_glyph(&val), Some("Exploit".to_string()));
    }

    #[test]
    fn test_extract_glyph_empty_object() {
        let val = json!({});
        assert_eq!(extract_glyph(&val), None);
    }

    #[test]
    fn test_extract_glyph_null() {
        let val = json!(null);
        assert_eq!(extract_glyph(&val), None);
    }

    #[test]
    fn test_paragon_sorted_by_index() {
        let val = json!({
            "BoardC": {"index": 2, "rotate": 0, "data": [], "glyph": {}},
            "BoardA": {"index": 0, "rotate": 1, "data": [], "glyph": {}},
            "BoardB": {"index": 1, "rotate": 0, "data": [], "glyph": {}}
        });
        let boards = parse_paragon(&val, "test").unwrap();
        assert_eq!(boards[0].index, 0);
        assert_eq!(boards[1].index, 1);
        assert_eq!(boards[2].index, 2);
    }

    #[test]
    fn test_build_not_found_when_data_null() {
        let resp = json!({
            "data": {
                "response_data": "{\"data\": null}"
            }
        });
        let result = parse_build_response(resp, "abc");
        assert!(matches!(result, Err(ParserError::BuildNotFound(_))));
    }
}
