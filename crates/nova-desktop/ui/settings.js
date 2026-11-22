/* Dedicated native NovaBrowser settings page.
 * This is chrome UI rendered by Tauri/WebView2, not a website in Obscura.
 */
(() => {
  "use strict";

  function create({ invoke, onOpen, onClose, onRoute }) {
    const page = document.getElementById("settings-page");
    const form = document.getElementById("settings-form");
    const theme = document.getElementById("theme-setting");
    const homepage = document.getElementById("homepage-setting");
    const startup = document.getElementById("startup-setting");
    const newTab = document.getElementById("new-tab-setting");
    const searchEngine = document.getElementById("search-engine-setting");
    const saveHistory = document.getElementById("save-history-setting");
    const error = document.getElementById("settings-error");
    const feedback = document.getElementById("settings-feedback");
    const saveButton = document.getElementById("save-settings");
    const clearHistory = document.getElementById("settings-clear-history");
    const historyMessage = document.getElementById("settings-history-message");
    const sectionLabel = document.getElementById("settings-section-label");
    const navigation = Array.from(page.querySelectorAll("[data-settings-section]"));
    const sections = Array.from(page.querySelectorAll("[data-settings-panel]"));
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)");
    const labels = {
      general: "General", appearance: "Appearance", search: "Search engine",
      tabs: "Tabs", privacy: "Privacy & data", about: "About NovaBrowser",
    };
    let persisted = null;
    let pending = false;
    let section = "general";
    let previewTheme = "light";

    function applyTheme(value) {
      previewTheme = value;
      const resolved = value === "system"
        ? (systemTheme.matches ? "dark" : "light")
        : value;
      document.documentElement.dataset.theme = resolved === "dark" ? "dark" : "light";
    }

    systemTheme.addEventListener("change", () => {
      if (previewTheme === "system") applyTheme("system");
    });

    function fill(settings) {
      theme.value = settings.theme || "light";
      homepage.value = settings.homepage || "https://example.com/";
      startup.value = settings.startup || "restore";
      newTab.value = settings.newTab || "homepage";
      searchEngine.value = settings.searchEngine || "google";
      saveHistory.checked = settings.saveHistory !== false;
      homepage.removeAttribute("aria-invalid");
      error.hidden = true;
      error.textContent = "";
      feedback.textContent = "";
      applyTheme(theme.value);
    }

    function selectSection(name) {
      section = Object.hasOwn(labels, name) ? name : "general";
      for (const button of navigation) {
        const selected = button.dataset.settingsSection === section;
        button.classList.toggle("is-active", selected);
        if (selected) button.setAttribute("aria-current", "page");
        else button.removeAttribute("aria-current");
      }
      for (const panel of sections) panel.hidden = panel.dataset.settingsPanel !== section;
      sectionLabel.textContent = labels[section];
      document.getElementById("settings-content").scrollTop = 0;
      if (!page.hidden) onRoute(section);
    }

    async function open(name = "general") {
      if (pending) return;
      if (!page.hidden) {
        selectSection(name);
        return;
      }
      const fresh = await invoke("browser_get_settings");
      persisted = { ...fresh };
      fill(persisted);
      historyMessage.textContent = "";
      page.hidden = false;
      onOpen();
      selectSection(name);
      const selected = navigation.find((button) => button.dataset.settingsSection === section);
      selected?.focus();
    }

    function close() {
      if (page.hidden || pending) return;
      if (persisted) applyTheme(persisted.theme);
      page.hidden = true;
      error.hidden = true;
      feedback.textContent = "";
      onClose();
    }

    function showError(message, invalidHomepage = false) {
      error.textContent = String(message);
      error.hidden = false;
      feedback.textContent = "";
      if (invalidHomepage) {
        homepage.setAttribute("aria-invalid", "true");
        selectSection("general");
        homepage.focus();
      }
    }

    for (const button of navigation) {
      button.addEventListener("click", () => selectSection(button.dataset.settingsSection));
    }
    document.getElementById("close-settings").addEventListener("click", close);
    document.getElementById("reset-settings").addEventListener("click", () => {
      if (!persisted || pending) return;
      fill(persisted);
      feedback.textContent = "Unsaved changes discarded.";
    });
    theme.addEventListener("change", () => {
      applyTheme(theme.value);
      feedback.textContent = "";
    });
    for (const control of [homepage, startup, newTab, searchEngine, saveHistory]) {
      control.addEventListener("change", () => { feedback.textContent = ""; error.hidden = true; });
    }
    homepage.addEventListener("input", () => {
      homepage.removeAttribute("aria-invalid");
      error.hidden = true;
      feedback.textContent = "";
    });

    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      if (pending || !persisted) return;
      const url = homepage.value.trim();
      if (!url) { showError("Enter a homepage URL or search query.", true); return; }
      if (new TextEncoder().encode(url).length > 4096) {
        showError("Homepage is too long (maximum 4,096 bytes).", true);
        return;
      }
      pending = true;
      saveButton.disabled = true;
      error.hidden = true;
      feedback.textContent = "";
      try {
        const settings = await invoke("browser_save_settings", {
          theme: theme.value,
          homepage: url,
          startup: startup.value,
          newTab: newTab.value,
          searchEngine: searchEngine.value,
          saveHistory: saveHistory.checked,
        });
        persisted = { ...settings };
        fill(persisted);
        feedback.textContent = "Your settings have been saved.";
      } catch (cause) {
        const message = String(cause);
        showError(message, /home|address|url|http/i.test(message));
      } finally {
        pending = false;
        saveButton.disabled = false;
      }
    });

    clearHistory.addEventListener("click", async () => {
      if (pending || page.hidden) return;
      if (!window.confirm("Clear all recorded browsing history? Bookmarks will be kept.")) return;
      clearHistory.disabled = true;
      historyMessage.textContent = "";
      try {
        await invoke("browser_clear_history");
        historyMessage.textContent = "Browsing history cleared. Bookmarks were not changed.";
      } catch (cause) {
        historyMessage.textContent = "Unable to clear history: " + String(cause);
      } finally {
        clearHistory.disabled = false;
      }
    });

    return Object.freeze({
      open, close, isOpen: () => !page.hidden,
      applySaved(settings) {
        persisted = { ...settings };
        if (page.hidden) applyTheme(settings.theme || "light");
      },
    });
  }

  window.NovaSettings = Object.freeze({ create });
})();
