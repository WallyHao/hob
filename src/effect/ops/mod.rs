// --- effect::ops ---
// Typed argument objects, one module per namespace. The engine decodes `cmd`
// into these, so a missing or mistyped argument is an error with the effect's
// name on it rather than a `nil` that travels on.

pub(crate) mod agent;
pub(crate) mod file;
pub(crate) mod logs;
pub(crate) mod proc;
pub(crate) mod term;
pub(crate) mod tmpl;
