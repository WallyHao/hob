// --- exec::term::print ---
// Terminal output. Styling is named in the effect and spelled here, so the
// engine decides whether colour makes sense and a flow never sees an escape.

use std::io::{self, IsTerminal as _, Write as _};

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::term::{Print, Stream};

use super::color::Color;

/// Print one line, styled when the run asked for it and the stream can show it.
pub(crate) fn print(line: &Print, color: Color) -> Result<Value, Failure> {
    let stream = line.stream.unwrap_or(Stream::Stdout);
    let text = if line.style.is_some() && color.allows(terminal(stream)) {
        paint(line.text.as_str(), line)
    } else {
        line.text.clone()
    };
    // `println!` would panic when the stream is gone, which a pipeline like
    // `hob ... | head` arranges on purpose; the write is fallible instead.
    let written = match stream {
        Stream::Stdout => writeln!(io::stdout().lock(), "{text}"),
        Stream::Stderr => writeln!(io::stderr().lock(), "{text}"),
    };
    match written {
        Ok(()) => Ok(Value::Null),
        // A reader that stopped early (`hob ... | head`) is not a flow failure.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(Value::Null),
        Err(error) => Err(Failure::new(format!("cannot write output: {error}"))),
    }
}

/// Whether the stream a line goes to is a terminal.
fn terminal(stream: Stream) -> bool {
    match stream {
        Stream::Stdout => io::stdout().is_terminal(),
        Stream::Stderr => io::stderr().is_terminal(),
    }
}

fn paint(text: &str, line: &Print) -> String {
    match line.style {
        None => text.to_owned(),
        Some(style) => format!("\u{1b}[{}m{text}\u{1b}[0m", style.code()),
    }
}
