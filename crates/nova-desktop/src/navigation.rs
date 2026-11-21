//! Address bar normalization and shell-managed navigation history.
//! The shell history is a first-step approximation; the engine's session
//! history should eventually become the single source of truth.

use url::Url;

/// Accept full HTTP(S) URLs, convenient hostnames, or search terms.
/// Reject file:, javascript:, data: and other privileged schemes.
pub fn normalize_address(raw: &str) -> Result<String, String> {
    let input = raw.trim();
    if input.is_empty() {
        return Err("Enter a URL or search term".into());
    }
    // Explicit schemes must be HTTP(S), including javascript: and file:.
    // Without this check, an unsupported scheme might be mistaken for a search term.
    if let Ok(explicit) = Url::parse(input) {
        if !matches!(explicit.scheme(), "http" | "https") {
            return Err("Only HTTP and HTTPS navigation is supported in this preview".into());
        }
    }

    let candidate = if input.contains("://") {
        input.to_string()
    } else if input.chars().any(char::is_whitespace)
        || (!input.contains('.') && input != "localhost")
    {
        let query: String = url::form_urlencoded::byte_serialize(input.as_bytes()).collect();
        format!("https://www.google.com/search?q={query}")
    } else {
        format!("https://{input}")
    };
    let url = Url::parse(&candidate).map_err(|_| "Invalid URL".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Only HTTP and HTTPS navigation is supported in this preview".into());
    }
    Ok(url.to_string())
}

#[derive(Default, Debug)]
pub struct NavigationHistory {
    entries: Vec<String>,
    position: Option<usize>,
}

impl NavigationHistory {
    pub fn current(&self) -> Option<&str> {
        self.position
            .and_then(|i| self.entries.get(i))
            .map(String::as_str)
    }

    pub fn record(&mut self, url: String) {
        if self.current() == Some(url.as_str()) {
            return;
        }
        let retain = self.position.map_or(0, |i| i + 1);
        self.entries.truncate(retain);
        self.entries.push(url);
        self.position = Some(self.entries.len() - 1);
    }

    pub fn previous(&self) -> Option<(usize, String)> {
        let i = self.position?.checked_sub(1)?;
        Some((i, self.entries.get(i)?.clone()))
    }

    pub fn next(&self) -> Option<(usize, String)> {
        let i = self.position?.checked_add(1)?;
        Some((i, self.entries.get(i)?.clone()))
    }

    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
            self.position = Some(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hostnames_and_explicit_https() {
        assert_eq!(normalize_address("example.com").unwrap(), "https://example.com/");
        assert_eq!(
            normalize_address("https://example.com/path").unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn searches_phrases() {
        assert_eq!(
            normalize_address("hello rust").unwrap(),
            "https://www.google.com/search?q=hello%20rust"
        );
    }

    #[test]
    fn rejects_privileged_schemes() {
        for address in ["file:///etc/passwd", "javascript:alert(1)", "data:text/plain,x"] {
            assert!(normalize_address(address).is_err(), "{address}");
        }
    }

    #[test]
    fn history_truncates_forward_entries() {
        let mut h = NavigationHistory::default();
        h.record("https://a.example/".into());
        h.record("https://b.example/".into());
        h.record("https://c.example/".into());
        let (index, _) = h.previous().unwrap();
        h.select(index);
        assert_eq!(h.current(), Some("https://b.example/"));
        h.record("https://d.example/".into());
        assert!(h.next().is_none());
        assert_eq!(h.previous().unwrap().1, "https://b.example/");
    }
}
