// --- exec::term::print ---
// Terminal output. Styling is named in the effect and spelled here, so the
// engine decides whether colour makes sense and a flow never sees an escape.

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::term::{Print, Stream};

/// Print one line.
// Every operation returns a `Result` so the dispatch table stays uniform;
// printing cannot fail, but the next effect in this namespace may.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn print(line: &Print) -> Result<Value, Failure> {
    let text = paint(line.text.as_str(), line);
    match line.stream.unwrap_or(Stream::Stdout) {
        Stream::Stdout => println!("{text}"),
        Stream::Stderr => eprintln!("{text}"),
    }
    Ok(Value::Null)
}

fn paint(text: &str, line: &Print) -> String {
    match line.style {
        None => text.to_owned(),
        Some(style) => format!("\u{1b}[{}m{text}\u{1b}[0m", style.code()),
    }
}
