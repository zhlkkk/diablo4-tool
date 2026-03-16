use regex::Regex;

use super::error::ParserError;

/// Extract the build ID from a full d2core.com URL or accept a raw alphanumeric ID.
///
/// - Full URL: `https://d2core.com/d4/planner?bd=1QMw` → `"1QMw"`
/// - Raw ID: `"1QMw"` → `"1QMw"`
/// - Invalid input → `ParserError::InvalidUrl`
pub fn extract_build_id(input: &str) -> Result<String, ParserError> {
    let input = input.trim();

    // Try to extract bd= parameter from a URL
    let url_re = Regex::new(r"bd=([A-Za-z0-9]+)").expect("regex is valid");
    if let Some(caps) = url_re.captures(input) {
        return Ok(caps[1].to_string());
    }

    // Accept raw alphanumeric ID (2-10 chars)
    let raw_re = Regex::new(r"^[A-Za-z0-9]{2,10}$").expect("regex is valid");
    if raw_re.is_match(input) {
        return Ok(input.to_string());
    }

    Err(ParserError::InvalidUrl(input.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_full_url() {
        let result = extract_build_id("https://d2core.com/d4/planner?bd=1QMw");
        assert_eq!(result.unwrap(), "1QMw");
    }

    #[test]
    fn test_extract_from_raw_id() {
        let result = extract_build_id("1QMw");
        assert_eq!(result.unwrap(), "1QMw");
    }

    #[test]
    fn test_extract_raw_id_with_whitespace() {
        let result = extract_build_id("  1QMw  ");
        assert_eq!(result.unwrap(), "1QMw");
    }

    #[test]
    fn test_extract_invalid_input() {
        let result = extract_build_id("not-a-valid-id!@#");
        assert!(matches!(result, Err(ParserError::InvalidUrl(_))));
    }

    #[test]
    fn test_extract_from_url_with_other_params() {
        let result = extract_build_id("https://d2core.com/d4/planner?other=123&bd=1qHh&foo=bar");
        assert_eq!(result.unwrap(), "1qHh");
    }
}
