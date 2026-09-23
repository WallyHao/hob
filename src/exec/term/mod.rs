// --- exec::term ---
// Terminal effects: output and interaction.

mod color;
mod defaults;
mod pick;
mod print;
mod prompt;

pub(crate) use color::Color;
pub(crate) use defaults::{choose as choose_default, select as select_default};
pub(crate) use pick::{choose, select};
pub(crate) use print::print;
pub(crate) use prompt::{allow, input};
