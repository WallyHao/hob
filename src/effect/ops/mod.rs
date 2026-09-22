// --- effect::ops ---
// Typed argument objects, one module per namespace. The engine decodes `cmd`
// into these, so a missing or mistyped argument is an error with the effect's
// name on it rather than a `nil` that travels on.

pub(crate) mod logs;
pub(crate) mod term;
