//! Address normalization and provisional shell-managed navigation history.
//! Real document and same-document history must ultimately come from the engine.

use url::Url;

pub fn normalize_address(raw: &str) -> Result<String, &'static str> {
    normalize_address_with_search(raw, "google")
}

pub fn normalize_address_with_search(raw: &str, search_engine: &str) -> Result<String, &'static str> {
    let input = raw.trim();
    if input.is_empty() {
        return Err("Enter a web address or search query");
    }

    // "localhost:3000" and "example.com:8080" look like custom URL schemes
    // to Url::parse(); recognize them before rejecting unsupported schemes.
    let host_port = input.split_once(':').is_some_and(|(host, port)| {
        (host == "localhost" || host.contains('.'))
            && !port.is_empty()
            && port.chars().all(|c| c.is_ascii_digit())
    });

    if !host_port {
        if let Ok(url) = Url::parse(input) {
            if !matches!(url.scheme(), "http" | "https") {
                return Err("Only http:// and https:// are supported");
            }
        }
    }

    let target = if input.contains("://") {
        input.to_string()
    } else if input.chars().any(char::is_whitespace)
        || (!input.contains('.') && input != "localhost" && !input.starts_with("localhost:"))
    {
        let encoded: String = url::form_urlencoded::byte_serialize(input.as_bytes()).collect();
        let base = match search_engine {
            "bing" => "https://www.bing.com/search?q=",
            "duckduckgo" => "https://duckduckgo.com/?q=",
            _ => "https://www.google.com/search?q=",
        };
        format!("{base}{encoded}")
    } else if input == "localhost" || input.starts_with("localhost:") {
        format!("http://{input}")
    } else {
        format!("https://{input}")
    };

    let url = Url::parse(&target).map_err(|_| "Invalid web address")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Only http:// and https:// are supported");
    }
    Ok(url.to_string())
}

#[derive(Default)]
pub struct NavigationHistory {
    urls: Vec<String>,
    current: Option<usize>,
}

impl NavigationHistory {
    pub fn current(&self) -> Option<&str> {
        self.current.and_then(|i| self.urls.get(i)).map(String::as_str)
    }

    pub fn record(&mut self, url: String) {
        if self.current() == Some(url.as_str()) {
            return;
        }
        let keep = self.current.map_or(0, |i| i + 1);
        self.urls.truncate(keep);
        self.urls.push(url);
        self.current = Some(self.urls.len() - 1);
    }

    pub fn previous(&self) -> Option<(usize, String)> {
        let i = self.current?.checked_sub(1)?;
        Some((i, self.urls.get(i)?.clone()))
    }

    pub fn next(&self) -> Option<(usize, String)> {
        let i = self.current?.checked_add(1)?;
        Some((i, self.urls.get(i)?.clone()))
    }

    pub fn select(&mut self, index: usize) {
        if index < self.urls.len() {
            self.current = Some(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_http_hostnames_and_searches() {
        assert_eq!(normalize_address("example.com").unwrap(), "https://example.com/");
        assert_eq!(normalize_address("localhost:3000").unwrap(), "http://localhost:3000/");
        // The URL serializer may normalize spaces as '+' (form-style encoding)
        // instead of '%20'. Both decode to the same search query.
        let search = Url::parse(&normalize_address("hello browser").unwrap()).unwrap();
        assert_eq!(search.host_str(), Some("www.google.com"));
        assert_eq!(search.path(), "/search");
        assert_eq!(search.query_pairs().find(|(key, _)| key == "q").unwrap().1, "hello browser");
    }

    #[test]
    fn search_preserves_literal_plus_and_unicode() {
        let search = Url::parse(&normalize_address("C++ tiếng Việt").unwrap()).unwrap();
        let query = search.query_pairs().find(|(key, _)| key == "q").unwrap().1;
        assert_eq!(query, "C++ tiếng Việt");
    }

    #[test]
    fn respects_the_default_search_engine() {
        let bing = normalize_address_with_search("rust language", "bing").unwrap();
        let duck = normalize_address_with_search("rust language", "duckduckgo").unwrap();
        assert_eq!(url::Url::parse(&bing).unwrap().host_str(), Some("www.bing.com"));
        assert_eq!(url::Url::parse(&duck).unwrap().host_str(), Some("duckduckgo.com"));
        let q = url::Url::parse(&duck).unwrap();
        assert_eq!(q.query_pairs().find(|(key, _)| key == "q").unwrap().1, "rust language");
        assert_eq!(normalize_address_with_search("example.com", "bing").unwrap(), "https://example.com/");
    }

    #[test]
    fn does_not_open_privileged_schemes() {
        for value in ["file:///etc/passwd", "javascript:alert(1)", "data:text/html,x"] {
            assert!(normalize_address(value).is_err(), "{value}");
        }
    }

    #[test]
    fn back_forward_and_branching() {
        let mut h = NavigationHistory::default();
        h.record("a".into());
        h.record("b".into());
        h.record("c".into());
        let (i, _) = h.previous().unwrap();
        h.select(i);
        assert_eq!(h.current(), Some("b"));
        h.record("d".into());
        assert!(h.next().is_none());
        assert_eq!(h.previous().unwrap().1, "b");
    }
}
