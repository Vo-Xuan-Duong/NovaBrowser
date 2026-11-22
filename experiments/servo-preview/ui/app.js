(() => {
  "use strict";
  const form = document.getElementById("search-form");
  const address = document.getElementById("address");
  const provider = document.getElementById("provider");
  const status = document.getElementById("status");
  const go = document.getElementById("go-button");
  const invoke = window.__TAURI__?.core?.invoke;
  let navigating = false;

  function report(message, error = false) {
    status.textContent = message;
    status.classList.toggle("error", error);
  }

  async function navigate(value) {
    const target = value.trim();
    if (!target || navigating) {
      if (!target) address.focus();
      return;
    }
    if (typeof invoke !== "function") {
      report("Không có kết nối với Servo Runtime. Kiểm tra bản build Tauri.", true);
      return;
    }
    navigating = true;
    go.disabled = true;
    report("Đang chuyển điều hướng cho Servo…");
    try {
      await invoke("servo_navigate", {
        address: target,
        searchEngine: provider.value,
      });
      // On success, Servo replaces this trusted Home page with the remote
      // document. Native Navigate-menu actions remain available.
    } catch (error) {
      report("Không thể điều hướng: " + String(error), true);
    } finally {
      navigating = false;
      go.disabled = false;
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    void navigate(address.value);
  });

  for (const shortcut of document.querySelectorAll("[data-target]")) {
    shortcut.addEventListener("click", () => {
      void navigate(shortcut.getAttribute("data-target") || "");
    });
  }
})();
