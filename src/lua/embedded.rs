// --- lua::embedded ---
// The Lua standard library, compiled into the binary.
//
// Shipping the stdlib inside the executable means hob works with no
// installation step and the modules a flow requires can never be missing or
// out of step with the engine. The engine's own capabilities are the
// exception: they live in Rust, and these modules are their Lua-facing shape.

/// Modules registered in `package.preload`, keyed by the name `require` uses.
pub(crate) const MODULES: &[(&str, &str)] = &[
    ("hob", include_str!("../../lua/hob/init.lua")),
    ("hob.agent", include_str!("../../lua/hob/agent.lua")),
    ("hob.file", include_str!("../../lua/hob/file.lua")),
    ("hob.logs", include_str!("../../lua/hob/logs.lua")),
    ("hob.proc", include_str!("../../lua/hob/proc.lua")),
    ("hob.term", include_str!("../../lua/hob/term.lua")),
    ("hob.tmpl", include_str!("../../lua/hob/tmpl.lua")),
];
