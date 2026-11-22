/* Offline NovaBrowser UI localization.
 * English source messages are fallbacks; Vietnamese is the first additional
 * locale. Translation applies only to the browser chrome, never web content.
 */
(() => {
  "use strict";
  const VI = Object.freeze({
  "Settings": "Cài đặt",
  "New tab": "Tab mới",
  "Open tabs": "Các tab đang mở",
  "Minimize": "Thu nhỏ",
  "Maximize": "Phóng to",
  "Close": "Đóng",
  "Window controls": "Điều khiển cửa sổ",
  "Browser controls": "Điều khiển trình duyệt",
  "Back": "Quay lại",
  "Forward": "Tiến tới",
  "Reload": "Tải lại",
  "Go to homepage": "Về trang chủ",
  "Home": "Trang chủ",
  "Search or type a URL": "Tìm kiếm hoặc nhập địa chỉ web",
  "Bookmark this page": "Đánh dấu trang này",
  "Remove bookmark": "Bỏ dấu trang",
  "Remove {title}": "Xóa {title}",
  "Close {title}": "Đóng {title}",
  "Show bookmarks": "Hiện dấu trang",
  "Bookmarks": "Dấu trang",
  "Show browsing history": "Xem lịch sử duyệt web",
  "History": "Lịch sử",
  "Bookmarks bar": "Thanh dấu trang",
  "Web page": "Trang web",
  "Rendered web page": "Trang web đã hiển thị",
  "Browser library": "Thư viện trình duyệt",
  "Close library": "Đóng thư viện",
  "YOUR LIBRARY": "THƯ VIỆN CỦA BẠN",
  "Browsing history": "Lịch sử duyệt web",
  "Clear browsing history": "Xóa lịch sử duyệt web",
  "Clear history": "Xóa lịch sử",
  "Pages you visit will appear here": "Các trang đã truy cập sẽ xuất hiện ở đây",
  "Your saved pages will appear here": "Trang đã lưu sẽ xuất hiện ở đây",
  "Bookmarks you save will appear here": "Dấu trang đã lưu sẽ xuất hiện ở đây",
  "Browser settings": "Cài đặt trình duyệt",
  "Settings navigation": "Điều hướng cài đặt",
  "Search settings": "Tìm trong cài đặt",
  "Search settings sections": "Tìm nhóm cài đặt",
  "Settings sections": "Các mục cài đặt",
  "Close settings": "Đóng cài đặt",
  "General": "Chung",
  "Appearance": "Giao diện",
  "Search engine": "Công cụ tìm kiếm",
  "Tabs": "Tab",
  "Privacy & data": "Quyền riêng tư và dữ liệu",
  "About NovaBrowser": "Giới thiệu NovaBrowser",
  "Desktop preview": "Bản xem trước desktop",
  "Choose how NovaBrowser starts and where your homepage leads.": "Chọn cách NovaBrowser khởi động và trang chủ sẽ mở.",
  "On startup": "Khi khởi động",
  "Open on startup": "Mở khi khởi động",
  "Restore the last session or start at your homepage.": "Khôi phục phiên trước hoặc mở trang chủ.",
  "Continue where you left off": "Tiếp tục phiên trước",
  "Open homepage": "Mở trang chủ",
  "Homepage": "Trang chủ",
  "Used when opening the homepage and, optionally, new tabs.": "Dùng cho nút Trang chủ và tab mới nếu bạn chọn.",
  "Enter a website address or search term. Changing it does not close your current tabs.": "Nhập địa chỉ website hoặc từ khóa. Việc thay đổi không đóng các tab đang mở.",
  "When session restore is enabled, NovaBrowser reopens saved tabs. Inactive tabs load when selected.": "Khi bật khôi phục phiên, NovaBrowser mở lại các tab đã lưu. Tab nền chỉ tải khi bạn chọn.",
  "Adjust the look and layout of the browser controls.": "Tùy chỉnh màu sắc và bố cục các nút của trình duyệt.",
  "Theme": "Chủ đề",
  "System": "Theo hệ thống",
  "Light": "Sáng",
  "Dark": "Tối",
  "The selected theme applies to NovaBrowser's interface. Website colors remain controlled by the website.": "Chủ đề chỉ áp dụng cho giao diện NovaBrowser; màu trang web do chính website quyết định.",
  "Toolbar": "Thanh công cụ",
  "Show Home button": "Hiển thị nút Trang chủ",
  "Add a Home button beside Back, Forward and Reload.": "Thêm nút Trang chủ bên cạnh các nút điều hướng.",
  "Show bookmarks bar": "Hiện thanh dấu trang",
  "Display saved pages below the address bar.": "Hiện các trang đã lưu bên dưới thanh địa chỉ.",
  "Choose how words typed into the address bar are searched.": "Chọn công cụ tìm kiếm khi nhập từ khóa vào thanh địa chỉ.",
  "Default search engine": "Công cụ tìm kiếm mặc định",
  "Search used in the address bar": "Tìm kiếm từ thanh địa chỉ",
  "Queries are sent to your chosen search provider.": "Từ khóa được gửi đến công cụ tìm kiếm đã chọn.",
  "Example": "Ví dụ",
  "rust browser engine": "rust browser engine",
  "Google Search": "Tìm kiếm Google",
  "Bing Search": "Tìm kiếm Bing",
  "DuckDuckGo": "DuckDuckGo",
  "Choose what appears when opening a new tab.": "Chọn nội dung hiển thị khi mở tab mới.",
  "New tabs": "Tab mới",
  "When opening a new tab": "Khi mở tab mới",
  "Applies to the plus button and Ctrl+T.": "Áp dụng cho nút dấu cộng và Ctrl+T.",
  "Open a blank tab": "Mở tab trống",
  "Maximum open tabs": "Số tab tối đa",
  "Current engine preview limit.": "Giới hạn hiện tại của engine thử nghiệm.",
  "8 tabs": "8 tab",
  "Manage browsing records saved on this computer.": "Quản lý dữ liệu duyệt web được lưu trên máy tính.",
  "Save browsing history": "Lưu lịch sử duyệt web",
  "New page visits are added to your local history when enabled.": "Khi bật, các trang mới truy cập sẽ được ghi vào lịch sử trên máy.",
  "Delete visit records. Bookmarks and tabs are preserved.": "Xóa các lượt truy cập. Dấu trang và tab vẫn được giữ lại.",
  "Remove all bookmarks": "Xóa toàn bộ dấu trang",
  "Permanently clear saved bookmarks from this profile.": "Xóa vĩnh viễn toàn bộ dấu trang trong hồ sơ này.",
  "Clear bookmarks": "Xóa dấu trang",
  "Deleting history does not remove cookies or other site data. Cookie and permission controls are not yet available in this engine preview.": "Xóa lịch sử không xóa cookie hay dữ liệu website khác. Bản thử nghiệm chưa có quản lý cookie và quyền website.",
  "Version and browser engine information.": "Thông tin phiên bản và engine trình duyệt.",
  "Version 0.1.0 · Desktop preview": "Phiên bản 0.1.0 · Bản xem trước desktop",
  "Browser engine": "Engine trình duyệt",
  "Experimental Rust rendering and JavaScript engine": "Engine hiển thị và JavaScript thử nghiệm bằng Rust",
  "Desktop application": "Ứng dụng desktop",
  "Native window and browser interface": "Cửa sổ và giao diện trình duyệt native",
  "Icon system": "Bộ biểu tượng",
  "Self-hosted SVG icons": "Biểu tượng SVG lưu trong ứng dụng",
  "Keyboard shortcuts": "Phím tắt",
  "Address bar": "Thanh địa chỉ",
  "Bookmark current page": "Đánh dấu trang hiện tại",
  "This is an engineering preview and does not yet provide the isolation and security guarantees of production browsers.": "Đây là bản thử nghiệm kỹ thuật, chưa có cơ chế cô lập và bảo mật tương đương trình duyệt thương mại.",
  "Discard changes": "Bỏ thay đổi",
  "Save changes": "Lưu thay đổi",
  "Language": "Ngôn ngữ",
  "Language & region": "Ngôn ngữ và khu vực",
  "Choose the language used in NovaBrowser's menus and settings.": "Chọn ngôn ngữ cho menu và phần cài đặt của NovaBrowser.",
  "Display language": "Ngôn ngữ giao diện",
  "Follow system language": "Theo ngôn ngữ hệ thống",
  "English": "English",
  "Tiếng Việt": "Tiếng Việt",
  "Changes appear immediately. Save to keep your choice after restarting.": "Giao diện đổi ngay khi chọn. Nhấn Lưu để giữ lựa chọn sau khi khởi động lại.",
  "This changes the browser interface, not the language of websites. Automatic webpage translation is not supported yet.": "Tùy chọn này chỉ đổi ngôn ngữ giao diện, không dịch nội dung trang web. Tính năng dịch trang tự động chưa được hỗ trợ.",
  "Changes saved.": "Đã lưu thay đổi.",
  "Changes discarded.": "Đã bỏ thay đổi.",
  "1 matching section": "Có 1 mục phù hợp",
  "{count} matching sections": "Có {count} mục phù hợp",
  "No matching settings": "Không tìm thấy cài đặt phù hợp",
  "Enter a homepage URL or search query.": "Nhập địa chỉ trang chủ hoặc từ khóa tìm kiếm.",
  "Homepage is too long (maximum 4,096 bytes).": "Địa chỉ trang chủ quá dài (tối đa 4.096 byte).",
  "Unable to save settings: {error}": "Không thể lưu cài đặt: {error}",
  "Permanently clear all bookmarks from this browser?": "Xóa vĩnh viễn toàn bộ dấu trang của trình duyệt?",
  "Permanently clear all browsing history from this browser?": "Xóa vĩnh viễn toàn bộ lịch sử duyệt web?",
  "Bookmarks cleared.": "Đã xóa dấu trang.",
  "Browsing history cleared.": "Đã xóa lịch sử duyệt web.",
  "Unable to clear {item}: {error}": "Không thể xóa {item}: {error}",
  "bookmarks": "dấu trang",
  "browsing history": "lịch sử duyệt web",
  "Remove {name}": "Xóa {name}",
  "Close tab": "Đóng tab",
  "Untitled page": "Trang chưa có tiêu đề",
  "Page loaded": "Đã tải trang",
  "Starting browser engine": "Đang khởi động engine",
  "No bookmarks yet": "Chưa có dấu trang",
  "New tab page": "Trang tab mới"
});
  const SOURCES = new WeakMap();
  const ATTR_SOURCES = new WeakMap();
  const SUBSCRIBERS = new Set();
  const OMIT = "#tab-list, #library-list, #bookmarks-bar, #page-area, #status-bubble";
  let preference = "system";
  let active = "en";

  function systemLocale() {
    const values = Array.isArray(navigator.languages) && navigator.languages.length
      ? navigator.languages : [navigator.language || "en"];
    // Respect preference order; an unsupported primary locale falls back
    // to English rather than unexpectedly selecting a later Vietnamese entry.
    const primary = values[0] || "en";
    return /^vi(?:-|$)/i.test(primary) ? "vi" : "en";
  }
  function resolve(value) {
    return value === "vi" || value === "en" ? value : systemLocale();
  }
  function t(source, vars = {}) {
    const phrase = active === "vi" ? (VI[source] || source) : source;
    return phrase.replace(/\{(\w+)\}/g, (match, name) =>
      Object.prototype.hasOwnProperty.call(vars, name) ? String(vars[name]) : match
    );
  }
  function canTranslate(original) {
    return Object.prototype.hasOwnProperty.call(VI, original);
  }
  function apply(root = document.body) {
    if (!root) return;
    // Cache the original source per text node so a Vietnamese -> English
    // switch never depends on reverse translation.
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    let node;
    while ((node = walker.nextNode())) {
      const parent = node.parentElement;
      if (!parent || parent.closest(OMIT)) continue;
      const current = SOURCES.get(node);
      const original = current ?? node.nodeValue;
      const key = original.trim();
      if (!canTranslate(key)) continue;
      SOURCES.set(node, original);
      node.nodeValue = original.replace(key, t(key));
    }
    for (const element of root.querySelectorAll("[aria-label], [title], [placeholder], [alt]")) {
      if (element.closest(OMIT)) continue;
      let originals = ATTR_SOURCES.get(element);
      if (!originals) { originals = new Map(); ATTR_SOURCES.set(element, originals); }
      for (const attribute of ["aria-label", "title", "placeholder", "alt"]) {
        const original = originals.get(attribute) ?? element.getAttribute(attribute);
        if (!original || !canTranslate(original)) continue;
        originals.set(attribute, original);
        element.setAttribute(attribute, t(original));
      }
    }
    document.documentElement.lang = active;
  }
  function setLanguage(value) {
    preference = ["system", "en", "vi"].includes(value) ? value : "system";
    const effective = resolve(preference);
    const changed = effective !== active;
    active = effective;
    apply();
    if (changed) {
      for (const listener of SUBSCRIBERS) listener(active);
    }
    return active;
  }
  function onChange(listener) {
    SUBSCRIBERS.add(listener);
    return () => SUBSCRIBERS.delete(listener);
  }
  window.NovaI18n = Object.freeze({
    t, apply, setLanguage, onChange,
    current: () => active,
    preference: () => preference,
    locale: () => active === "vi" ? "vi-VN" : "en-US",
    available: Object.freeze(["system", "en", "vi"]),
  });
})();
