// --- exec::agent::schema::example ---
// Synthesizing a smallest-valid instance from a schema.
//
// A preview refuses to call the model, but the rest of the flow still has to
// run: a schema-shaped example stands in for the answer so the flow has
// something to index instead of nil, and the preview shows what it would do.
// The value is deliberately obviously synthetic, so nobody mistakes a preview
// for a real answer.

use serde_json::{Map, Value, json};

/// Build a minimal instance that satisfies a schema.
pub(crate) fn example(schema: &Value) -> Value {
    if let Some(constant) = schema.get("const") {
        return constant.clone();
    }
    if let Some(options) = schema["enum"]
        .as_array()
        .filter(|options| !options.is_empty())
    {
        return options[0].clone();
    }
    for keyword in ["oneOf", "anyOf"] {
        if let Some(branch) = schema[keyword].as_array().and_then(|list| list.first()) {
            return example(branch);
        }
    }
    match schema["type"].as_str() {
        Some("object") if schema.get("properties").is_some() => object(schema),
        Some("array") => {
            let count = schema.get("minItems").and_then(Value::as_u64).unwrap_or(1);
            let items = std::iter::repeat_with(|| example(&schema["items"]))
                .take(usize::try_from(count).unwrap_or(usize::MAX))
                .collect();
            Value::Array(items)
        }
        // A visible placeholder rather than an empty string: an empty answer
        // makes a preview read as if the flow were broken.
        Some("string") => Value::String(pad(schema)),
        Some("integer") => Value::from(schema.get("minimum").and_then(Value::as_i64).unwrap_or(0)),
        Some("number") => Value::from(schema.get("minimum").and_then(Value::as_f64).unwrap_or(0.0)),
        Some("boolean") => Value::Bool(false),
        _ => Value::Null,
    }
}

/// The placeholder, grown to whatever `minLength` demands.
fn pad(schema: &Value) -> String {
    let minimum = schema.get("minLength").and_then(Value::as_u64).unwrap_or(0);
    let minimum = usize::try_from(minimum).unwrap_or(usize::MAX);
    let mut text = String::from("mock");
    while text.chars().count() < minimum {
        text.push('x');
    }
    text
}

fn object(schema: &Value) -> Value {
    let Some(properties) = schema["properties"].as_object() else {
        return json!({});
    };
    let required = schema["required"].as_array();
    let mut instance = Map::new();
    for (name, property) in properties {
        // With no `required` list every property is filled, so the example is
        // representative of the whole shape the flow has to handle.
        let needed = required.is_none_or(|required| {
            required
                .iter()
                .any(|entry| entry.as_str() == Some(name.as_str()))
        });
        if needed {
            instance.insert(name.clone(), example(property));
        }
    }
    Value::Object(instance)
}

#[cfg(test)]
mod tests {
    use super::example;
    use serde_json::json;

    #[test]
    fn an_enum_answers_with_its_first_value() {
        let schema = json!({ "type": "string", "enum": ["feat", "fix"] });
        assert_eq!(example(&schema), json!("feat"));
    }

    #[test]
    fn an_object_fills_what_is_required() {
        let schema = json!({
            "type": "object",
            "required": ["name"],
            "properties": { "name": { "type": "string" }, "note": { "type": "string" } }
        });
        assert_eq!(example(&schema), json!({ "name": "mock" }));
    }
}
