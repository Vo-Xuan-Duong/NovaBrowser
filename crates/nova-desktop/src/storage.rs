use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_HISTORY_ENTRIES: usize = 500;

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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSettings {
    pub theme: String,
    pub homepage: String,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            homepage: "https://example.com/".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct BrowserData {
    bookmarks: Vec<Bookmark>,
    visits: Vec<Visit>,
    settings: BrowserSettings,
}

pub struct BrowserDataStore {
    path: PathBuf,
    data: Mutex<BrowserData>,
}

impl BrowserDataStore {
    pub fn load(path: PathBuf) -> Self {
        let data = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
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

    pub fn save_settings(&self, settings: BrowserSettings) -> Result<(), String> {
        let mut data = self.lock();
        data.settings = settings;
        self.save(&data)
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
