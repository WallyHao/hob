// --- exec::agent::schema::tests ---
// The subset a model is held to, keyword by keyword.

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
    let schema = json!({ "type": "object", "properties": { "x": { "type": ["string", "null"] } } });
    assert!(validate(&schema, &json!({ "x": null })).is_ok());
    assert!(validate(&schema, &json!({ "x": 1 })).is_err());
}

#[test]
fn an_unknown_type_is_refused() {
    let typo = json!({ "type": "sting" });
    let error = validate(&typo, &json!("x")).expect_err("a typo must not pass");
    assert!(error.contains("sting"), "{error}");
}

#[test]
fn ranges_lengths_and_constants_are_enforced() {
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "slug": { "type": "string", "minLength": 3, "maxLength": 8 },
            "count": { "type": "integer", "minimum": 1, "exclusiveMaximum": 5 },
            "kind": { "const": "task" },
        },
    });
    assert!(
        validate(
            &schema,
            &json!({ "slug": "abc", "count": 4, "kind": "task" })
        )
        .is_ok()
    );
    assert!(
        validate(&schema, &json!({ "slug": "ab" }))
            .unwrap_err()
            .contains("minimum 3 characters")
    );
    assert!(
        validate(&schema, &json!({ "count": 0 }))
            .unwrap_err()
            .contains("minimum")
    );
    assert!(
        validate(&schema, &json!({ "count": 5 }))
            .unwrap_err()
            .contains("exclusive maximum")
    );
    assert!(
        validate(&schema, &json!({ "kind": "note" }))
            .unwrap_err()
            .contains("constant")
    );
    assert!(
        validate(&schema, &json!({ "extra": 1 }))
            .unwrap_err()
            .contains("unexpected field")
    );
}

#[test]
fn branches_must_match_as_declared() {
    let one = json!({ "oneOf": [{ "type": "string" }, { "type": "integer" }] });
    assert!(validate(&one, &json!("x")).is_ok());
    assert!(validate(&one, &json!(1)).is_ok());
    assert!(validate(&one, &json!(true)).is_err());

    let any = json!({ "anyOf": [{ "type": "string", "minLength": 3 }, { "type": "integer" }] });
    assert!(validate(&any, &json!(1)).is_ok());
    assert!(validate(&any, &json!("abc")).is_ok());
    assert!(validate(&any, &json!("a")).is_err());

    let all = json!({ "allOf": [{ "type": "string" }, { "minLength": 2 }] });
    assert!(validate(&all, &json!("ab")).is_ok());
    assert!(validate(&all, &json!("a")).is_err());
}

#[test]
fn item_counts_are_enforced() {
    let schema =
        json!({ "type": "array", "minItems": 1, "maxItems": 2, "items": { "type": "integer" } });
    assert!(validate(&schema, &json!([1])).is_ok());
    assert!(validate(&schema, &json!([])).unwrap_err().contains("fewer"));
    assert!(
        validate(&schema, &json!([1, 2, 3]))
            .unwrap_err()
            .contains("more")
    );
    assert!(
        validate(&schema, &json!(["x"]))
            .unwrap_err()
            .contains("expected integer")
    );
}
