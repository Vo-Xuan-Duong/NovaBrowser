use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::navigation::normalize_address_with_search;

const MAX_HISTORY_ENTRIES: usize = 500;
pub const MAX_TABS: usize = 8;
const MAX_SESSION_URL_LENGTH: usize = 4096;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Visit {
    pub title: String,
    pub url: String,
    pub visited_at: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct BrowserSettings {
    pub theme: String,
    pub homepage: String,
    pub startup: String,
    pub new_tab: String,
    pub search_engine: String,
    pub save_history: bool,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            homepage: "https://example.com/".to_string(),
            startup: "restore".to_string(),
            new_tab: "homepage".to_string(),
            search_engine: "google".to_string(),
            save_history: true,
        }
    }
}

impl BrowserSettings {
    /// Profiles from older versions can be missing fields or contain
    /// unsupported values. Keep the rest of the profile instead of discarding
    /// bookmarks and tabs when settings need a fallback.
    pub fn sanitized(mut self) -> Self {
        if !matches!(self.theme.as_str(), "light" | "dark" | "system") {
            self.theme = "light".to_string();
        }
        if !matches!(self.startup.as_str(), "restore" | "homepage") {
            self.startup = "restore".to_string();
        }
        if !matches!(self.new_tab.as_str(), "homepage" | "blank") {
            self.new_tab = "homepage".to_string();
        }
        if !matches!(self.search_engine.as_str(), "google" | "bing" | "duckduckgo") {
            self.search_engine = "google".to_string();
        }
        self.homepage = if self.homepage.len() <= MAX_SESSION_URL_LENGTH {
            normalize_address_with_search(&self.homepage, &self.search_engine)
                .unwrap_or_else(|_| BrowserSettings::default().homepage)
        } else {
            BrowserSettings::default().homepage
        };
        self
    }
}


#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct BrowserSession {
    pub tabs: Vec<SessionTab>,
    pub active_index: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTab {
    pub title: String,
    pub url: String,
}

impl BrowserSession {
    /// Reject invalid and privileged URLs before restoring stored tabs.
    fn sanitized(self) -> Self {
        let mut tabs = Vec::new();
        let mut selected = None;
        for (original_index, mut tab) in self.tabs.into_iter().enumerate() {
            if tabs.len() >= MAX_TABS { break; }
            let valid = tab.url.is_empty() || (
                tab.url.len() <= MAX_SESSION_URL_LENGTH
                    && Url::parse(&tab.url).ok().is_some_and(|url| {
                        matches!(url.scheme(), "http" | "https") && url.host_str().is_some()
                    })
            );
            if !valid { continue; }
            if original_index == self.active_index {
                selected = Some(tabs.len());
            }
            tab.title = tab.title.chars().take(240).collect();
            tabs.push(tab);
        }
        let active_index = selected.unwrap_or_else(|| {
            self.active_index.min(tabs.len().saturating_sub(1))
        });
        Self { tabs, active_index }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct BrowserData {
    bookmarks: Vec<Bookmark>,
    visits: Vec<Visit>,
    settings: BrowserSettings,
    session: BrowserSession,
}

pub struct BrowserDataStore {
    path: PathBuf,
    data: Mutex<BrowserData>,
}

impl BrowserDataStore {
    pub fn load(path: PathBuf) -> Self {
        let mut data: BrowserData = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        data.settings = data.settings.sanitized();
        Self {
            path,
            data: Mutex::new(data),
        }
    }

    fn lock(&self) -> MutexGuard<'_, BrowserData> {
        self.data.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn save(&self, data: &BrowserData) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(data).map_err(|error| error.to_string())?;
        fs::write(&self.path, bytes).map_err(|error| error.to_string())
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.lock().bookmarks.iter().any(|bookmark| bookmark.url == url)
    }

    pub fn toggle_bookmark(&self, title: String, url: String) -> Result<bool, String> {
        let mut data = self.lock();
        let is_bookmarked = if let Some(index) = data.bookmarks.iter().position(|item| item.url == url) {
            data.bookmarks.remove(index);
            false
        } else {
            data.bookmarks.push(Bookmark { title, url });
            true
        };
        self.save(&data)?;
        Ok(is_bookmarked)
    }

    pub fn remove_bookmark(&self, url: &str) -> Result<(), String> {
        let mut data = self.lock();
        data.bookmarks.retain(|bookmark| bookmark.url != url);
        self.save(&data)
    }

    pub fn bookmarks(&self) -> Vec<Bookmark> {
        let mut bookmarks = self.lock().bookmarks.clone();
        bookmarks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        bookmarks
    }

    pub fn history(&self) -> Vec<Visit> {
        self.lock().visits.iter().rev().take(100).cloned().collect()
    }

    pub fn settings(&self) -> BrowserSettings {
        self.lock().settings.clone()
    }

    pub fn session(&self) -> BrowserSession {
        self.lock().session.clone().sanitized()
    }

    pub fn save_session(&self, session: BrowserSession) -> Result<(), String> {
        let mut data = self.lock();
        data.session = session.sanitized();
        self.save(&data)
    }

    pub fn save_settings(&self, settings: BrowserSettings) -> Result<(), String> {
        let mut data = self.lock();
        let previous = std::mem::replace(&mut data.settings, settings.sanitized());
        if let Err(error) = self.save(&data) {
            data.settings = previous;
            return Err(error);
        }
        Ok(())
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let mut data = self.lock();
        data.visits.clear();
        self.save(&data)
    }

    pub fn record_visit(&self, title: String, url: String) -> Result<(), String> {
        let mut data = self.lock();
        let visited_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        if let Some(last) = data.visits.last_mut().filter(|last| last.url == url) {
            last.title = title;
            last.visited_at = visited_at;
        } else {
            data.visits.push(Visit { title, url, visited_at });
            if data.visits.len() > MAX_HISTORY_ENTRIES {
                let excess = data.visits.len() - MAX_HISTORY_ENTRIES;
                data.visits.drain(0..excess);
            }
        }
        self.save(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_compatible_with_old_and_invalid_profiles() {
        let missing: BrowserSettings = serde_json::from_str(r#"{"theme":"dark"}"#).unwrap();
        assert_eq!(missing.sanitized().homepage, BrowserSettings::default().homepage);
        let invalid: BrowserSettings = serde_json::from_str(
            r#"{"theme":"midnight","homepage":"javascript:alert(1)"}"#
        ).unwrap();
        assert_eq!(invalid.sanitized(), BrowserSettings::default());
        let search: BrowserSettings = serde_json::from_str(
            r#"{"theme":"dark","homepage":"hello browser"}"#
        ).unwrap();
        assert!(search.sanitized().homepage.starts_with("https://www.google.com/search?q="));
    }

    #[test]
    fn expanded_preferences_migrate_old_profiles() {
        let old: BrowserSettings = serde_json::from_str(
            r#"{"theme":"dark","homepage":"https://example.com/"}"#
        ).unwrap();
        let settings = old.sanitized();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.startup, "restore");
        assert_eq!(settings.new_tab, "homepage");
        assert_eq!(settings.search_engine, "google");
        assert!(settings.save_history);
    }

    #[test]
    fn expanded_preferences_survive_a_profile_round_trip() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("nova-preferences-{}-{stamp}.json", std::process::id()));
        let store = BrowserDataStore::load(path.clone());
        let changed = BrowserSettings {
            theme: "system".into(),
            homepage: "https://example.org/".into(),
            startup: "homepage".into(),
            new_tab: "blank".into(),
            search_engine: "duckduckgo".into(),
            save_history: false,
        };
        store.save_settings(changed.clone()).unwrap();
        let recovered = BrowserDataStore::load(path.clone());
        assert_eq!(recovered.settings(), changed);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_preferences_fall_back_without_discarding_good_values() {
        let corrupt: BrowserSettings = serde_json::from_str(
            r#"{"theme":"system","homepage":"https://example.com/","startup":"random","newTab":"unknown","searchEngine":"unknown","saveHistory":false}"#
        ).unwrap();
        let safe = corrupt.sanitized();
        assert_eq!(safe.theme, "system");
        assert_eq!(safe.startup, "restore");
        assert_eq!(safe.new_tab, "homepage");
        assert_eq!(safe.search_engine, "google");
        assert!(!safe.save_history);
    }

    #[test]
    fn a_failed_settings_save_keeps_previous_in_memory_value() {
        // A directory cannot be used as a regular file on Windows/Linux.
        let store = BrowserDataStore::load(std::env::temp_dir());
        let previous = store.settings();
        let error = store.save_settings(BrowserSettings {
            theme: "dark".into(),
            homepage: "https://www.rust-lang.org/".into(),
            ..BrowserSettings::default()
        });
        assert!(error.is_err());
        assert_eq!(store.settings(), previous);
    }

    #[test]
    fn restores_tabs_from_existing_profile_without_losing_bookmarks() {
        let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("nova-session-{}-{suffix}.json", std::process::id()));
        let store = BrowserDataStore::load(path.clone());
        store.toggle_bookmark("Example".into(), "https://example.com/".into()).unwrap();
        store.save_session(BrowserSession {
            tabs: vec![
                SessionTab { title: "First".into(), url: "https://example.com/".into() },
                SessionTab { title: "Second".into(), url: "https://www.rust-lang.org/".into() },
            ],
            active_index: 1,
        }).unwrap();
        let recovered = BrowserDataStore::load(path.clone());
        assert_eq!(recovered.session().active_index, 1);
        assert_eq!(recovered.session().tabs.len(), 2);
        assert_eq!(recovered.session().tabs[1].title, "Second");
        assert!(recovered.is_bookmarked("https://example.com/"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_and_excessive_tabs_are_filtered() {
        let session = BrowserSession {
            tabs: std::iter::once(SessionTab {
                title: "Unsafe".into(), url: "javascript:alert(1)".into(),
            }).chain((0..20).map(|i| SessionTab {
                title: format!("Page {i}"), url: format!("https://example.com/{i}"),
            })).collect(),
            active_index: 100,
        }.sanitized();
        assert_eq!(session.tabs.len(), MAX_TABS);
        assert_eq!(session.active_index, MAX_TABS - 1);
        assert!(session.tabs.iter().all(|tab| tab.url.starts_with("https://")));
    }

    #[test]
    fn preserves_active_tab_when_invalid_entries_are_removed() {
        let session = BrowserSession {
            tabs: vec![
                SessionTab { title: "Bad".into(), url: "file:///etc/passwd".into() },
                SessionTab { title: "A".into(), url: "https://a.example/".into() },
                SessionTab { title: "B".into(), url: "https://b.example/".into() },
            ],
            active_index: 2,
        }.sanitized();
        assert_eq!(session.active_index, 1);
        assert_eq!(session.tabs[session.active_index].title, "B");
    }

    #[test]
    fn supports_legacy_profile_without_session_field() {
        let old = r#"{"bookmarks":[],"visits":[],"settings":{"theme":"dark","homepage":"https://example.com/"}}"#;
        let profile: BrowserData = serde_json::from_str(old).unwrap();
        assert_eq!(profile.settings.theme, "dark");
        assert!(profile.session.tabs.is_empty());
    }
}
