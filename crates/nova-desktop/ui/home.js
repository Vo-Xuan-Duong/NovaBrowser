/* Native, offline home/new-tab page for the NovaBrowser desktop chrome.
 * Web pages still load through the Obscura engine, never in this WebView2 UI.
 */
(() => {
  "use strict";

  const SUGGESTIONS = Object.freeze([
    { title: "GitHub", url: "https://github.com/" },
    { title: "Wikipedia", url: "https://www.wikipedia.org/" },
    { title: "MDN Web Docs", url: "https://developer.mozilla.org/" },
    { title: "YouTube", url: "https://www.youtube.com/" },
  ]);

  function create({ invoke, i18n, onNavigate, onBookmarks, onHistory, onCustomize, onError }) {
    const page = document.getElementById("home-page");
    const form = document.getElementById("home-search-form");
    const search = document.getElementById("home-search-input");
    const list = document.getElementById("home-shortcuts-list");
    const recent = document.getElementById("home-recent-list");
    const provider = document.getElementById("home-search-provider");
    const greeting = document.getElementById("home-greeting");
    const today = document.getElementById("home-today");
    const shortcutsTitle = document.getElementById("home-shortcuts-title");
    let generation = 0;
    let currentSearchEngine = "google";

    const allowed = (url) => {
      try {
        return ["https:", "http:"].includes(new URL(url).protocol);
      } catch {
        return false;
      }
    };

    function greetingText() {
      const hour = new Date().getHours();
      return i18n.t(hour < 12 ? "Good morning" : hour < 18 ? "Good afternoon" : "Good evening");
    }

    function updateClock() {
      greeting.textContent = greetingText();
      today.textContent = new Intl.DateTimeFormat(i18n.locale(), {
        weekday: "long", month: "long", day: "numeric",
      }).format(new Date());
    }

    function updateProvider() {
      const engines = { google: "Google", bing: "Bing", duckduckgo: "DuckDuckGo" };
      provider.textContent = i18n.t("Search with {provider}", {
        provider: engines[currentSearchEngine] || "Google",
      });
    }

    function makeShortcut(item) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "home-shortcut";
      button.title = item.url;
      button.setAttribute("aria-label", item.title || item.url);

      const avatar = document.createElement("span");
      avatar.className = "home-shortcut-avatar";
      const label = item.title || new URL(item.url).hostname;
      avatar.textContent = label.trim().charAt(0).toLocaleUpperCase();
      const name = document.createElement("span");
      name.className = "home-shortcut-name";
      name.textContent = label;
      button.append(avatar, name);
      button.addEventListener("click", () => onNavigate(item.url));
      return button;
    }

    function renderShortcuts(bookmarks) {
      const favorites = Array.isArray(bookmarks)
        ? bookmarks.filter((item) => allowed(item.url)).slice(0, 6)
        : [];
      const items = favorites.length ? favorites : SUGGESTIONS;
      shortcutsTitle.textContent = i18n.t(favorites.length ? "Your bookmarks" : "Suggested sites");
      list.replaceChildren(...items.map(makeShortcut));
    }

    function renderRecent(visits) {
      const entries = [];
      const seen = new Set();
      for (const visit of Array.isArray(visits) ? visits : []) {
        if (!allowed(visit.url) || seen.has(visit.url)) continue;
        entries.push(visit);
        seen.add(visit.url);
        if (entries.length === 4) break;
      }
      recent.replaceChildren();
      if (!entries.length) {
        const empty = document.createElement("p");
        empty.className = "home-empty";
        empty.textContent = i18n.t("Pages you visit will appear here.");
        recent.append(empty);
        return;
      }
      for (const entry of entries) {
        const button = document.createElement("button");
        button.type = "button";
        button.className = "home-recent-entry";
        const icon = document.createElement("span");
        icon.className = "home-recent-icon";
        icon.append(window.NovaIcons.create("globe"));
        const content = document.createElement("span");
        content.className = "home-recent-text";
        const title = document.createElement("strong");
        title.textContent = entry.title || new URL(entry.url).hostname;
        const details = document.createElement("small");
        details.textContent = new URL(entry.url).hostname;
        content.append(title, details);
        const arrow = document.createElement("span");
        arrow.className = "home-recent-arrow";
        arrow.append(window.NovaIcons.create("arrow-right"));
        button.append(icon, content, arrow);
        button.title = entry.url;
        button.addEventListener("click", () => onNavigate(entry.url));
        recent.append(button);
      }
    }

    async function refresh() {
      if (page.hidden) return;
      const request = ++generation;
      updateClock();
      updateProvider();
      const [bookmarksResult, historyResult] = await Promise.allSettled([
        invoke("browser_get_bookmarks"), invoke("browser_get_history"),
      ]);
      if (page.hidden || request !== generation) return;
      renderShortcuts(bookmarksResult.status === "fulfilled" ? bookmarksResult.value : []);
      renderRecent(historyResult.status === "fulfilled" ? historyResult.value : []);
      if (bookmarksResult.status === "rejected" || historyResult.status === "rejected") {
        onError(i18n.t("Some Home content could not be loaded."));
      }
    }

    function show() {
      if (!page.hidden) return;
      page.hidden = false;
      void refresh();
    }

    function hide() {
      ++generation;
      page.hidden = true;
    }

    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const input = search.value.trim();
      if (!input) {
        search.focus();
        return;
      }
      onNavigate(input);
      search.value = "";
    });

    document.getElementById("home-open-bookmarks").addEventListener("click", onBookmarks);
    document.getElementById("home-open-history").addEventListener("click", onHistory);
    document.getElementById("home-customize").addEventListener("click", onCustomize);

    i18n.onChange(() => {
      updateClock();
      updateProvider();
      if (!page.hidden) void refresh();
    });

    return Object.freeze({
      show, hide, refresh,
      isOpen: () => !page.hidden,
      focusSearch: () => search.focus(),
      setPreferences(settings) {
        currentSearchEngine = settings?.searchEngine || "google";
        updateProvider();
      },
    });
  }

  window.NovaHome = Object.freeze({ create });
})();
