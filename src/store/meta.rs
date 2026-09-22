// --- store::meta ---
// The `---` header of a command: its summary, its usage line and how many
// arguments it accepts.
//
// The header is read without executing anything: only leading `---` lines are
// looked at, and only the keys this module knows are interpreted. Any other
// `---` line is the summary, which is what `hob list` shows. `hob <name>
// --help` prints all of it.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// What a command says about itself in its header.
#[derive(Debug, Default, Clone)]
pub(crate) struct Meta {
    /// The first `---` line that is not a known key.
    pub(crate) summary: Option<String>,
    /// The line `--help` and a validation error show.
    pub(crate) usage: Option<String>,
    /// How many arguments a call may carry.
    pub(crate) args: Option<Args>,
}

/// The `args:` range: `N`, `N+` or `N..M`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Args {
    min: usize,
    max: Option<usize>,
}

impl Args {
    /// Parse the three accepted forms; anything else is `None`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if let Some(min) = text.strip_suffix('+') {
            return Some(Self {
                min: min.trim().parse().ok()?,
                max: None,
            });
        }
        if let Some((min, max)) = text.split_once("..") {
            return Some(Self {
                min: min.trim().parse().ok()?,
                max: Some(max.trim().parse().ok()?),
            });
        }
        Some(Self {
            min: text.parse().ok()?,
            max: Some(text.parse().ok()?),
        })
    }

    /// Check an argument count, with a message that repeats the usage line.
    pub(crate) fn check(self, count: usize, usage: &str) -> Result<(), String> {
        if count < self.min || self.max.is_some_and(|max| count > max) {
            return Err(format!(
                "expected {}, got {count}\n  usage: {usage}",
                self.describe()
            ));
        }
        Ok(())
    }

    /// The range in words, for `--help`.
    pub(crate) fn describe(self) -> String {
        match self.max {
            Some(max) if max == self.min => format!("{max} arguments"),
            Some(max) => format!("{} to {max} arguments", self.min),
            None => format!("{} or more arguments", self.min),
        }
    }
}

/// Read the header of a command file.
pub(crate) fn read(path: &Path) -> Meta {
    let Ok(file) = fs::File::open(path) else {
        return Meta::default();
    };
    let mut meta = Meta::default();
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(text) = line.strip_prefix("--- ") else {
            break;
        };
        let text = text.trim();
        match text
            .split_once(':')
            .map(|(key, value)| (key.trim(), value.trim()))
        {
            Some(("usage", value)) if !value.is_empty() => meta.usage = Some(value.to_owned()),
            Some(("args", value)) if !value.is_empty() => meta.args = Args::parse(value),
            _ if meta.summary.is_none() && !text.is_empty() => {
                meta.summary = Some(text.to_owned());
            }
            _ => {}
        }
    }
    meta
}

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn the_three_forms_parse() {
        assert_eq!(
            Args::parse("2"),
            Some(Args {
                min: 2,
                max: Some(2)
            })
        );
        assert_eq!(Args::parse("1+"), Some(Args { min: 1, max: None }));
        assert_eq!(
            Args::parse("0..2"),
            Some(Args {
                min: 0,
                max: Some(2)
            })
        );
        assert_eq!(Args::parse("many"), None);
    }

    #[test]
    fn a_count_outside_the_range_names_the_usage() {
        let args = Args::parse("1..2").expect("parses");
        assert!(args.check(1, "cmd <a> [b]").is_ok());
        let error = args.check(0, "cmd <a> [b]").expect_err("too few");
        assert!(error.contains("cmd <a> [b]"), "{error}");
        assert!(args.check(3, "cmd <a> [b]").is_err());
    }
}
