/// Exact-match and trailing-wildcard (`*`) URI allowlist.
#[derive(Clone, Debug)]
pub struct UriAllowlist {
    exact: Vec<String>,
    prefix: Vec<String>,
}

impl UriAllowlist {
    #[must_use]
    pub fn new(entries: Vec<String>) -> Self {
        let mut exact = Vec::new();
        let mut prefix = Vec::new();
        for entry in entries {
            if entry.ends_with('*') {
                prefix.push(entry[..entry.len() - 1].to_string());
            } else {
                exact.push(entry);
            }
        }
        Self { exact, prefix }
    }

    #[must_use]
    pub fn is_allowed(&self, uri: &str) -> bool {
        if self.exact.iter().any(|entry| entry == uri) {
            return true;
        }
        self.prefix.iter().any(|prefix| uri.starts_with(prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_and_wildcard() {
        let list = UriAllowlist::new(vec![
            "http://example.com".to_string(),
            "http://store.example/*".to_string(),
        ]);
        assert!(list.is_allowed("http://example.com"));
        assert!(list.is_allowed("http://store.example/products/foo"));
        assert!(!list.is_allowed("http://example.com/"));
    }
}
