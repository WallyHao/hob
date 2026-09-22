// --- lua ---
// The Lua side of the engine: the VM, its sandbox and the embedded modules.

pub(crate) mod embedded;
pub(crate) mod library;
pub(crate) mod preload;
pub(crate) mod pure;

use mlua::{Lua, LuaOptions, StdLib, Table, Value};

/// Build a VM that cannot reach the world on its own.
///
/// `io`, `os` and `debug` are deliberately left out, and the base functions
/// that could load code or write outside an effect are removed outright. A
/// flow therefore has no way to read a file, spawn a process or reach into the
/// VM except by yielding an effect, which is what makes a preview meaningful
/// rather than advisory.
pub(crate) fn new_vm() -> mlua::Result<Lua> {
    let libs = StdLib::COROUTINE
        | StdLib::TABLE
        | StdLib::STRING
        | StdLib::MATH
        | StdLib::UTF8
        | StdLib::PACKAGE;
    let lua = Lua::new_with(libs, LuaOptions::default())?;
    let globals = lua.globals();
    for name in [
        "io", "os", "debug", "loadfile", "dofile", "load", "print", "warn",
    ] {
        globals.set(name, Value::Nil)?;
    }
    let package = globals.get::<Table>("package")?;
    package.set("loadlib", Value::Nil)?;
    Ok(lua)
}

#[cfg(test)]
mod tests {
    use super::new_vm;
    use mlua::Value;

    #[test]
    fn the_vm_cannot_reach_the_world() {
        let lua = new_vm().expect("the VM builds");
        for name in [
            "io", "os", "debug", "loadfile", "dofile", "load", "print", "warn",
        ] {
            let value = lua.globals().get::<Value>(name).expect("global lookup");
            assert!(matches!(value, Value::Nil), "`{name}` must be unreachable");
        }
        let package = lua
            .globals()
            .get::<mlua::Table>("package")
            .expect("package exists");
        let loadlib = package.get::<Value>("loadlib").expect("loadlib lookup");
        assert!(
            matches!(loadlib, Value::Nil),
            "`package.loadlib` must be gone"
        );
    }
}
