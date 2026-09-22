// --- cli::help ---

/// The help text, shown for `--help` and every usage error.
///
/// Built at compile time, so a usage error costs one allocation for the message
/// rather than one for the text as well.
pub(crate) const fn text() -> &'static str {
    concat!(
        env!("CARGO_PKG_NAME"),
        " ",
        env!("CARGO_PKG_VERSION"),
        "\nScriptable CLI for AI chores.\n\n",
        "Usage: ",
        env!("CARGO_PKG_NAME"),
        " [options] <command> [args...]\n\n",
        "Commands:\n",
        "  run <file.lua> [args...]     Run a flow file\n",
        "  list                         List installed commands\n",
        "  new <name> [--user|--local]  Create a command\n",
        "  rm <name>                    Remove a command\n",
        "  which <name>                 Show where a command comes from\n",
        "  <name> [args...]             Run an installed command\n",
        "\nOptions:\n",
        "  --dry-run      Perform reads only and print the effects that were refused\n",
        "  --step         Ask before performing each effect\n",
        "  -v, --verbose  Show debug detail; repeat for trace\n",
        "  -q, --quiet    Only report warnings and errors\n",
        "  -y, --yes      Answer every prompt with its default\n",
        "  -h, --help     Print help\n",
        "  -V, --version  Print version\n",
        "\nControl flags may come before or after the command name; everything after `--`\n",
        "goes to the flow.\n",
    )
}
