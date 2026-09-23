// --- lua::json::decode ---
// Null and empty containers must survive a round trip through Lua.

use super::tag;
use mlua::{Lua, Value as LuaValue};
use serde_json::Value;

pub(crate) fn decode(lua: &Lua, value: &Value) -> mlua::Result<LuaValue> {
    Ok(match value {
        Value::Null => LuaValue::UserData(lua.named_registry_value("hob.json.null")?),
        Value::Bool(flag) => LuaValue::Boolean(*flag),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                LuaValue::Integer(integer)
            } else if number.is_u64() {
                return Err(mlua::Error::RuntimeError(
                    "JSON integer exceeds Lua's signed integer range".to_owned(),
                ));
            } else {
                LuaValue::Number(number.as_f64().unwrap_or_default())
            }
        }
        Value::String(text) => LuaValue::String(lua.create_string(text)?),
        Value::Array(items) => {
            let table = lua.create_table_with_capacity(items.len(), 0)?;
            tag(lua, &table, "array")?;
            for (index, item) in items.iter().enumerate() {
                table.raw_set(index + 1, decode(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
        Value::Object(fields) => {
            let table = lua.create_table_with_capacity(0, fields.len())?;
            tag(lua, &table, "object")?;
            for (key, item) in fields {
                table.raw_set(key.as_str(), decode(lua, item)?)?;
            }
            LuaValue::Table(table)
        }
    })
}
