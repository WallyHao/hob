// --- lua::preload ---
// Installs the module set, then closes the door.
//
// `require` is served entirely from `package.preload`, which is filled from
// the embedded stdlib. The file searchers are removed afterwards, so the set
// of modules a flow can load is fixed before the flow starts and no script can
// pull code in from disk while it runs. User and project module directories
// will be installed after the embedded ones, so a module of the same name can
// replace the bundled one.

use mlua::{Lua, Table};

use crate::lua::embedded;

/// Fill `package.preload`, lock `require` down, and return the `hob` table.
pub(crate) fn install(lua: &Lua) -> mlua::Result<Table> {
    let preload = preload_table(lua)?;
    for (name, source) in embedded::MODULES {
        preload.set(*name, compile(lua, source, name)?)?;
    }
    lock_down(lua)?;
    let hob = require(lua, "hob")?;
    // Attach the modules here, after `hob` is cached: a module that requires
    // `hob` at load time would otherwise see a half-initialised table.
    for name in ["logs", "term", "tmpl"] {
        hob.set(name, require(lua, &format!("hob.{name}"))?)?;
    }
    // Flows write `hob.term.print`, not `require("hob").term.print`, so the
    // module is also a global.
    lua.globals().set("hob", &hob)?;
    Ok(hob)
}

fn compile(lua: &Lua, source: &str, name: &str) -> mlua::Result<mlua::Function> {
    lua.load(source).set_name(name).into_function()
}

fn require(lua: &Lua, name: &str) -> mlua::Result<Table> {
    lua.load(format!("return require({name:?})"))
        .set_name("hob:preload")
        .eval()
}

/// Remove every loader that could read a file at flow time.
fn lock_down(lua: &Lua) -> mlua::Result<()> {
    lua.load(
        r#"
        package.path = ""
        package.cpath = ""
        package.searchers = { package.searchers[1] }
        "#,
    )
    .set_name("hob:lockdown")
    .exec()
}

fn preload_table(lua: &Lua) -> mlua::Result<Table> {
    lua.globals()
        .get::<Table>("package")?
        .get::<Table>("preload")
}

#[cfg(test)]
mod tests {
    use crate::lua::{new_vm, preload};

    #[test]
    fn require_serves_only_installed_modules() {
        let lua = new_vm().expect("the VM builds");
        preload::install(&lua).expect("the modules install");
        let io_loaded = lua
            .load(r#"return pcall(require, "io")"#)
            .eval::<bool>()
            .expect("the chunk evaluates");
        assert!(!io_loaded, "`require` must not reach the file system");
        let logs_loaded = lua
            .load(r#"return require("hob.logs") ~= nil"#)
            .eval::<bool>()
            .expect("the chunk evaluates");
        assert!(logs_loaded, "an embedded module must load");
    }
}
