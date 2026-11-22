//! The browser-owned boundary to a website renderer.
//!
//! NovaBrowser owns tabs, navigation UI, bookmarks, profile and preferences.
//! A render engine owns only website WebViews, input and page events.
//! This module is intentionally independent of Servo and Obscura types: the
//! same NovaBrowser UI must survive swapping the underlying web renderer.
//!
//! The current Obscura page pipeline has not yet been ported to this API.
//! This contract is for incremental migration and does not claim Servo tabs
//! are already embedded in the NovaBrowser desktop.

use url::Url;

/// A stable NovaBrowser tab identifier, not an engine's internal WebView ID.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TabId(u64);

impl TabId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Pixel rectangle reserved for website content below trusted browser chrome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ViewBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ViewBounds {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        (width != 0 && height != 0).then_some(Self { x, y, width, height })
    }
}

/// Native input (not synthetic DOM JavaScript). The eventual Servo adapter
/// maps these to WebView::notify_input_event and the native event loop.
#[derive(Clone, Debug, PartialEq)]
pub enum PageInput {
    PointerMove { x: f32, y: f32 },
    PointerDown { x: f32, y: f32, button: u8 },
    PointerUp { x: f32, y: f32, button: u8 },
    Wheel { x: f32, y: f32, dx: f32, dy: f32 },
    KeyDown { key: String },
    KeyUp { key: String },
    Text(String),
}

/// Site-to-shell updates. Never grant a remote origin access to a privileged
/// NovaBrowser/Tauri command simply to implement a navigation event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PageEvent {
    Loading { tab: TabId, url: String },
    Navigated { tab: TabId, url: String },
    Title { tab: TabId, title: String },
    History { tab: TabId, can_back: bool, can_forward: bool },
    FrameReady { tab: TabId },
    Failed { tab: TabId, message: String },
}

/// An implementation owns Servo (or another engine) WebView handles.
/// WebViews and their render contexts remain on the embedder's event-loop
/// thread. The browser shell does not access the engine's JavaScript realm.
pub trait WebsiteEngine {
    type Error;

    fn create_view(&mut self, tab: TabId, bounds: ViewBounds) -> Result<(), Self::Error>;
    fn navigate(&mut self, tab: TabId, url: &Url) -> Result<(), Self::Error>;
    fn back(&mut self, tab: TabId) -> Result<(), Self::Error>;
    fn forward(&mut self, tab: TabId) -> Result<(), Self::Error>;
    fn reload(&mut self, tab: TabId) -> Result<(), Self::Error>;
    fn set_visible(&mut self, tab: TabId, visible: bool) -> Result<(), Self::Error>;
    fn resize(&mut self, tab: TabId, bounds: ViewBounds) -> Result<(), Self::Error>;
    fn input(&mut self, tab: TabId, event: PageInput) -> Result<(), Self::Error>;
    fn close_view(&mut self, tab: TabId);
}

/// External pages never receive a nova:// privileged internal route.
/// Existing omnibox search normalization stays in navigation.rs.
pub fn external_page_url(address: &str) -> Result<Url, &'static str> {
    if address.len() > 4096 || address.chars().any(char::is_control) {
        return Err("Invalid web address");
    }
    let url = Url::parse(address).map_err(|_| "Invalid web address")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("Only HTTP and HTTPS pages are supported");
    }
    Ok(url)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabRecord {
    pub id: TabId,
    pub title: String,
    /// Includes internal routes like nova://home, which stay in the shell.
    pub address: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabError {
    TooManyTabs,
    UnknownTab,
    CannotCloseLastTab,
    TabIdExhausted,
}

/// NovaBrowser-side tab metadata, independent from page engine handles.
/// The existing storage model can later serialize these records unchanged.
pub struct TabRegistry {
    tabs: Vec<TabRecord>,
    active: Option<TabId>,
    next_id: u64,
    maximum: usize,
}

impl TabRegistry {
    pub fn new(maximum: usize) -> Self {
        assert!(maximum > 0, "a browser must allow at least one tab");
        Self { tabs: Vec::new(), active: None, next_id: 1, maximum }
    }

    pub fn tabs(&self) -> &[TabRecord] {
        &self.tabs
    }

    pub fn active(&self) -> Option<TabId> {
        self.active
    }

    pub fn create(&mut self, address: impl Into<String>) -> Result<TabId, TabError> {
        if self.tabs.len() >= self.maximum {
            return Err(TabError::TooManyTabs);
        }
        let next = self.next_id.checked_add(1).ok_or(TabError::TabIdExhausted)?;
        let id = TabId(self.next_id);
        self.next_id = next;
        self.tabs.push(TabRecord { id, title: "New tab".into(), address: address.into() });
        self.active = Some(id);
        Ok(id)
    }

    pub fn activate(&mut self, id: TabId) -> Result<(), TabError> {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return Err(TabError::UnknownTab);
        }
        self.active = Some(id);
        Ok(())
    }

    pub fn update(&mut self, id: TabId, address: String, title: String) -> Result<(), TabError> {
        let tab = self.tabs.iter_mut().find(|tab| tab.id == id)
            .ok_or(TabError::UnknownTab)?;
        tab.address = address;
        tab.title = title;
        Ok(())
    }

    pub fn close(&mut self, id: TabId) -> Result<TabId, TabError> {
        if self.tabs.len() <= 1 {
            return Err(TabError::CannotCloseLastTab);
        }
        let index = self.tabs.iter().position(|tab| tab.id == id)
            .ok_or(TabError::UnknownTab)?;
        self.tabs.remove(index);
        if self.active == Some(id) {
            let replacement = index.saturating_sub(1).min(self.tabs.len() - 1);
            self.active = Some(self.tabs[replacement].id);
        }
        Ok(self.active.expect("nonempty tab list has an active tab"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_owns_tabs_and_can_switch_without_rebuilding_engine() {
        let mut registry = TabRegistry::new(3);
        let first = registry.create("nova://home").unwrap();
        let second = registry.create("https://example.org/").unwrap();
        let third = registry.create("https://www.wikipedia.org/").unwrap();
        assert_eq!([first.value(), second.value(), third.value()], [1, 2, 3]);
        assert_eq!(registry.active(), Some(third));
        assert_eq!(registry.create("https://github.com/"), Err(TabError::TooManyTabs));
        registry.activate(first).unwrap();
        assert_eq!(registry.active(), Some(first));
        assert_eq!(registry.close(second), Ok(first));
        assert_eq!(registry.tabs().len(), 2);
        assert_eq!(registry.close(first), Ok(third));
        assert_eq!(registry.close(third), Err(TabError::CannotCloseLastTab));
    }

    #[test]
    fn refuses_unknown_tabs_and_preserves_existing_metadata() {
        let mut registry = TabRegistry::new(2);
        let first = registry.create("nova://home").unwrap();
        let invalid = TabId(7);
        assert_eq!(registry.activate(invalid), Err(TabError::UnknownTab));
        assert_eq!(registry.update(invalid, "x".into(), "x".into()), Err(TabError::UnknownTab));
        registry.update(first, "nova://settings".into(), "Settings".into()).unwrap();
        assert_eq!(registry.tabs()[0].title, "Settings");
    }

    #[test]
    fn enforces_webview_bounds_and_safe_external_schemes() {
        assert_eq!(ViewBounds::new(0, 42, 1280, 0), None);
        assert_eq!(ViewBounds::new(0, 42, 1280, 720).unwrap().y, 42);
        assert_eq!(external_page_url("https://example.org/").unwrap().scheme(), "https");
        for disallowed in [
            "nova://settings", "file:///C:/Windows", "javascript:alert(1)",
            "data:text/html,hello", "https://example.org/\nother",
        ] {
            assert!(external_page_url(disallowed).is_err(), "{disallowed}");
        }
    }
}
