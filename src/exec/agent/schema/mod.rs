// --- exec::agent::schema ---
// The JSON-Schema subset a model is held to, and the words used to ask again.
// The validator itself lives in `check`; this module owns the prompts and the
// parsing of an answer that may arrive fenced.

mod check;
mod example;

use serde_json::Value;

pub(crate) use check::validate;
pub(crate) use example::example;

/// The instruction that asks for JSON of the given shape.
pub(crate) fn instruction(schema: &Value) -> String {
    format!(
        "Answer with a single JSON value that matches this schema, with no prose and no code \
         fences:\n{schema}"
    )
}

/// The message that asks again after a malformed answer.
pub(crate) fn repair(error: &str, schema: &Value) -> String {
    format!(
        "That answer was not valid: {error}. Answer again with JSON only, matching the \
         schema:\n{schema}"
    )
}

/// Parse the answer as JSON, tolerating a fenced block.
pub(crate) fn extract(text: &str) -> Result<Value, String> {
    let trimmed = text.trim();
    let body = strip_fence(trimmed).unwrap_or(trimmed);
    serde_json::from_str(body).map_err(|error| format!("not JSON: {error}"))
}

/// The body of a fenced code block, when the answer wrapped the JSON in one.
fn strip_fence(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("```")?;
    let rest = rest.split_once('\n')?.1;
    rest.strip_suffix("```").map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::{extract, validate};
    use serde_json::json;

    #[test]
    fn fences_are_tolerated() {
        let value = extract("```json\n{\"ok\": true}\n```").expect("fenced JSON parses");
        assert_eq!(value, json!({ "ok": true }));
    }

    #[test]
    fn prose_is_not_json() {
        assert!(extract("sure, here you go").is_err());
    }

    #[test]
    fn a_shape_is_checked_with_a_path() {
        let schema = json!({
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "type": "string" }, "n": { "type": "integer" } }
        });
        assert!(validate(&schema, &json!({ "name": "a", "n": 1 })).is_ok());
        let missing = validate(&schema, &json!({ "n": 1 })).expect_err("required");
        assert!(missing.contains("name"), "{missing}");
        let wrong = validate(&schema, &json!({ "name": "a", "n": "x" })).expect_err("type");
        assert!(wrong.contains("$.n"), "{wrong}");
    }

    #[test]
    fn a_list_of_types_is_accepted() {
        let schema =
            json!({ "type": "object", "properties": { "x": { "type": ["string", "null"] } } });
        assert!(validate(&schema, &json!({ "x": null })).is_ok());
        assert!(validate(&schema, &json!({ "x": 1 })).is_err());
    }
}
