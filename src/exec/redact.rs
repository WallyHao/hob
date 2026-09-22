// --- exec::redact ---
// Masking values the environment calls secret before they reach a screen or a
// trace file.
//
// The engine cannot know which argument a flow considers sensitive, but it does
// know which environment variables are named like credentials, and those values
// are what a command line, a header or a prompt would carry if a flow copied one
// in. Masking is exact-match and length-guarded: a value shorter than eight
// characters is left alone, because masking `1` would censor half the output.

/// Masks known secret values in text.
#[derive(Debug)]
pub(crate) struct Redactor {
    secrets: Vec<String>,
}

impl Redactor {
    /// Build a redactor for the given values.
    pub(crate) fn new(mut secrets: Vec<String>) -> Self {
        // Longest first, so a value that contains another is masked whole.
        secrets.sort_by_key(|value| std::cmp::Reverse(value.len()));
        secrets.dedup();
        Self { secrets }
    }

    /// Collect the values of environment variables named like credentials.
    pub(crate) fn from_env() -> Self {
        Self::new(
            std::env::vars()
                .filter(|(name, value)| looks_secret(name) && value.len() >= 8)
                .map(|(_, value)| value)
                .collect(),
        )
    }

    /// Replace every known secret in `text`.
    pub(crate) fn text(&self, text: &str) -> String {
        let mut masked = text.to_owned();
        for secret in &self.secrets {
            if masked.contains(secret.as_str()) {
                masked = masked.replace(secret.as_str(), "***");
            }
        }
        masked
    }
}

/// Whether a variable name looks like it holds a credential.
fn looks_secret(name: &str) -> bool {
    name.split('_').any(|word| {
        matches!(
            word,
            "KEY" | "TOKEN" | "SECRET" | "PASSWORD" | "PASSWD" | "CREDENTIAL"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{Redactor, looks_secret};

    #[test]
    fn a_secret_is_masked_wherever_it_appears() {
        let redactor = Redactor::new(vec!["sk-verysecret".to_owned()]);
        let line = "proc.shell \"curl -H 'Authorization: Bearer sk-verysecret'\"";
        let masked = redactor.text(line);
        assert!(!masked.contains("sk-verysecret"), "{masked}");
        assert!(masked.contains("***"), "{masked}");
    }

    #[test]
    fn a_longer_secret_wins_over_a_shorter_one_inside_it() {
        let redactor = Redactor::new(vec!["secret".to_owned(), "secret-longer".to_owned()]);
        assert_eq!(redactor.text("secret-longer"), "***");
    }

    #[test]
    fn only_credential_names_look_secret() {
        assert!(looks_secret("DEEPSEEK_API_KEY"));
        assert!(looks_secret("GITHUB_TOKEN"));
        assert!(looks_secret("DB_PASSWORD"));
        assert!(!looks_secret("PATH"));
        assert!(!looks_secret("MONKEY"));
    }
}
