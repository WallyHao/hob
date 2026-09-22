// --- effect::bridge ---
// Translating between Lua values and JSON.
//
// Both directions are written by hand rather than going through mlua's serde
// bridge, because that bridge makes choices a flow author would not expect: it
// maps JSON `null` to a light userdata rather than `nil`, and it refuses Lua
// tables whose keys are not all of one kind. Doing it here means `if x == nil`
// works and the two directions are exact inverses.
//
// An empty Lua table has no shape -- `{}` is how both "no arguments" and "an
// empty list" are written -- so it is encoded as `null`, and a field whose
// value is an empty table is left out of its record entirely. Absent fields
// are what serde's defaults are for: the flow did not say whether it meant a
// list or a record.

use mlua::{Lua, Value as LuaValue};
use serde_json::{Map, Value};

/// Convert a Lua value into JSON.
///
/// A table becomes an array only when its keys are exactly `1..n`, an object
/// otherwise, and `null` when it is empty; anything else is an error.
pub(crate) fn lua_to_json(value: &LuaValue) -> mlua::Result<Value> {
    Ok(match value {
        LuaValue::Nil => Value::Null,
        LuaValue::Boolean(flag) => Value::Bool(*flag),
        LuaValue::Integer(number) => Value::from(*number),
        LuaValue::Number(number) => Value::from(*number),
        LuaValue::String(text) => Value::String(text.to_string_lossy().clone()),
        LuaValue::Table(table) => table_to_json(table)?,
        other => {
            return Err(mlua::Error::RuntimeError(format!(
                "unsupported value from Lua: {}",
                other.type_name()
            )));
        }
    })
}

/// Convert JSON into Lua values, turning `null` into `nil`.
pub(crate) fn json_to_lua(lua: &Lua, value: &Value) -> mlua::Result<LuaValue> {
    Ok(match value {
        Value::Null => LuaValue::Nil,
        Value::Bool(flag) => LuaValue::Boolean(*flag),
        // Lua 5.4 has a real integer subtype, so whole numbers stay integral
        // and do not print as `1.0` when a flow echoes them back.
        Value::Number(number) => match number.as_i64() {
            Some(integer) => LuaValue::Integer(integer),
            None => LuaValue::Number(number.as_f64().unwrap_or_default()),
        },
        Value::String(text) => LuaValue::String(lua.create_string(text)?),
        Value::Array(items) => {
            let table = lua.create_table_with_capacity(items.len(), 0)?;
            for (index, item) in items.iter().enumerate() {
                table.raw_set(index + 1, json_to_lua(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(fields) => {
            let table = lua.create_table_with_capacity(0, fields.len())?;
            for (key, item) in fields {
                table.raw_set(key.as_str(), json_to_lua(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
    })
}

fn table_to_json(table: &mlua::Table) -> mlua::Result<Value> {
    let mut entries = Vec::new();
    for pair in table.clone().pairs::<LuaValue, LuaValue>() {
        entries.push(pair?);
    }
    if entries.is_empty() {
        return Ok(Value::Null);
    }
    let is_sequence = entries.iter().enumerate().all(|(offset, (key, _))| {
        matches!(
            key,
            LuaValue::Integer(number)
                if i64::try_from(offset).is_ok_and(|index| *number == index + 1)
        )
    });
    if is_sequence {
        return entries
            .into_iter()
            .map(|(_, value)| lua_to_json(&value))
            .collect::<mlua::Result<Vec<_>>>()
            .map(Value::Array);
    }
    let mut mapping = Map::new();
    for (key, value) in entries {
        let key = match key {
            LuaValue::String(text) => text.to_string_lossy().clone(),
            LuaValue::Integer(number) => number.to_string(),
            _ => {
                return Err(mlua::Error::RuntimeError(
                    "table keys must be strings or integers".to_owned(),
                ));
            }
        };
        let encoded = lua_to_json(&value)?;
        // An empty table is the only thing that encodes as `null` here, and
        // dropping the key lets the target field's default speak for it.
        if !encoded.is_null() {
            mapping.insert(key, encoded);
        }
    }
    Ok(Value::Object(mapping))
}

#[cfg(test)]
mod tests {
    use super::lua_to_json;
    use crate::lua::new_vm;
    use serde_json::json;

    fn encode(source: &str) -> serde_json::Value {
        let lua = new_vm().expect("the VM builds");
        let value: mlua::Value = lua.load(source).eval().expect("the chunk evaluates");
        lua_to_json(&value).expect("the value encodes")
    }

    #[test]
    fn an_empty_table_is_encoded_as_absent() {
        // `{}` says nothing about whether a list or a record was meant, so the
        // field's own default is what should decide.
        assert_eq!(encode("return {}"), json!(null));
        assert_eq!(encode("return { text = 'hi' }"), json!({ "text": "hi" }));
    }

    #[test]
    fn a_sequence_is_an_array_and_an_object_is_an_object() {
        assert_eq!(encode("return { 'a', 'b' }"), json!(["a", "b"]));
        assert_eq!(encode("return { a = 1 }"), json!({ "a": 1 }));
    }
}
