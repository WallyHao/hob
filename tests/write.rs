//! `hob.file.write` replaces in place: a symlink keeps pointing, no temp stays.

mod common;

use common::Flow;

#[test]
fn a_replace_follows_a_symlink_and_leaves_no_temp() {
    let flow = Flow::new("atomic", r#"hob.file.write("link.txt", "through")"#);
    std::os::unix::fs::symlink("target.txt", flow.dir().join("link.txt")).expect("symlink");
    std::fs::write(flow.dir().join("target.txt"), "before").expect("target");
    let output = flow.run(&[]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(flow.dir().join("target.txt")).expect("target"),
        "through"
    );
    assert!(
        std::fs::symlink_metadata(flow.dir().join("link.txt"))
            .expect("link")
            .file_type()
            .is_symlink()
    );
    let leftovers: Vec<String> = std::fs::read_dir(flow.dir())
        .expect("dir")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains("hob-tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
