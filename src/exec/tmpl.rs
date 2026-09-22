// --- exec::tmpl ---
// Prompt templates come from the project, then the user's configuration, then
// the binary. The name is a relative path: an absolute one or one that climbs
// out of the directory is refused, so a flow cannot turn a template lookup into
// a general file read.

use std::fs;
use std::path::{Component, Path};

use serde_json::Value;

use crate::effect::Failure;
use crate::effect::ops::tmpl::Fetch;
use crate::paths::Paths;

/// Read one template by name.
pub(crate) fn fetch(paths: &Paths, request: &Fetch) -> Result<Value, Failure> {
    let name = Path::new(&request.name);
    if !safe(name) {
        return Err(Failure::new(format!(
            "`{}` is not a valid template name",
            request.name
        )));
    }
    let roots = [paths.project_prompts(), paths.user_prompts()];
    for root in roots.into_iter().flatten() {
        let path = root.join(name);
        match fs::read_to_string(&path) {
            Ok(text) => return Ok(Value::String(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(Failure::new(format!(
                    "cannot read `{}`: {error}",
                    path.display()
                )));
            }
        }
    }
    Err(Failure::new(format!(
        "no template named `{}`",
        request.name
    )))
}

/// A relative path made of plain components, with nothing that could escape.
fn safe(name: &Path) -> bool {
    !name.as_os_str().is_empty()
        && name
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
