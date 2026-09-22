// --- exec::term::prompt ---
// One-line questions. The question goes to stderr and the answer comes from
// stdin, so stdout stays the flow's data channel and a run with piped input
// still has a usable stdin.

use std::io::{self, Write as _};

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::term::{Allow, Input};

/// One line of input; an empty line takes the default.
pub(crate) fn input(request: &Input, yes: bool) -> Result<Value, Failure> {
    if yes {
        return Ok(Value::String(
            request
                .default
                .clone()
                .or_else(|| request.initial.clone())
                .unwrap_or_default(),
        ));
    }
    let prompt = request.prompt.as_deref().unwrap_or("");
    let line = ask(prompt)?;
    if line.is_empty()
        && let Some(default) = request.default.clone().or_else(|| request.initial.clone())
    {
        return Ok(Value::String(default));
    }
    Ok(Value::String(line))
}

/// Yes or no; an empty line takes the default.
pub(crate) fn allow(request: &Allow, yes: bool) -> Result<Value, Failure> {
    let default = request.default.unwrap_or(false);
    if yes {
        return Ok(Value::Bool(default));
    }
    if let Some(detail) = &request.detail {
        eprintln!("{detail}");
    }
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    let question = request.prompt.as_deref().unwrap_or("continue?");
    let prompt = format!("{question} {hint} ");
    loop {
        match ask(&prompt)?.to_ascii_lowercase().as_str() {
            "" => return Ok(Value::Bool(default)),
            "y" | "yes" => return Ok(Value::Bool(true)),
            "n" | "no" => return Ok(Value::Bool(false)),
            _ => eprintln!("please answer yes or no"),
        }
    }
}

/// Write a prompt and read one line. End of input is a failure, not an empty
/// answer, so a flow cannot mistake a closed pipe for a decision.
pub(super) fn ask(prompt: &str) -> Result<String, Failure> {
    let mut err = io::stderr();
    let _ = write!(err, "{prompt}");
    let _ = err.flush();
    let mut line = String::new();
    let read = io::stdin()
        .read_line(&mut line)
        .map_err(|error| Failure::new(format!("cannot read input: {error}")))?;
    if read == 0 {
        return Err(Failure::new("input closed before an answer arrived"));
    }
    Ok(line.trim_end_matches(['\n', '\r']).to_owned())
}
