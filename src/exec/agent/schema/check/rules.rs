// --- exec::agent::schema::check::rules ---
// The scalar and array keywords: ranges, lengths and item recursion.

use serde_json::Value;

// Recursion goes through the parent, which owns the entry point.

/// Apply the keywords a value's kind makes meaningful.
pub(super) fn check(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    number(schema, value, path)?;
    string(schema, value, path)?;
    array(schema, value, path)
}

fn number(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Some(number) = value.as_f64() else {
        return Ok(());
    };
    let bound = |name: &str| schema.get(name).and_then(Value::as_f64);
    if let Some(min) = bound("minimum")
        && number < min
    {
        return Err(format!("{path}: {number} is below the minimum {min}"));
    }
    if let Some(max) = bound("maximum")
        && number > max
    {
        return Err(format!("{path}: {number} is above the maximum {max}"));
    }
    if let Some(min) = bound("exclusiveMinimum")
        && number <= min
    {
        return Err(format!(
            "{path}: {number} is not above the exclusive minimum {min}"
        ));
    }
    if let Some(max) = bound("exclusiveMaximum")
        && number >= max
    {
        return Err(format!(
            "{path}: {number} is not below the exclusive maximum {max}"
        ));
    }
    Ok(())
}

fn string(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Some(text) = value.as_str() else {
        return Ok(());
    };
    let length = u64::try_from(text.chars().count()).unwrap_or(u64::MAX);
    if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
        && length < min
    {
        return Err(format!("{path}: shorter than the minimum {min} characters"));
    }
    if let Some(max) = schema.get("maxLength").and_then(Value::as_u64)
        && length > max
    {
        return Err(format!("{path}: longer than the maximum {max} characters"));
    }
    Ok(())
}

fn array(schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let Value::Array(items) = value else {
        return Ok(());
    };
    let count = u64::try_from(items.len()).unwrap_or(u64::MAX);
    if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
        && count < min
    {
        return Err(format!("{path}: fewer than the minimum {min} items"));
    }
    if let Some(max) = schema.get("maxItems").and_then(Value::as_u64)
        && count > max
    {
        return Err(format!("{path}: more than the maximum {max} items"));
    }
    if let Some(sub) = schema.get("items") {
        for (index, item) in items.iter().enumerate() {
            super::check(sub, item, &format!("{path}[{index}]"))?;
        }
    }
    Ok(())
}
