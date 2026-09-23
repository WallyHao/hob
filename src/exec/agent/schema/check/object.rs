// --- exec::agent::schema::check::object ---
// The object keywords: what must be there, what is checked, what is allowed.

use serde_json::{Map, Value};

// Recursion goes through the parent, which owns the entry point.

/// Apply `required`, `properties` and `additionalProperties`.
pub(super) fn check(schema: &Value, fields: &Map<String, Value>, path: &str) -> Result<(), String> {
    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        for name in required.iter().filter_map(Value::as_str) {
            if !fields.contains_key(name) {
                return Err(format!("{path}: missing required field `{name}`"));
            }
        }
    }
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for (name, sub) in properties {
            if let Some(field) = fields.get(name) {
                super::check(sub, field, &format!("{path}.{name}"))?;
            }
        }
    }
    additional(schema, fields, path)
}

/// `additionalProperties` closes the shape (`false`) or holds a schema for the
/// fields `properties` did not name. Absent or `true` allows anything, which is
/// the default in JSON Schema.
fn additional(schema: &Value, fields: &Map<String, Value>, path: &str) -> Result<(), String> {
    let declared = schema.get("properties").and_then(Value::as_object);
    let known = |name: &str| declared.is_some_and(|properties| properties.contains_key(name));
    match schema.get("additionalProperties") {
        Some(Value::Bool(false)) => {
            for name in fields.keys() {
                if !known(name) {
                    return Err(format!("{path}: unexpected field `{name}`"));
                }
            }
        }
        Some(sub @ Value::Object(_)) => {
            for (name, value) in fields {
                if !known(name) {
                    super::check(sub, value, &format!("{path}.{name}"))?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
