//! `hob.tmpl.fetch` through the real binary: project, user, and the names
//! that must not resolve to anything.

mod common;

use common::{Flow, stderr, stdout};

#[test]
fn project_prompts_win_over_user_prompts() {
    let flow = Flow::new("project", r#"hob.term.print(hob.tmpl.fetch("greet.txt"))"#);
    flow.write(".hob/prompts/greet.txt", "from the project");
    flow.write("config/prompts/greet.txt", "from the user");
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "from the project\n");
}

#[test]
fn user_prompts_are_the_fallback() {
    let flow = Flow::new("user", r#"hob.term.print(hob.tmpl.fetch("note.txt"))"#);
    flow.write(".hob/prompts/greet.txt", "from the project");
    flow.write("config/prompts/note.txt", "from the user");
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "from the user\n");
}

#[test]
fn traversal_is_refused() {
    let flow = Flow::new("traversal", r#"hob.tmpl.fetch("../outside.txt")"#);
    flow.write("outside.txt", "not a template");
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("not a valid template name"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_missing_template_names_itself() {
    let flow = Flow::new("missing", r#"hob.tmpl.fetch("nope.txt")"#);
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no template named `nope.txt`"),
        "{}",
        stderr(&output)
    );
}
