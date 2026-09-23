// --- exec::agent::schema ---
// The JSON-Schema subset a model is held to, and the words used to ask again.
// The validator itself lives in `check`; this module owns the prompts and the
// parsing of an answer that may arrive fenced.

mod check;
mod example;

use serde_json::Value;

pub(crate) use check::validate;
pub(crate) use example::example;

/// The instruction that asks for JSON of the given shape.
pub(crate) fn instruction(schema: &Value) -> String {
    format!(
        "Answer with a single JSON value that matches this schema, with no prose and no code \
         fences:\n{schema}"
    )
}

/// The message that asks again after a malformed answer.
pub(crate) fn repair(error: &str, schema: &Value) -> String {
    format!(
        "That answer was not valid: {error}. Answer again with JSON only, matching the \
         schema:\n{schema}"
    )
}

/// Parse the answer as JSON, tolerating a fenced block.
pub(crate) fn extract(text: &str) -> Result<Value, String> {
    let trimmed = text.trim();
    let body = strip_fence(trimmed).unwrap_or(trimmed);
    serde_json::from_str(body).map_err(|error| format!("not JSON: {error}"))
}

/// The body of a fenced code block, when the answer wrapped the JSON in one.
fn strip_fence(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("```")?;
    let rest = rest.split_once('\n')?.1;
    rest.strip_suffix("```").map(str::trim)
}

#[cfg(test)]
mod tests;
