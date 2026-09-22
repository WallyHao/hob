// --- exec::term ---
// Terminal effects: output and interaction.

mod pick;
mod print;
mod prompt;

pub(crate) use pick::{choose, select};
pub(crate) use print::print;
pub(crate) use prompt::{allow, input};
