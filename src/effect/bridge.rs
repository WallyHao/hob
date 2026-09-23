// --- effect::bridge ---
// Effect results retain nil for absent fields; public JSON uses explicit null.

use mlua::{Lua, Value as LuaValue};
use serde_json::Value;

pub(crate) use crate::lua::json::encode as lua_to_json;

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
            crate::lua::json::tag(lua, &table, "array")?;
            for (index, item) in items.iter().enumerate() {
                table.raw_set(index + 1, json_to_lua(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(fields) => {
            let table = lua.create_table_with_capacity(0, fields.len())?;
            crate::lua::json::tag(lua, &table, "object")?;
            for (key, item) in fields {
                table.raw_set(key.as_str(), json_to_lua(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
    })
}
