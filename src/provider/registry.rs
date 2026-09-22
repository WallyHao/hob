// --- provider::registry ---
// The built-in providers. Adding an OpenAI-compatible one is a data change,
// not a code change: id, base URL, key variable, protocol.

use super::spec::{Protocol, ProviderSpec};

/// `DeepSeek`, the first provider and the reference entry.
fn deepseek() -> ProviderSpec {
    ProviderSpec::new(
        "deepseek",
        "https://api.deepseek.com",
        "DEEPSEEK_API_KEY",
        Protocol::OpenAi,
    )
}

/// Every provider hob knows out of the box.
pub fn builtins() -> Vec<ProviderSpec> {
    vec![deepseek()]
}

/// Look up a provider by id.
pub fn find(id: &str) -> Option<ProviderSpec> {
    builtins().into_iter().find(|spec| spec.id == id)
}

#[cfg(test)]
mod tests {
    use super::{deepseek, find};

    #[test]
    fn deepseek_names_its_own_key_variable() {
        let spec = deepseek();
        assert_eq!(spec.base_url, "https://api.deepseek.com");
        assert_eq!(spec.api_key_env, "DEEPSEEK_API_KEY");
        assert_eq!(
            spec.endpoint("chat/completions"),
            "https://api.deepseek.com/chat/completions"
        );
    }

    #[test]
    fn unknown_ids_are_not_invented() {
        assert!(find("not-a-provider").is_none());
    }
}
