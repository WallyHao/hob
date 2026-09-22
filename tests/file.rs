//! `hob.file` through the real binary: read, write, stat and list.

mod common;

use common::{Flow, stderr, stdout};

#[test]
fn write_then_read_round_trips() {
    let flow = Flow::new(
        "round",
        r#"
        hob.file.write("out/data.txt", "hello\n")
        hob.file.write("out/data.txt", "world\n", { append = true })
        hob.term.print(hob.file.read("out/data.txt"))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "hello\nworld\n\n");
}

#[test]
fn stat_reports_size_and_kind() {
    let flow = Flow::new(
        "stat",
        r#"
        hob.file.write("notes.txt", "abc")
        local info = hob.file.stat("notes.txt")
        hob.term.print(info.kind .. " " .. tostring(info.size))
        hob.term.print(hob.file.stat("nowhere.txt") == nil and "missing")
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "file 3\nmissing\n");
}

#[test]
fn list_sorts_entry_names() {
    let flow = Flow::new(
        "list",
        r#"
        hob.file.write("dir/b.txt", "")
        hob.file.write("dir/a.txt", "")
        hob.term.print(table.concat(hob.file.list("dir"), ","))
        "#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "a.txt,b.txt\n");
}

#[test]
fn optional_read_returns_nil() {
    let flow = Flow::new(
        "optional",
        r#"hob.term.print(tostring(hob.file.read("nope.txt", { optional = true })))"#,
    );
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "nil\n");
}

#[test]
fn a_missing_file_fails_loudly() {
    let flow = Flow::new("missing", r#"hob.file.read("nope.txt")"#);
    let output = flow.run(&[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("cannot read `nope.txt`"),
        "{}",
        stderr(&output)
    );
}
