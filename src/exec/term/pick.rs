// --- exec::term::pick ---
// Numbered lists. An answer may be a number or an exact label, and an empty
// line takes the default, so both a person and a script can answer the same
// question.

use std::io::{self, Write as _};

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::term::{Choice, Choose, Select};

use super::defaults;
use super::prompt::ask;

/// Pick one option from a numbered list.
pub(crate) fn select(request: &Select, yes: bool) -> Result<Value, Failure> {
    let labels: Vec<&str> = request.options.iter().map(Choice::label).collect();
    if labels.is_empty() {
        return Err(Failure::new("`term.select` needs at least one option"));
    }
    if yes {
        return Ok(defaults::select(request));
    }
    print_options(request.prompt.as_deref().unwrap_or("choose one"), &labels);
    loop {
        let line = ask("> ")?;
        if line.is_empty() {
            let Some(default) = &request.default else {
                let _ = writeln!(io::stderr(), "please pick one");
                continue;
            };
            return match labels.iter().position(|label| label == default) {
                Some(index) => Ok(request.options[index].value()),
                None => Err(Failure::new(format!(
                    "`{default}` is not one of the options"
                ))),
            };
        }
        match pick(&line, &labels) {
            Some(index) => return Ok(request.options[index].value()),
            None => {
                let _ = writeln!(io::stderr(), "please pick a number or a label");
            }
        }
    }
}

/// Pick several options from a numbered list.
pub(crate) fn choose(request: &Choose, yes: bool) -> Result<Value, Failure> {
    let labels: Vec<&str> = request.options.iter().map(Choice::label).collect();
    if labels.is_empty() {
        return Err(Failure::new("`term.choose` needs at least one option"));
    }
    if yes {
        return Ok(defaults::choose(request));
    }
    print_options(request.prompt.as_deref().unwrap_or("choose"), &labels);
    let min = request.min.unwrap_or(0);
    let max = request.max.unwrap_or(labels.len());
    loop {
        let line = ask("> ")?;
        let answers = if line.is_empty() {
            request.defaults.clone()
        } else {
            split(&line)
        };
        let mut picked = Vec::new();
        let mut unknown = None;
        for answer in &answers {
            if let Some(index) = pick(answer, &labels) {
                picked.push(index);
            } else {
                unknown = Some(answer.clone());
                break;
            }
        }
        if let Some(answer) = unknown {
            let _ = writeln!(io::stderr(), "`{answer}` is not one of the options");
            continue;
        }
        if picked.len() < min || picked.len() > max {
            let _ = writeln!(io::stderr(), "pick between {min} and {max}");
            continue;
        }
        return Ok(Value::Array(
            picked
                .into_iter()
                .map(|index| request.options[index].value())
                .collect(),
        ));
    }
}

fn print_options(prompt: &str, labels: &[&str]) {
    let mut err = io::stderr().lock();
    let _ = writeln!(err, "{prompt}");
    for (index, label) in labels.iter().enumerate() {
        let _ = writeln!(err, "  {}) {label}", index + 1);
    }
}

/// A one-based number or an exact label; anything else is `None`.
fn pick(answer: &str, labels: &[&str]) -> Option<usize> {
    let answer = answer.trim();
    if let Ok(number) = answer.parse::<usize>() {
        return (1..=labels.len()).contains(&number).then(|| number - 1);
    }
    labels.iter().position(|label| *label == answer)
}

fn split(line: &str) -> Vec<String> {
    line.split([',', ' '])
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}
