/* NovaBrowser's native Settings page; intentionally offline and framework-free. */
(() => {
  "use strict";

  function create({ invoke, onOpen, onClose, onRoute, onSaved }) {
    const i18n = window.NovaI18n;
    const page = document.getElementById("settings-page");
    const form = document.getElementById("settings-form");
    const themeChoices = [...page.querySelectorAll('input[name="appearance"]')];
    const homepage = document.getElementById("homepage-setting");
    const startup = document.getElementById("startup-setting");
    const newTab = document.getElementById("new-tab-setting");
    const searchEngine = document.getElementById("search-engine-setting");
    const saveHistory = document.getElementById("save-history-setting");
    const showHome = document.getElementById("show-home-setting");
    const showBookmarks = document.getElementById("show-bookmarks-setting");
    const language = document.getElementById("language-setting");
    const error = document.getElementById("settings-error");
    const feedback = document.getElementById("settings-feedback");
    const saveButton = document.getElementById("save-settings");
    const resetButton = document.getElementById("reset-settings");
    const clearHistory = document.getElementById("settings-clear-history");
    const clearBookmarks = document.getElementById("settings-clear-bookmarks");
    const historyMessage = document.getElementById("settings-history-message");
    const sectionLabel = document.getElementById("settings-section-label");
    const searchInput = document.getElementById("settings-search");
    const searchSummary = document.getElementById("settings-search-summary");
    const content = document.getElementById("settings-content");
    const navigation = [...page.querySelectorAll("[data-settings-section]")];
    const sections = [...page.querySelectorAll("[data-settings-panel]")];
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
    const labels = {
      general: "General", appearance: "Appearance", language: "Language & region",
      search: "Search engine", tabs: "Tabs",
      privacy: "Privacy & data", about: "About NovaBrowser",
    };
    let persisted = null;
    let pending = false;
    let section = "general";
    let previewTheme = "light";

    function selectedTheme() {
      return themeChoices.find((choice) => choice.checked)?.value || "light";
    }

    function applyTheme(value) {
      previewTheme = value;
      const actual = value === "system"
        ? (systemTheme.matches ? "dark" : "light")
        : value;
      document.documentElement.dataset.theme = actual === "dark" ? "dark" : "light";
    }

    systemTheme.addEventListener("change", () => {
      if (previewTheme === "system") applyTheme("system");
    });

    function readState() {
      return {
        theme: selectedTheme(),
        homepage: homepage.value.trim(),
        startup: startup.value,
        newTab: newTab.value,
        searchEngine: searchEngine.value,
        saveHistory: saveHistory.checked,
        showHomeButton: showHome.checked,
        showBookmarksBar: showBookmarks.checked,
        language: language.value,
      };
    }

    function isDirty() {
      if (!persisted) return false;
      return Object.entries(readState()).some(([key, value]) => value !== persisted[key]);
    }

    function updateSaveState() {
      saveButton.disabled = pending || !isDirty();
      resetButton.disabled = pending || !isDirty();
    }

    function updateSearchExample() {
      const names = { google: "Google Search", bing: "Bing Search", duckduckgo: "DuckDuckGo" };
      document.getElementById("settings-search-example").textContent = i18n.t(names[searchEngine.value] || "Google Search");
    }

    function fill(settings) {
      const target = settings.theme || "light";
      const radio = themeChoices.find((choice) => choice.value === target) || themeChoices[1];
      for (const choice of themeChoices) choice.checked = choice === radio;
      homepage.value = settings.homepage || "https://example.com/";
      startup.value = settings.startup || "restore";
      newTab.value = settings.newTab || "homepage";
      searchEngine.value = settings.searchEngine || "google";
      saveHistory.checked = settings.saveHistory !== false;
      showHome.checked = settings.showHomeButton === true;
      showBookmarks.checked = settings.showBookmarksBar === true;
      language.value = ["system", "en", "vi"].includes(settings.language) ? settings.language : "system";
      i18n.setLanguage(language.value);
      homepage.removeAttribute("aria-invalid");
      error.hidden = true;
      error.textContent = "";
      feedback.textContent = "";
      historyMessage.textContent = "";
      applyTheme(selectedTheme());
      updateSearchExample();
      updateSaveState();
    }

    function selectSection(name, { clearSearch = false } = {}) {
      section = Object.hasOwn(labels, name) ? name : "general";
      if (clearSearch) {
        searchInput.value = "";
        filterNavigation();
      }
      for (const button of navigation) {
        const active = button.dataset.settingsSection === section;
        button.classList.toggle("is-active", active);
        if (active) button.setAttribute("aria-current", "page");
        else button.removeAttribute("aria-current");
      }
      for (const panel of sections) panel.hidden = panel.dataset.settingsPanel !== section;
      sectionLabel.textContent = i18n.t(labels[section]);
      content.scrollTop = 0;
      if (!page.hidden) onRoute(section);
    }

    function filterNavigation() {
      const query = searchInput.value.toLocaleLowerCase().trim();
      let matches = 0;
      let first = null;
      for (const button of navigation) {
        const name = button.dataset.settingsSection;
        const panel = sections.find((candidate) => candidate.dataset.settingsPanel === name);
        const haystack = (labels[name] + " " + i18n.t(labels[name]) + " " + (panel?.textContent || "")).toLocaleLowerCase();
        const match = !query || haystack.includes(query);
        button.hidden = !match;
        if (match) {
          matches += 1;
          if (!first) first = name;
        }
      }
      searchSummary.textContent = !query ? "" : matches
        ? matches === 1 ? i18n.t("1 matching section") : i18n.t("{count} matching sections", { count: matches })
        : i18n.t("No matching settings");
      if (query && first && navigation.find((button) => button.dataset.settingsSection === section)?.hidden) {
        selectSection(first);
      }
    }

    async function open(name = "general") {
      if (pending) return;
      if (!page.hidden) {
        selectSection(name, { clearSearch: true });
        return;
      }
      const fresh = await invoke("browser_get_settings");
      persisted = { ...fresh };
      fill(persisted);
      searchInput.value = "";
      filterNavigation();
      page.hidden = false;
      onOpen();
      selectSection(name);
    }

    function close() {
      if (page.hidden || pending) return;
      if (persisted) {
        applyTheme(persisted.theme);
        i18n.setLanguage(persisted.language || "system");
      }
      page.hidden = true;
      feedback.textContent = "";
      error.hidden = true;
      searchInput.value = "";
      onClose();
    }

    function showError(message, invalidHomepage = false) {
      error.textContent = String(message);
      error.hidden = false;
      feedback.textContent = "";
      if (invalidHomepage) {
        homepage.setAttribute("aria-invalid", "true");
        selectSection("general", { clearSearch: true });
        homepage.focus();
      }
    }

    for (const button of navigation) {
      button.addEventListener("click", () => selectSection(button.dataset.settingsSection, { clearSearch: true }));
    }
    searchInput.addEventListener("input", filterNavigation);
    searchInput.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        const first = navigation.find((button) => !button.hidden);
        if (first) {
          event.preventDefault();
          selectSection(first.dataset.settingsSection, { clearSearch: true });
          first.focus();
        }
      }
    });
    document.getElementById("close-settings").addEventListener("click", close);
    resetButton.addEventListener("click", () => {
      if (!persisted || pending) return;
      fill(persisted);
      feedback.textContent = i18n.t("Changes discarded.");
    });

    function onChange() {
      error.hidden = true;
      feedback.textContent = "";
      updateSaveState();
    }
    for (const choice of themeChoices) {
      choice.addEventListener("change", () => {
        applyTheme(selectedTheme());
        onChange();
      });
    }
    for (const control of [homepage, startup, newTab, searchEngine, saveHistory, showHome, showBookmarks]) {
      control.addEventListener("change", onChange);
    }
    homepage.addEventListener("input", () => {
      homepage.removeAttribute("aria-invalid");
      onChange();
    });
    searchEngine.addEventListener("change", updateSearchExample);
    language.addEventListener("change", () => {
      i18n.setLanguage(language.value);
      sectionLabel.textContent = i18n.t(labels[section]);
      updateSearchExample();
      filterNavigation();
      onChange();
    });


    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      if (pending || !persisted || !isDirty()) return;
      const next = readState();
      if (!next.homepage) { showError(i18n.t("Enter a homepage URL or search query."), true); return; }
      if (new TextEncoder().encode(next.homepage).length > 4096) {
        showError(i18n.t("Homepage is too long (maximum 4,096 bytes)."), true);
        return;
      }
      pending = true;
      updateSaveState();
      error.hidden = true;
      feedback.textContent = "";
      try {
        const saved = await invoke("browser_save_settings", next);
        persisted = { ...saved };
        fill(persisted);
        onSaved(persisted);
        feedback.textContent = i18n.t("Changes saved.");
      } catch (cause) {
        const message = String(cause);
        showError(i18n.t("Unable to save settings: {error}", { error: message }),
          /home|address|url|http/i.test(message));
      } finally {
        pending = false;
        updateSaveState();
      }
    });

    async function clearItems(kind) {
      if (pending || page.hidden) return;
      const isBookmarks = kind === "bookmarks";
      const label = isBookmarks ? i18n.t("bookmarks") : i18n.t("browsing history");
      const confirmText = isBookmarks
        ? i18n.t("Permanently clear all bookmarks from this browser?")
        : i18n.t("Permanently clear all browsing history from this browser?");
      if (!window.confirm(confirmText)) return;
      const button = isBookmarks ? clearBookmarks : clearHistory;
      button.disabled = true;
      historyMessage.textContent = "";
      try {
        await invoke(isBookmarks ? "browser_clear_bookmarks" : "browser_clear_history");
        historyMessage.textContent = i18n.t(isBookmarks ? "Bookmarks cleared." : "Browsing history cleared.");
        onSaved(persisted);
      } catch (cause) {
        historyMessage.textContent = i18n.t("Unable to clear {item}: {error}", { item: label, error: String(cause) });
      } finally {
        button.disabled = false;
      }
    }

    clearHistory.addEventListener("click", () => void clearItems("history"));
    clearBookmarks.addEventListener("click", () => void clearItems("bookmarks"));

    return Object.freeze({
      open, close, isOpen: () => !page.hidden,
      applySaved(settings) {
        persisted = { ...settings };
        if (page.hidden) {
          applyTheme(settings.theme || "light");
          i18n.setLanguage(settings.language || "system");
        }
        onSaved(persisted);
      },
    });
  }

  window.NovaSettings = Object.freeze({ create });
})();