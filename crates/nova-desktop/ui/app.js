(() => {
  "use strict";

  const { invoke } = window.__TAURI__.core;
  const { listen } = window.__TAURI__.event;
  const appWindow = window.__TAURI__.window.getCurrentWindow();
  const pageArea = document.getElementById("page-area");
  const pageFrame = document.getElementById("page-frame");
  const address = document.getElementById("address");
  const tabList = document.getElementById("tab-list");
  const statusBubble = document.getElementById("status-bubble");
  const statusMessage = document.getElementById("status-message");
  const loadingIndicator = document.getElementById("loading-indicator");
  const bookmarkButton = document.getElementById("bookmark-current");
  const libraryPanel = document.getElementById("library-panel");
  const libraryList = document.getElementById("library-list");
  const libraryTitle = document.getElementById("library-title");
  const historyFooter = document.getElementById("history-footer");
  const settingsPanel = document.getElementById("settings-panel");
  const settingsForm = document.getElementById("settings-form");
  const settingsError = document.getElementById("settings-error");
  const themeSetting = document.getElementById("theme-setting");
  const homepageSetting = document.getElementById("homepage-setting");

  let lastSize = "";
  let libraryMode = "bookmarks";
  let activeTabId = null;
  let currentTitle = "New tab";

  function applyTheme(theme) {
    document.documentElement.dataset.theme = theme === "dark" ? "dark" : "light";
  }

  function renderTabs(tabs) {
    tabList.replaceChildren();
    for (const tab of tabs) {
      const item = document.createElement("div");
      item.className = `tab${tab.active ? " is-active" : ""}`;
      item.setAttribute("role", "tab");
      item.setAttribute("aria-selected", String(tab.active));
      item.tabIndex = tab.active ? 0 : -1;
      item.addEventListener("click", (event) => {
        if (!event.target.closest(".tab-close")) void call("browser_activate_tab", { tabId: tab.id });
      });

      const mark = document.createElement("span");
      mark.className = "brand-mark";
      mark.setAttribute("aria-hidden", "true");
      mark.textContent = "N";
      const title = document.createElement("span");
      title.className = "tab-title";
      title.textContent = tab.title || "New tab";
      if (tab.active) title.id = "tab-title";
      const close = document.createElement("button");
      close.className = "tab-close";
      close.type = "button";
      close.textContent = "×";
      close.title = "Close tab";
      close.setAttribute("aria-label", `Close ${tab.title || "tab"}`);
      close.hidden = !tab.closable;
      close.addEventListener("click", (event) => {
        event.stopPropagation();
        void call("browser_close_tab", { tabId: tab.id });
      });
      item.append(mark, title, close);
      tabList.append(item);

      if (tab.active) {
        activeTabId = tab.id;
        currentTitle = tab.title || "New tab";
        if (document.activeElement !== address) address.value = tab.address || "";
        item.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    }
  }

  function showError(error) {
    statusMessage.textContent = String(error);
    statusBubble.classList.add("is-visible");
  }

  async function call(command, args) {
    try {
      return await invoke(command, args);
    } catch (error) {
      showError(error);
      return undefined;
    }
  }

  document.getElementById("address-form").addEventListener("submit", (event) => {
    event.preventDefault();
    void call("browser_navigate", { address: address.value });
    address.blur();
    pageArea.focus();
  });

  document.getElementById("back").addEventListener("click", () => void call("browser_back"));
  document.getElementById("forward").addEventListener("click", () => void call("browser_forward"));
  document.getElementById("reload").addEventListener("click", () => void call("browser_reload"));

  async function refreshBookmarkState() {
    const url = address.value.trim();
    if (!/^https?:\/\//i.test(url)) {
      bookmarkButton.classList.remove("is-bookmarked");
      return;
    }
    const bookmarked = await call("browser_is_bookmarked", { url });
    bookmarkButton.classList.toggle("is-bookmarked", bookmarked === true);
    bookmarkButton.setAttribute("aria-label", bookmarked ? "Remove bookmark" : "Bookmark this page");
    bookmarkButton.title = bookmarked ? "Remove bookmark" : "Bookmark this page";
  }

  bookmarkButton.addEventListener("click", async () => {
    const url = address.value.trim();
    if (!/^https?:\/\//i.test(url)) return;
    const bookmarked = await call("browser_toggle_bookmark", {
      title: currentTitle || url,
      url,
    });
    if (typeof bookmarked === "boolean") {
      bookmarkButton.classList.toggle("is-bookmarked", bookmarked);
      bookmarkButton.setAttribute("aria-label", bookmarked ? "Remove bookmark" : "Bookmark this page");
      bookmarkButton.title = bookmarked ? "Remove bookmark" : "Bookmark this page";
      if (libraryMode === "bookmarks" && !libraryPanel.hidden) void renderLibrary("bookmarks");
    }
  });

  function closeLibrary() {
    libraryPanel.hidden = true;
  }

  async function renderLibrary(mode) {
    libraryMode = mode;
    const isHistory = mode === "history";
    libraryTitle.textContent = isHistory ? "Browsing history" : "Bookmarks";
    historyFooter.hidden = !isHistory;
    document.getElementById("history-tab").classList.toggle("is-active", isHistory);
    document.getElementById("bookmarks-tab").classList.toggle("is-active", !isHistory);
    libraryPanel.hidden = false;
    libraryList.replaceChildren();

    const entries = await call(isHistory ? "browser_get_history" : "browser_get_bookmarks");
    if (!Array.isArray(entries)) return;
    if (entries.length === 0) {
      const empty = document.createElement("div");
      empty.className = "library-empty";
      empty.textContent = isHistory ? "Pages you visit will appear here" : "Your saved pages will appear here";
      libraryList.append(empty);
      return;
    }

    for (const entry of entries) {
      const row = document.createElement("div");
      row.className = "library-entry";
      const open = document.createElement("button");
      open.className = "entry-open";
      open.type = "button";
      const title = document.createElement("span");
      title.className = "entry-title";
      title.textContent = entry.title || entry.url;
      const subtitle = document.createElement("span");
      subtitle.className = "entry-url";
      subtitle.textContent = isHistory && entry.visitedAt
        ? `${new Date(entry.visitedAt * 1000).toLocaleString()} · ${entry.url}`
        : entry.url;
      open.append(title, subtitle);
      open.addEventListener("click", () => {
        address.value = entry.url;
        closeLibrary();
        void call("browser_navigate", { address: entry.url });
      });
      row.append(open);

      if (!isHistory) {
        const remove = document.createElement("button");
        remove.className = "entry-remove";
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Remove ${entry.title || entry.url}`);
        remove.addEventListener("click", async () => {
          await call("browser_remove_bookmark", { url: entry.url });
          await refreshBookmarkState();
          void renderLibrary("bookmarks");
        });
        row.append(remove);
      }
      libraryList.append(row);
    }
  }

  document.getElementById("show-bookmarks").addEventListener("click", () => {
    closeSettings();
    if (libraryMode === "bookmarks" && !libraryPanel.hidden) closeLibrary();
    else void renderLibrary("bookmarks");
  });
  document.getElementById("show-history").addEventListener("click", () => {
    closeSettings();
    if (libraryMode === "history" && !libraryPanel.hidden) closeLibrary();
    else void renderLibrary("history");
  });
  document.getElementById("history-tab").addEventListener("click", () => void renderLibrary("history"));
  document.getElementById("bookmarks-tab").addEventListener("click", () => void renderLibrary("bookmarks"));
  document.getElementById("close-library").addEventListener("click", closeLibrary);
  document.getElementById("clear-history").addEventListener("click", async () => {
    await call("browser_clear_history");
    void renderLibrary("history");
  });

  async function openSettings() {
    const settings = await call("browser_get_settings");
    if (!settings) return;
    closeLibrary();
    themeSetting.value = settings.theme;
    homepageSetting.value = settings.homepage;
    settingsError.hidden = true;
    settingsPanel.hidden = false;
    homepageSetting.focus();
    homepageSetting.select();
  }

  function closeSettings() {
    settingsPanel.hidden = true;
  }

  document.getElementById("show-settings").addEventListener("click", () => void openSettings());
  document.getElementById("close-settings").addEventListener("click", closeSettings);
  document.getElementById("cancel-settings").addEventListener("click", closeSettings);
  settingsForm.addEventListener("submit", async (event) => {
    event.preventDefault();
    settingsError.hidden = true;
    try {
      const saved = await invoke("browser_save_settings", {
        theme: themeSetting.value,
        homepage: homepageSetting.value,
      });
      applyTheme(saved.theme);
      address.value = saved.homepage;
      closeSettings();
      await call("browser_navigate", { address: saved.homepage });
    } catch (error) {
      settingsError.textContent = String(error);
      settingsError.hidden = false;
    }
  });

  document.addEventListener("click", (event) => {
    if (!libraryPanel.hidden && !libraryPanel.contains(event.target)
        && !event.target.closest("#show-bookmarks, #show-history")) closeLibrary();
    if (!settingsPanel.hidden && !settingsPanel.contains(event.target)
        && !event.target.closest("#show-settings")) closeSettings();
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeLibrary();
      closeSettings();
    }
  });

  document.getElementById("minimize").addEventListener("click", () => void appWindow.minimize());
  document.getElementById("maximize").addEventListener("click", () => void appWindow.toggleMaximize());
  document.getElementById("close").addEventListener("click", () => void appWindow.close());

  const titlebar = document.getElementById("titlebar");
  titlebar.addEventListener("mousedown", (event) => {
    if (event.button === 0 && !event.target.closest(".tab, #new-tab, .window-controls")) {
      void appWindow.startDragging();
    }
  });
  titlebar.addEventListener("dblclick", (event) => {
    if (!event.target.closest(".tab, #new-tab, .window-controls")) void appWindow.toggleMaximize();
  });

  document.getElementById("new-tab").addEventListener("click", () => void call("browser_new_tab"));

  pageArea.addEventListener("click", (event) => {
    pageArea.focus();
    const rect = pageArea.getBoundingClientRect();
    void call("browser_click", {
      x: (event.clientX - rect.left) / rect.width,
      y: (event.clientY - rect.top) / rect.height,
    });
  });

  pageArea.addEventListener("wheel", (event) => {
    event.preventDefault();
    const rect = pageArea.getBoundingClientRect();
    void call("browser_scroll", {
      x: (event.clientX - rect.left) / rect.width,
      y: (event.clientY - rect.top) / rect.height,
      dx: event.deltaX,
      dy: event.deltaY,
    });
  }, { passive: false });

  pageArea.addEventListener("keydown", (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    let key = event.key;
    if (key === "Backspace") key = "__NOVA_BACKSPACE__";
    else if (key === "Delete") key = "__NOVA_DELETE__";
    else if (key === "Enter") key = "__NOVA_ENTER__";
    else if (key.length !== 1 && !(Array.from(key).length === 1 && key.codePointAt(0) > 0xffff)) return;
    else if (key.charCodeAt(0) < 32) return;
    event.preventDefault();
    void call("browser_type", { key });
  });

  window.addEventListener("keydown", (event) => {
    if (event.ctrlKey && event.key.toLowerCase() === "t") {
      event.preventDefault();
      void call("browser_new_tab");
    } else if (event.ctrlKey && event.key.toLowerCase() === "w") {
      event.preventDefault();
      if (activeTabId) void call("browser_close_tab", { tabId: activeTabId });
    } else if (event.ctrlKey && event.key.toLowerCase() === "l") {
      event.preventDefault();
      address.focus();
      address.select();
    } else if (event.ctrlKey && event.key.toLowerCase() === "d") {
      event.preventDefault();
      bookmarkButton.click();
    } else if (event.ctrlKey && event.key.toLowerCase() === "h") {
      event.preventDefault();
      void renderLibrary("history");
    } else if (event.ctrlKey && event.key === ",") {
      event.preventDefault();
      void openSettings();
    } else if (event.ctrlKey && event.shiftKey && event.key.toLowerCase() === "b") {
      event.preventDefault();
      void renderLibrary("bookmarks");
    } else if ((event.ctrlKey && event.key.toLowerCase() === "r") || event.key === "F5") {
      event.preventDefault();
      void call("browser_reload");
    }
  });

  function publishSize() {
    const rect = pageArea.getBoundingClientRect();
    const size = `${Math.round(rect.width)}x${Math.round(rect.height)}`;
    if (size === lastSize || rect.width < 1 || rect.height < 1) return Promise.resolve();
    lastSize = size;
    return call("browser_resize", { width: rect.width, height: rect.height });
  }

  new ResizeObserver(publishSize).observe(pageArea);

  async function connectEvents() {
    await Promise.all([
      listen("browser-status", ({ payload }) => {
        if (payload.address && document.activeElement !== address) address.value = payload.address;
        void refreshBookmarkState();
        statusMessage.textContent = payload.message;
        statusBubble.classList.toggle("is-visible", payload.showStatus);
        loadingIndicator.classList.toggle("is-loading", payload.loading);
      }),
      listen("browser-history", ({ payload }) => {
        document.getElementById("back").disabled = !payload.canBack;
        document.getElementById("forward").disabled = !payload.canForward;
      }),
      listen("browser-tabs", ({ payload }) => renderTabs(payload)),
      listen("browser-title", ({ payload }) => {
        currentTitle = payload || "Untitled page";
        const activeLabel = document.querySelector(".tab.is-active .tab-title");
        if (activeLabel) activeLabel.textContent = currentTitle;
        document.title = `${currentTitle} - NovaBrowser`;
        if (libraryMode === "bookmarks" && !libraryPanel.hidden) void renderLibrary("bookmarks");
      }),
      listen("browser-frame", ({ payload }) => {
        pageFrame.src = `data:image/png;base64,${payload}`;
      }),
    ]);
    await publishSize();
    const settings = await call("browser_get_settings");
    if (settings) {
      applyTheme(settings.theme);
      address.value = settings.homepage;
      await call("browser_navigate", { address: settings.homepage });
    }
  }

  void connectEvents().catch(showError);
})();
