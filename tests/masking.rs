//! Secret masking in previews: a value the environment calls secret never
//! reaches a printed line, however long it is or wherever it sits.

mod common;

use common::{Flow, stderr};

#[test]
fn preview_masks_a_long_secret_before_clipping_argv() {
    // Longer than the 40-character clip a `proc.exec` argument gets: masking
    // after describing would print the key's first 39 characters.
    let key = format!("sk-{}", "a".repeat(60));
    let flow = Flow::new("argv-mask", r#"hob.proc.exec({ "echo", hob.args[1] })"#);
    let output = flow.run_with(
        &["--dry-run", key.as_str()],
        &[("TEST_API_KEY", key.as_str())],
        None,
    );
    assert!(output.status.success(), "{output:?}");
    let err = stderr(&output);
    assert!(err.contains("dry-run: proc.exec"), "{err}");
    assert!(err.contains("***"), "{err}");
    assert!(!err.contains(&key[..20]), "{err}");
    assert!(!err.contains(&key[key.len() - 10..]), "{err}");
}
