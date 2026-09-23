// --- lua::library ---
// Shared Lua modules for commands.
//
// The embedded modules are installed and cached before a flow runs, so this
// searcher adds modules rather than replacing the stdlib: it resolves a name
// like `util.text` in the roots authorized before execution. A trusted project's
// library wins over the user's `lib/`. Module names cannot express traversal,
// and the authorized roots check symlink targets before handing back a path.

use std::path::PathBuf;

use mlua::{Lua, Value};

use crate::store::libraries::Libraries;

/// Install the library searcher for one run.
pub(crate) fn install(lua: &Lua, libraries: Libraries) -> mlua::Result<()> {
    let searcher = lua.create_function(move |lua, name: String| -> mlua::Result<Value> {
        match resolve(&libraries, &name)? {
            Some(path) => {
                let source = std::fs::read_to_string(&path).map_err(|error| {
                    mlua::Error::RuntimeError(format!("cannot read {}: {error}", path.display()))
                })?;
                let loader = lua
                    .load(&source)
                    .set_name(path.display().to_string())
                    .into_function()?;
                Ok(Value::Function(loader))
            }
            // A string tells `require` this searcher did not find the
            // module, and becomes part of its error message.
            None => Ok(Value::String(
                lua.create_string(format!("no library `{name}`"))?,
            )),
        }
    })?;
    let searchers = lua
        .globals()
        .get::<mlua::Table>("package")?
        .get::<mlua::Table>("searchers")?;
    searchers.raw_set(searchers.raw_len() + 1, searcher)?;
    Ok(())
}

/// Find a module in the library directories, first match wins.
fn resolve(libraries: &Libraries, name: &str) -> mlua::Result<Option<PathBuf>> {
    if !valid_module(name) {
        return Ok(None);
    }
    let relative = name.split('.').collect::<Vec<_>>().join("/");
    libraries
        .find(&PathBuf::from(relative).with_extension("lua"))
        .map_err(|failure| mlua::Error::RuntimeError(failure.message))
}

/// Whether a module name is a run of lowercase words separated by dots.
fn valid_module(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|word| {
            let mut chars = word.chars();
            matches!(chars.next(), Some(first) if first.is_ascii_lowercase() || first == '_')
                && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::valid_module;

    #[test]
    fn a_dotted_name_is_a_module() {
        assert!(valid_module("util"));
        assert!(valid_module("util.text"));
    }

    #[test]
    fn a_path_is_not_a_module() {
        assert!(!valid_module(""));
        assert!(!valid_module("../secret"));
        assert!(!valid_module("Util"));
        assert!(!valid_module("util/text"));
    }
}
