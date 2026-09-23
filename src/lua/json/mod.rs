// --- lua::json ---
// JSON shape is explicit; effect defaults must not erase application data.

mod decode;
mod encode;
pub(crate) use decode::decode;
pub(crate) use encode::encode;

use mlua::{Lua, Table, UserData, Value};

#[derive(Debug, Clone, Copy)]
pub(super) struct Null;
impl UserData for Null {}

pub(crate) fn install(lua: &Lua, hob: &Table) -> mlua::Result<()> {
    let json = lua.create_table()?;
    let null = lua.create_userdata(Null)?;
    lua.set_named_registry_value("hob.json.null", null.clone())?;
    json.set("null", null)?;
    json.set(
        "encode",
        lua.create_function(|_, value: Value| {
            serde_json::to_string(&encode(&value)?).map_err(mlua::Error::external)
        })?,
    )?;
    json.set(
        "decode",
        lua.create_function(|lua, text: String| {
            let value = serde_json::from_str(&text).map_err(mlua::Error::external)?;
            decode(lua, &value)
        })?,
    )?;
    for kind in ["array", "object"] {
        json.set(
            kind,
            lua.create_function(move |lua, table: Option<Table>| {
                let table = table.map_or_else(|| lua.create_table(), Ok)?;
                tag(lua, &table, kind)?;
                Ok(table)
            })?,
        )?;
    }
    hob.set("json", json.clone())?;
    lua.globals()
        .get::<Table>("package")?
        .get::<Table>("preload")?
        .set(
            "hob.json",
            lua.create_function(move |_, ()| Ok(json.clone()))?,
        )
}

pub(crate) fn tag(lua: &Lua, table: &Table, kind: &str) -> mlua::Result<()> {
    let meta = lua.create_table()?;
    meta.raw_set("__hob_json", kind)?;
    meta.raw_set("__metatable", format!("hob.json.{kind}"))?;
    table.set_metatable(Some(meta))
}

pub(super) fn kind(table: &Table) -> mlua::Result<Option<String>> {
    table
        .metatable()
        .map_or(Ok(None), |meta| meta.raw_get("__hob_json"))
}
