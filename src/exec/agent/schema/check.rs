// --- exec::agent::schema::check ---
// The validator. The keywords enforced here are: `type`, `const`, `enum`,
// `allOf`, `anyOf`, `oneOf`, `required`, `properties`, `additionalProperties`,
// `items`, `minItems`, `maxItems`, `minimum`, `maximum`, `exclusiveMinimum`,
// `exclusiveMaximum`, `minLength` and `maxLength`.
//
// Everything else a schema may say is ignored rather than refused, which is the
// honest limit of this check: `pattern`, `format` and tuple-form `items` are
// not implemented, and a schema that uses them is weaker than it looks.

use serde_json::Value;

mod object;
mod rules;

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
    if let Some(constant) = schema.get("const")
        && value != constant
    {
        return Err(format!("{path}: expected the constant {constant}"));
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array)
        && !options.contains(value)
    {
        return Err(format!("{path}: not one of the allowed values"));
    }
    branches(schema, value, path)?;
    rules::check(schema, value, path)?;
    if let Value::Object(fields) = value {
        object::check(schema, fields, path)?;
    }
    Ok(())
}

/// The branch keywords: every `allOf` must pass, one `anyOf` must, and
/// `oneOf` must match exactly once.
fn branches(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
        for branch in all {
            check(branch, value, path)?;
        }
    }
    if let Some(any) = schema.get("anyOf").and_then(Value::as_array)
        && !any.iter().any(|branch| check(branch, value, path).is_ok())
    {
        return Err(format!("{path}: matches none of the `anyOf` branches"));
    }
    if let Some(one) = schema.get("oneOf").and_then(Value::as_array) {
        let matches = one
            .iter()
            .filter(|branch| check(branch, value, path).is_ok())
            .count();
        if matches != 1 {
            return Err(format!(
                "{path}: matches {matches} of the `oneOf` branches, expected exactly one"
            ));
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
        // An unknown type name is a schema bug; passing everything would hide
        // it the way a typo in a required field would not.
        _ => false,
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
