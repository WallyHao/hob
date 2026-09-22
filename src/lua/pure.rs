// --- lua::pure ---
// Pure helpers bound directly, with no effect round trip: nothing here touches
// the world, so trace and preview have nothing to say about it.

use mlua::{Lua, Table, Value};

use crate::effect::bridge;

/// Attach `hob.json` to the module table and preload it under that name.
pub(crate) fn install(lua: &Lua, hob: &Table) -> mlua::Result<()> {
    let json = lua.create_table()?;
    json.set(
        "encode",
        lua.create_function(|_, value: Value| {
            let encoded = bridge::lua_to_json(&value)?;
            serde_json::to_string(&encoded).map_err(mlua::Error::external)
        })?,
    )?;
    json.set(
        "decode",
        lua.create_function(|lua, text: String| {
            let decoded: serde_json::Value =
                serde_json::from_str(&text).map_err(mlua::Error::external)?;
            bridge::json_to_lua(lua, &decoded)
        })?,
    )?;
    hob.set("json", json.clone())?;
    let loader = json;
    lua.globals()
        .get::<Table>("package")?
        .get::<Table>("preload")?
        .set(
            "hob.json",
            lua.create_function(move |_, ()| Ok(loader.clone()))?,
        )
}
