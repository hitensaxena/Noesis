use regex::Regex;
use std::sync::LazyLock;

/// Regex to strip markdown code fences from LLM responses.
static FENCE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^\s*```(?:json)?\s*|\s*```\s*$").unwrap());

/// Regex to find JSON objects in text.
static JSON_OBJECT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{(?:[^{}]|(?:\{[^{}]*\}))*\}").unwrap());

/// Regex to find JSON arrays in text.
static JSON_ARRAY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(?:[^\[\]]|(?:\[[^\[\]]*\]))*\]").unwrap());

/// Strip markdown code fences from an LLM response.
///
/// Handles ```json, ```, and other fence variants.
pub fn strip_fences(s: &str) -> String {
    let s = s.trim();
    let result = FENCE_REGEX.replace_all(s, "").to_string();
    result.trim().to_string()
}

/// Find the first balanced JSON value in a string, fence/prose tolerant.
///
/// Returns `None` if no valid JSON can be extracted.
pub fn first_json(content: &str) -> Option<serde_json::Value> {
    let cleaned = strip_fences(content);
    if cleaned.is_empty() {
        return None;
    }

    // Try full string first
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&cleaned) {
        return Some(v);
    }

    // Try finding a JSON object
    if let Some(mat) = JSON_OBJECT.find(&cleaned) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(mat.as_str()) {
            return Some(v);
        }
    }

    // Try finding a JSON array
    if let Some(mat) = JSON_ARRAY.find(&cleaned) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(mat.as_str()) {
            return Some(v);
        }
    }

    // Try first balanced bracket
    first_balanced(&cleaned)
}

/// Parse JSON records from an LLM response.
///
/// Returns an iterator of successfully-parsed JSON values from a response
/// that may contain prose, fences, or partial output. Handles the common
/// case of an LLM wrapping JSON in ``` fences or prepending text.
pub fn json_records(content: &str) -> Vec<serde_json::Value> {
    let cleaned = strip_fences(content);
    if cleaned.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();

    // Try parsing as a complete JSON value
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&cleaned) {
        match &v {
            serde_json::Value::Array(arr) => {
                results.extend(arr.iter().cloned());
                return results;
            }
            serde_json::Value::Object(_) => {
                results.push(v);
                return results;
            }
            _ => {}
        }
    }

    // Try extracting each JSON object from the text
    for mat in JSON_OBJECT.find_iter(&cleaned) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(mat.as_str()) {
            results.push(v);
        }
    }

    // If no objects, try arrays
    if results.is_empty() {
        for mat in JSON_ARRAY.find_iter(&cleaned) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(mat.as_str()) {
                match &v {
                    serde_json::Value::Array(arr) => {
                        results.extend(arr.iter().cloned());
                    }
                    _ => results.push(v),
                }
            }
        }
    }

    results
}

/// Find the first balanced `{...}` or `[...]` substring, string/escape aware.
fn first_balanced(s: &str) -> Option<serde_json::Value> {
    let chars: Vec<char> = s.chars().collect();
    let mut start: Option<usize> = None;
    let mut open_ch: char = '{';
    let mut close_ch: char = '}';
    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut esc = false;

    for (i, &ch) in chars.iter().enumerate() {
        if start.is_none() {
            if ch == '{' || ch == '[' {
                start = Some(i);
                open_ch = ch;
                close_ch = if ch == '{' { '}' } else { ']' };
                depth = 1;
            }
            continue;
        }
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        if ch == '"' {
            in_str = true;
        } else if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth -= 1;
            if depth == 0 {
                let candidate: String = chars[start.unwrap()..=i].iter().collect();
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&candidate) {
                    return Some(v);
                }
                start = None;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_fences_basic() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_fences(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_strip_fences_no_fence() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(strip_fences(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_first_json_with_prose() {
        let input = "Here is the result:\n\n```json\n{\"name\": \"test\", \"count\": 42}\n```\n\nHope that helps!";
        let result = first_json(input).unwrap();
        assert_eq!(result["name"], "test");
        assert_eq!(result["count"], 42);
    }

    #[test]
    fn test_json_records_array() {
        let input = r#"{"results":[{"id":"1","type":"canonical"},{"id":"2","type":"belief"}]}"#;
        let records = json_records(input);
        // Returns the outer object as one record
        assert_eq!(records.len(), 1);
        assert!(records[0].is_object());
    }

    #[test]
    fn test_json_records_flat_array() {
        let input = r#"[{"id":"1","type":"canonical"},{"id":"2","type":"belief"}]"#;
        let records = json_records(input);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_first_balanced_nested() {
        let input = r#"Some text {"outer": {"inner": "value"}} more text"#;
        let result = first_json(input).unwrap();
        assert_eq!(result["outer"]["inner"], "value");
    }
}
