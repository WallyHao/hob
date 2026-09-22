// --- exec::describe ---
// One-line, human-readable forms of the effects that get announced.
//
// A preview and a step print an effect before deciding what to do with it, and
// dumping the wire arguments would defeat the point: a prompt is thousands of
// characters and the useful part is its size. Text is clipped and newlines are
// flattened so one effect stays one line; the caller masks secrets.

use serde_json::Value;

use crate::effect::Request;

/// Summarise an effect in one line.
pub(crate) fn line(request: &Request) -> String {
    match (request.ns.as_str(), request.op.as_str()) {
        ("file", "write") => write_line(request),
        ("file", "read" | "stat" | "list") => {
            format!("file.{} {}", request.op, text(request, "path"))
        }
        ("proc", "exec") => format!("proc.exec {}", argv(request)),
        ("proc", "shell") => format!("proc.shell {:?}", clip(&text(request, "line"), 80)),
        ("proc", "which") => format!("proc.which {}", text(request, "prog")),
        ("proc", "setenv") => format!("proc.setenv {}", text(request, "name")),
        ("proc", "unset") => format!("proc.unset {}", text(request, "name")),
        ("proc", "chdir") => format!("proc.chdir {}", text(request, "path")),
        ("proc", other) => format!("proc.{other}"),
        ("agent", "ask" | "send") => ask_line(request),
        ("agent", "open") => format!(
            "agent.open (provider {}, model {})",
            provider(request),
            model(request)
        ),
        ("agent", "list") => "agent.list".to_owned(),
        ("term", "print") => format!("term.print ({} bytes)", text(request, "text").len()),
        ("term", op @ ("input" | "allow" | "select" | "choose")) => {
            format!("term.{op} {:?}", clip(&text(request, "prompt"), 60))
        }
        ("logs", "write") => format!(
            "logs.{} {:?}",
            text(request, "level"),
            clip(&text(request, "msg"), 60)
        ),
        ("tmpl", "fetch") => format!("tmpl.fetch {}", text(request, "name")),
        _ => format!("{}.{}", request.ns, request.op),
    }
}

fn write_line(request: &Request) -> String {
    let note = if bool_field(request, "append") {
        ", append"
    } else {
        ""
    };
    format!(
        "file.write {} ({} bytes{note})",
        text(request, "path"),
        text(request, "text").len()
    )
}

fn ask_line(request: &Request) -> String {
    let what = match request.cmd.get("prompt").and_then(Value::as_str) {
        Some(prompt) => format!("{} byte prompt", prompt.len()),
        None => format!("{} messages", count(request, "messages")),
    };
    let schema = if request.cmd.get("schema").is_some() {
        "schema enforced"
    } else {
        "free-form answer"
    };
    format!(
        "agent.{} ({what}, provider {}, model {}, {schema})",
        request.op,
        provider(request),
        model(request)
    )
}

fn provider(request: &Request) -> String {
    match request.cmd.get("provider") {
        Some(Value::String(id)) => id.clone(),
        Some(Value::Object(spec)) => text_of(spec.get("id")),
        _ => "-".to_owned(),
    }
}

fn model(request: &Request) -> String {
    request
        .cmd
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_owned()
}

fn argv(request: &Request) -> String {
    let Some(items) = request.cmd.get("argv").and_then(Value::as_array) else {
        return String::new();
    };
    items
        .iter()
        .map(|item| clip(item.as_str().unwrap_or("?"), 40))
        .collect::<Vec<_>>()
        .join(" ")
}

fn text(request: &Request, key: &str) -> String {
    text_of(request.cmd.get(key))
}

fn text_of(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or_default().to_owned()
}

fn bool_field(request: &Request, key: &str) -> bool {
    request
        .cmd
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn count(request: &Request, key: &str) -> usize {
    request
        .cmd
        .get(key)
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

/// Shorten long text so one effect stays on one line.
fn clip(text: &str, limit: usize) -> String {
    let flat = text.replace('\n', " ");
    if flat.chars().count() <= limit {
        return flat;
    }
    let kept: String = flat.chars().take(limit.saturating_sub(1)).collect();
    format!("{kept}...")
}
