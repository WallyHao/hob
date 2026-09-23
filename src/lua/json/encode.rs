// --- lua::json::encode ---
// Bound recursion and preserve explicit shapes instead of dropping empty fields.

use super::{Null, kind};
use mlua::{Error, Table, Value as LuaValue};
use serde_json::{Map, Value};

pub(crate) fn encode(value: &LuaValue) -> mlua::Result<Value> {
    convert(value, 0)
}

fn convert(value: &LuaValue, depth: usize) -> mlua::Result<Value> {
    if depth > 64 {
        return Err(error("cyclic or too deeply nested JSON value"));
    }
    Ok(match value {
        LuaValue::Nil => Value::Null,
        LuaValue::UserData(value) if value.is::<Null>() => Value::Null,
        LuaValue::Boolean(flag) => Value::Bool(*flag),
        LuaValue::Integer(number) => Value::from(*number),
        LuaValue::Number(number) if number.is_finite() => Value::from(*number),
        LuaValue::String(text) => Value::String(text.to_str()?.to_owned()),
        LuaValue::Table(table) => table_value(table, depth + 1)?,
        other => {
            return Err(error(&format!(
                "unsupported JSON value: {}",
                other.type_name()
            )));
        }
    })
}

fn table_value(table: &Table, depth: usize) -> mlua::Result<Value> {
    let entries: Vec<_> = table
        .clone()
        .pairs::<LuaValue, LuaValue>()
        .collect::<mlua::Result<_>>()?;
    let kind = kind(table)?;
    let sequence = !entries.is_empty() && entries.iter().all(|(key, _)| {
        matches!(key, LuaValue::Integer(index) if usize::try_from(*index).is_ok_and(|i| i > 0 && i <= entries.len()))
    });
    if kind.as_deref() == Some("array") || (kind.is_none() && sequence) {
        if !entries.is_empty() && !sequence {
            return Err(error("JSON array keys must be exactly 1..n"));
        }
        let mut values = Vec::with_capacity(entries.len());
        for index in 1..=entries.len() {
            values.push(convert(&table.raw_get::<LuaValue>(index)?, depth)?);
        }
        return Ok(Value::Array(values));
    }
    let mut object = Map::new();
    for (key, value) in entries {
        let key = match key {
            LuaValue::String(text) => text.to_str()?.to_owned(),
            LuaValue::Integer(number) if kind.is_none() => number.to_string(),
            _ => return Err(error("JSON object keys must be strings")),
        };
        if object.insert(key, convert(&value, depth)?).is_some() {
            return Err(error("JSON object keys collide after conversion"));
        }
    }
    Ok(Value::Object(object))
}

fn error(message: &str) -> Error {
    Error::RuntimeError(message.to_owned())
}
