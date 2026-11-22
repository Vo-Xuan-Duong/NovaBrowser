(() => {
  "use strict";

  const { invoke } = window.__TAURI__.core;
  const icons = window.NovaIcons;
  if (!icons) throw new Error("Lucide icon registry failed to load");
  icons.hydrate();
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
  const libraryTitleIcon = document.getElementById("library-title-icon");
  const historyFooter = document.getElementById("history-footer");
  const settingsTrigger = document.getElementById("show-settings");

  let lastSize = "";
  let libraryMode = "bookmarks";
  let activeTabId = null;
  let currentTitle = "New tab";
  let currentTabAddress = "";

  const settingsView = window.NovaSettings.create({
    invoke,
    onOpen() {
      closeLibrary();
      pageArea.hidden = true;
      bookmarkButton.disabled = true;
      settingsTrigger.classList.add("is-active");
      document.title = "Settings - NovaBrowser";
    },
    onRoute(section) {
      address.value = section === "general" ? "nova://settings" : `nova://settings/${section}`;
    },
    onClose() {
      pageArea.hidden = false;
      bookmarkButton.disabled = false;
      settingsTrigger.classList.remove("is-active");
      address.value = currentTabAddress;
      document.title = `${currentTitle} - NovaBrowser`;
      pageArea.focus();
      void refreshBookmarkState();
      void publishSize();
    },
  });

  function renderTabs(tabs) {
    tabList.replaceChildren();
    for (const tab of tabs) {
      const item = document.createElement("div");
      item.className = `tab${tab.active ? " is-active" : ""}`;
      item.setAttribute("role", "tab");
      item.setAttribute("aria-selected", String(tab.active));
      item.tabIndex = tab.active ? 0 : -1;
      item.addEventListener("click", (event) => {
        if (!event.target.closest(".tab-close")) {
          closeSettings();
          void call("browser_activate_tab", { tabId: tab.id });
        }
      });

      const mark = document.createElement("span");
      mark.className = "brand-mark";
      mark.setAttribute("aria-hidden", "true");
      mark.append(icons.create("globe"));
      const title = document.createElement("span");
      title.className = "tab-title";
      title.textContent = tab.title || "New tab";
      if (tab.active) title.id = "tab-title";
      const close = document.createElement("button");
      close.className = "tab-close";
      close.type = "button";
      close.append(icons.create("x"));
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
        currentTabAddress = tab.address || "";
        if (!settingsView.isOpen() && document.activeElement !== address) address.value = currentTabAddress;
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
    const target = address.value.trim();
    const internal = /^nova:\/\/settings(?:\/(general|appearance|search|tabs|privacy|about))?\/?$/i.exec(target);
    if (internal) {
      void openSettings((internal[1] || "general").toLowerCase());
      address.blur();
      return;
    }
    if (settingsView.isOpen()) closeSettings();
    address.value = target;
    void call("browser_navigate", { address: target });
    address.blur();
    pageArea.focus();
  });

  document.getElementById("back").addEventListener("click", () => {
    if (settingsView.isOpen()) closeSettings();
    else void call("browser_back");
  });
  document.getElementById("forward").addEventListener("click", () => {
    if (settingsView.isOpen()) closeSettings();
    else void call("browser_forward");
  });
  document.getElementById("reload").addEventListener("click", () => {
    if (!settingsView.isOpen()) void call("browser_reload");
  });

  function setBookmarkIcon(bookmarked) {
    bookmarkButton.classList.toggle("is-bookmarked", bookmarked);
    bookmarkButton.replaceChildren(icons.create(bookmarked ? "bookmark-check" : "bookmark"));
    const label = bookmarked ? "Remove bookmark" : "Bookmark this page";
    bookmarkButton.setAttribute("aria-label", label);
    bookmarkButton.title = label;
  }

  async function refreshBookmarkState() {
    if (settingsView.isOpen()) return;
    const url = address.value.trim();
    if (!/^https?:\/\//i.test(url)) {
      setBookmarkIcon(false);
      return;
    }
    const bookmarked = await call("browser_is_bookmarked", { url });
    // Discard stale requests when a tab was switched during the lookup.
    if (address.value.trim() !== url) return;
    setBookmarkIcon(bookmarked === true);
  }

  bookmarkButton.addEventListener("click", async () => {
    const url = address.value.trim();
    if (!/^https?:\/\//i.test(url)) return;
    const bookmarked = await call("browser_toggle_bookmark", {
      title: currentTitle || url,
      url,
    });
    if (typeof bookmarked === "boolean") {
      setBookmarkIcon(bookmarked);
      if (libraryMode === "bookmarks" && !libraryPanel.hidden) void renderLibrary("bookmarks");
    }
  });

  function closeLibrary() {
    libraryPanel.hidden = true;
  }

  async function renderLibrary(mode) {
    if (settingsView.isOpen()) closeSettings();
    libraryMode = mode;
    const isHistory = mode === "history";
    libraryTitle.textContent = isHistory ? "Browsing history" : "Bookmarks";
    libraryTitleIcon.replaceChildren(icons.create(isHistory ? "clock" : "folder-open"));
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
        remove.append(icons.create("trash"));
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

  function openSettings(section = "general") {
    return settingsView.open(section).catch(showError);
  }

  function closeSettings() {
    settingsView.close();
  }

  settingsTrigger.addEventListener("click", () => {
    if (settingsView.isOpen()) closeSettings();
    else void openSettings();
  });

  document.addEventListener("click", (event) => {
    if (!libraryPanel.hidden && !libraryPanel.contains(event.target)
        && !event.target.closest("#show-bookmarks, #show-history")) closeLibrary();
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

  document.getElementById("new-tab").addEventListener("click", () => {
    closeSettings();
    void call("browser_new_tab");
  });

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
      closeSettings();
      void call("browser_new_tab");
    } else if (event.ctrlKey && event.key.toLowerCase() === "w") {
      event.preventDefault();
      closeSettings();
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
      if (!settingsView.isOpen()) void call("browser_reload");
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
        if (payload.address) {
          currentTabAddress = payload.address;
          if (!settingsView.isOpen() && document.activeElement !== address) address.value = payload.address;
        }
        if (!settingsView.isOpen()) void refreshBookmarkState();
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
        if (!settingsView.isOpen()) document.title = `${currentTitle} - NovaBrowser`;
        if (libraryMode === "bookmarks" && !libraryPanel.hidden) void renderLibrary("bookmarks");
      }),
      listen("browser-frame", ({ payload }) => {
        // A newly activated lazy tab has no frame until its page loads.
        // Clear the previous tab's image rather than showing stale content.
        if (payload) {
          pageFrame.src = `data:image/png;base64,${payload}`;
          pageFrame.style.visibility = "visible";
        } else {
          pageFrame.removeAttribute("src");
          pageFrame.style.visibility = "hidden";
        }
      }),
    ]);
    await publishSize();
    const settings = await call("browser_get_settings");
    if (settings) settingsView.applySaved(settings);
    // Register listeners *before* the engine publishes restored tabs.
    // Startup is one-shot; the engine decides between restoring a session
    // and opening the configured homepage on the first launch.
    await call("browser_frontend_ready");
  }

  void connectEvents().catch(showError);
})();
