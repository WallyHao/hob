// --- exec::agent::schema::check ---
// The validator: type, enum, required, properties and items, reporting the
// first problem with its path. Everything else a schema may say is ignored,
// which is the honest limit of this check.

use serde_json::Value;

/// Check `value` against `schema`.
pub(crate) fn validate(schema: &Value, value: &Value) -> Result<(), String> {
    check(schema, value, "$")
}

fn check(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    if let Some(kinds) = schema.get("type")
        && !matches_kind(kinds, value)
    {
        return Err(format!(
            "{path}: expected {}, got {}",
            render(kinds),
            kind_of(value)
        ));
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array)
        && !options.contains(value)
    {
        return Err(format!("{path}: not one of the allowed values"));
    }
    match value {
        Value::Object(fields) => object(schema, fields, path),
        Value::Array(items) => {
            if let Some(sub) = schema.get("items") {
                for (index, item) in items.iter().enumerate() {
                    check(sub, item, &format!("{path}[{index}]"))?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn object(
    schema: &Value,
    fields: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), String> {
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
                check(sub, field, &format!("{path}.{name}"))?;
            }
        }
    }
    Ok(())
}

/// Whether a value satisfies a `type` that may be a name or a list of names.
fn matches_kind(kinds: &Value, value: &Value) -> bool {
    match kinds {
        Value::String(kind) => matches_name(kind, value),
        Value::Array(kinds) => kinds
            .iter()
            .filter_map(Value::as_str)
            .any(|kind| matches_name(kind, value)),
        _ => false,
    }
}

fn matches_name(kind: &str, value: &Value) -> bool {
    match kind {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn kind_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.as_i64().is_some() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn render(kinds: &Value) -> String {
    match kinds {
        Value::String(kind) => kind.clone(),
        Value::Array(kinds) => kinds
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" or "),
        _ => "a valid type".to_owned(),
    }
}
