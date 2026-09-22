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
        Some("array") => Value::Array(vec![example(&schema["items"])]),
        // A visible placeholder rather than an empty string: an empty answer
        // makes a preview read as if the flow were broken.
        Some("string") => Value::String(String::from("mock")),
        Some("integer") => Value::from(0),
        Some("number") => Value::from(0.0),
        Some("boolean") => Value::Bool(false),
        _ => Value::Null,
    }
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
