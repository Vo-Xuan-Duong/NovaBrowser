//! Address normalization and provisional shell-managed navigation history.
//! Real document and same-document history must ultimately come from the engine.

use url::Url;

pub fn normalize_address(raw: &str) -> Result<String, &'static str> {
    normalize_address_with_search(raw, "google")
}

/// Build a search URL for a deliberately selected provider, even when the
/// query looks like a hostname. Used only after an explicit user action.
pub fn build_search_url(query: &str, provider: &str) -> Result<String, &'static str> {
    let query = query.trim();
    if query.is_empty() || query.len() > 4096 || query.chars().any(char::is_control) {
        return Err("Enter a valid search query");
    }
    let base = match provider {
        "google" => "https://www.google.com/search?q=",
        "bing" => "https://www.bing.com/search?q=",
        "duckduckgo" => "https://duckduckgo.com/?q=",
        _ => return Err("Unsupported search engine"),
    };
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    Ok(format!("{base}{encoded}"))
}

/// Detect a Google search redirected to Google's own unusual-traffic page.
/// The redirect alone does not imply a broken search box, and it should not
/// trigger an automatic request to another search provider.
pub fn blocked_google_search_query(requested: &str, landed: &str) -> Option<String> {
    fn is_google(url: &Url) -> bool {
        matches!(url.host_str(), Some("google.com" | "www.google.com"))
            && url.scheme() == "https"
    }
    fn query_from_search(url: &Url) -> Option<String> {
        if !is_google(url) || url.path() != "/search" {
            return None;
        }
        url.query_pairs()
            .find(|(key, _)| key == "q")
            .map(|(_, value)| value.into_owned())
            .filter(|value| !value.trim().is_empty()
                && value.len() <= 4096 && !value.chars().any(char::is_control))
    }
    let landed = Url::parse(landed).ok()?;
    if !is_google(&landed) || !matches!(landed.path(), "/sorry" | "/sorry/" | "/sorry/index") {
        return None;
    }
    // Prefer the original request. The continue= parameter is a fallback
    // when a website-initiated Google search redirects without going through
    // NovaBrowser's address bar.
    Url::parse(requested).ok().and_then(|url| query_from_search(&url))
        .or_else(|| {
            let continued = landed.query_pairs().find(|(key, _)| key == "continue")?.1;
            Url::parse(&continued).ok().and_then(|url| query_from_search(&url))
        })
}

pub fn normalize_address_with_search(raw: &str, search_engine: &str) -> Result<String, &'static str> {
    let input = raw.trim();
    if input.eq_ignore_ascii_case("nova://home") {
        return Ok("nova://home".to_string());
    }
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
        build_search_url(input, search_engine)?
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
    fn explicitly_chosen_provider_never_mistakes_a_query_for_a_website() {
        let url = Url::parse(&build_search_url("example.com", "bing").unwrap()).unwrap();
        assert_eq!(url.host_str(), Some("www.bing.com"));
        assert_eq!(url.query_pairs().find(|(key, _)| key == "q").unwrap().1, "example.com");
        assert!(build_search_url("hello", "unknown").is_err());
        assert!(build_search_url("", "google").is_err());
        assert!(build_search_url("a\n", "bing").is_ok()); // trim trailing whitespace
        assert!(build_search_url("a\u{0000}b", "bing").is_err());
    }

    #[test]
    fn detects_a_real_google_sorry_redirect_but_not_lookalike_hosts() {
        let requested = build_search_url("xin chào Việt Nam", "google").unwrap();
        let blocked = format!(
            "https://www.google.com/sorry/index?continue={}",
            url::form_urlencoded::byte_serialize(requested.as_bytes()).collect::<String>()
        );
        assert_eq!(blocked_google_search_query(&requested, &blocked).as_deref(), Some("xin chào Việt Nam"));
        assert_eq!(blocked_google_search_query("https://example.com/", &blocked).as_deref(), Some("xin chào Việt Nam"));
        assert_eq!(blocked_google_search_query(&requested, "https://www.google.com/search?q=hello"), None);
        assert_eq!(blocked_google_search_query(&requested, "https://www.google.com.evil.test/sorry/index"), None);
        assert_eq!(blocked_google_search_query(&requested, "http://www.google.com/sorry/index"), None);
        assert_eq!(blocked_google_search_query("https://malicious.test/search?q=x",
            "https://www.google.com/sorry/index?continue=https%3A%2F%2Fmalicious.test%2Fsearch%3Fq%3Dx"), None);
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
    fn accepts_only_the_internal_home_address() {
        assert_eq!(normalize_address("nova://home").unwrap(), "nova://home");
        assert_eq!(normalize_address("NOVA://HOME").unwrap(), "nova://home");
        for value in ["nova://settings", "nova://home/evil", "nova://debug", "nova://home?x=1"] {
            assert!(normalize_address(value).is_err(), "{value}");
        }
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
