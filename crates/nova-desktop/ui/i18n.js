/* Offline NovaBrowser UI localization.
 * English source messages are fallbacks; Vietnamese is the first additional
 * locale. Translation applies only to the browser chrome, never web content.
 */
(() => {
  "use strict";
  const VI = Object.freeze({
  "{count} matching sections": "Có {count} mục phù hợp",
  "1 matching section": "Có 1 mục phù hợp",
  "8 tabs": "8 tab",
  "About NovaBrowser": "Giới thiệu NovaBrowser",
  "Add a Home button beside Back, Forward and Reload.": "Thêm nút Trang chủ bên cạnh các nút điều hướng.",
  "Address bar": "Thanh địa chỉ",
  "Adjust the look and layout of the browser controls.": "Tùy chỉnh màu sắc và bố cục các nút của trình duyệt.",
  "Appearance": "Giao diện",
  "Applies to the plus button and Ctrl+T.": "Áp dụng cho nút dấu cộng và Ctrl+T.",
  "Back": "Quay lại",
  "Bing Search": "Tìm kiếm Bing",
  "Bookmark current page": "Đánh dấu trang hiện tại",
  "Bookmark this page": "Đánh dấu trang này",
  "bookmarks": "dấu trang",
  "Bookmarks": "Dấu trang",
  "Bookmarks bar": "Thanh dấu trang",
  "Bookmarks cleared.": "Đã xóa dấu trang.",
  "Bookmarks you save will appear here": "Dấu trang đã lưu sẽ xuất hiện ở đây",
  "Browser controls": "Điều khiển trình duyệt",
  "Browser engine": "Engine trình duyệt",
  "Rendering process": "Tiến trình hiển thị",
  "Native website rendering status": "Trạng thái tiến trình hiển thị trang web",
  "Servo 0.8 (experimental)": "Servo 0.8 (thử nghiệm)",
  "Servo runs websites in a separate rendering process.": "Servo hiển thị website bằng tiến trình render riêng.",
  "Active": "Đang hoạt động",
  "Stopped": "Đã dừng",
  "Unknown": "Chưa xác định",
  "Browser library": "Thư viện trình duyệt",
  "Browser settings": "Cài đặt trình duyệt",
  "browsing history": "lịch sử duyệt web",
  "Browsing history": "Lịch sử duyệt web",
  "Browsing history cleared.": "Đã xóa lịch sử duyệt web.",
  "Changes appear immediately. Save to keep your choice after restarting.": "Giao diện đổi ngay khi chọn. Nhấn Lưu để giữ lựa chọn sau khi khởi động lại.",
  "Changes discarded.": "Đã bỏ thay đổi.",
  "Changes saved.": "Đã lưu thay đổi.",
  "Choose how NovaBrowser starts and where your homepage leads.": "Chọn cách NovaBrowser khởi động và trang chủ sẽ mở.",
  "Choose how words typed into the address bar are searched.": "Chọn công cụ tìm kiếm khi nhập từ khóa vào thanh địa chỉ.",
  "Choose the language used in NovaBrowser's menus and settings.": "Chọn ngôn ngữ cho menu và phần cài đặt của NovaBrowser.",
  "Choose what appears when opening a new tab.": "Chọn nội dung hiển thị khi mở tab mới.",
  "Clear bookmarks": "Xóa dấu trang",
  "Clear browsing history": "Xóa lịch sử duyệt web",
  "Clear history": "Xóa lịch sử",
  "Close": "Đóng",
  "Close {title}": "Đóng {title}",
  "Close library": "Đóng thư viện",
  "Close settings": "Đóng cài đặt",
  "Close tab": "Đóng tab",
  "Continue where you left off": "Tiếp tục phiên trước",
  "Current engine preview limit.": "Giới hạn hiện tại của engine thử nghiệm.",
  "Customize": "Tùy chỉnh",
  "Dark": "Tối",
  "Default search engine": "Công cụ tìm kiếm mặc định",
  "Delete visit records. Bookmarks and tabs are preserved.": "Xóa các lượt truy cập. Dấu trang và tab vẫn được giữ lại.",
  "Deleting history does not remove cookies or other site data. Cookie and permission controls are not yet available in this engine preview.": "Xóa lịch sử không xóa cookie hay dữ liệu website khác. Bản thử nghiệm chưa có quản lý cookie và quyền website.",
  "Desktop application": "Ứng dụng desktop",
  "Desktop preview": "Bản xem trước desktop",
  "Discard changes": "Bỏ thay đổi",
  "Display language": "Ngôn ngữ giao diện",
  "Display saved pages below the address bar.": "Hiện các trang đã lưu bên dưới thanh địa chỉ.",
  "DuckDuckGo": "DuckDuckGo",
  "English": "English",
  "Enter a homepage URL or search query.": "Nhập địa chỉ trang chủ hoặc từ khóa tìm kiếm.",
  "Enter a website address or search term. Changing it does not close your current tabs.": "Nhập địa chỉ website hoặc từ khóa. Việc thay đổi không đóng các tab đang mở.",
  "Example": "Ví dụ",
  "Experimental Rust rendering and JavaScript engine": "Engine hiển thị và JavaScript thử nghiệm bằng Rust",
  "Follow system language": "Theo ngôn ngữ hệ thống",
  "Forward": "Tiến tới",
  "General": "Chung",
  "Go to homepage": "Về trang chủ",
  "Good afternoon": "Chào buổi chiều",
  "Good evening": "Chào buổi tối",
  "Good morning": "Chào buổi sáng",
  "Google Search": "Tìm kiếm Google",
  "History": "Lịch sử",
  "Home": "Trang chủ",
  "Homepage": "Trang chủ",
  "Homepage is too long (maximum 4,096 bytes).": "Địa chỉ trang chủ quá dài (tối đa 4.096 byte).",
  "Icon system": "Bộ biểu tượng",
  "Keyboard shortcuts": "Phím tắt",
  "Language": "Ngôn ngữ",
  "Language & region": "Ngôn ngữ và khu vực",
  "Light": "Sáng",
  "Manage browsing records saved on this computer.": "Quản lý dữ liệu duyệt web được lưu trên máy tính.",
  "Maximize": "Phóng to",
  "Maximum open tabs": "Số tab tối đa",
  "Minimize": "Thu nhỏ",
  "Native window and browser interface": "Cửa sổ và giao diện trình duyệt native",
  "New page visits are added to your local history when enabled.": "Khi bật, các trang mới truy cập sẽ được ghi vào lịch sử trên máy.",
  "New tab": "Tab mới",
  "New tab page": "Trang tab mới",
  "New tabs": "Tab mới",
  "No bookmarks yet": "Chưa có dấu trang",
  "No matching settings": "Không tìm thấy cài đặt phù hợp",
  "NovaBrowser · New tab": "NovaBrowser · Tab mới",
  "NovaBrowser home": "Trang chủ NovaBrowser",
  "NovaBrowser Home": "Trang chủ NovaBrowser",
  "On startup": "Khi khởi động",
  "Open a blank tab": "Mở tab trống",
  "Open homepage": "Mở trang chủ",
  "Open on startup": "Mở khi khởi động",
  "Open tabs": "Các tab đang mở",
  "Page loaded": "Đã tải trang",
  "Pages you visit will appear here": "Các trang đã truy cập sẽ xuất hiện ở đây",
  "Pages you visit will appear here.": "Các trang bạn truy cập sẽ hiển thị tại đây.",
  "Permanently clear all bookmarks from this browser?": "Xóa vĩnh viễn toàn bộ dấu trang của trình duyệt?",
  "Permanently clear all browsing history from this browser?": "Xóa vĩnh viễn toàn bộ lịch sử duyệt web?",
  "Permanently clear saved bookmarks from this profile.": "Xóa vĩnh viễn toàn bộ dấu trang trong hồ sơ này.",
  "Privacy & data": "Quyền riêng tư và dữ liệu",
  "Queries are sent to your chosen search provider.": "Từ khóa được gửi đến công cụ tìm kiếm đã chọn.",
  "Quick links": "Truy cập nhanh",
  "Recently visited": "Đã truy cập gần đây",
  "Reload": "Tải lại",
  "Remove {name}": "Xóa {name}",
  "Remove {title}": "Xóa {title}",
  "Remove all bookmarks": "Xóa toàn bộ dấu trang",
  "Remove bookmark": "Bỏ dấu trang",
  "Rendered web page": "Trang web đã hiển thị",
  "Reopen closed tab": "Mở lại tab vừa đóng",
  "Restore the last session or start at your homepage.": "Khôi phục phiên trước hoặc mở trang chủ.",
  "rust browser engine": "rust browser engine",
  "Save browsing history": "Lưu lịch sử duyệt web",
  "Save changes": "Lưu thay đổi",
  "Search": "Tìm kiếm",
  "Search engine": "Công cụ tìm kiếm",
  "Search or type a URL": "Tìm kiếm hoặc nhập địa chỉ web",
  "Search settings": "Tìm trong cài đặt",
  "Search settings sections": "Tìm nhóm cài đặt",
  "Search the web or enter an address": "Tìm kiếm hoặc nhập địa chỉ website",
  "Search used in the address bar": "Tìm kiếm từ thanh địa chỉ",
  "Search with {provider}": "Tìm kiếm bằng {provider}",
  "Search with Google": "Tìm kiếm bằng Google",
  "Self-hosted SVG icons": "Biểu tượng SVG lưu trong ứng dụng",
  "Settings": "Cài đặt",
  "Settings navigation": "Điều hướng cài đặt",
  "Settings sections": "Các mục cài đặt",
  "Shift": "Shift",
  "Show bookmarks": "Hiện dấu trang",
  "Show bookmarks bar": "Hiện thanh dấu trang",
  "Show browsing history": "Xem lịch sử duyệt web",
  "Show Home button": "Hiển thị nút Trang chủ",
  "Some Home content could not be loaded.": "Không thể tải một phần nội dung trang chủ.",
  "Starting browser engine": "Đang khởi động engine",
  "Suggested sites": "Website gợi ý",
  "Switch tabs": "Chuyển tab",
  "System": "Theo hệ thống",
  "Tab": "Tab",
  "Tabs": "Tab",
  "The selected theme applies to NovaBrowser's interface. Website colors remain controlled by the website.": "Chủ đề chỉ áp dụng cho giao diện NovaBrowser; màu trang web do chính website quyết định.",
  "Theme": "Chủ đề",
  "This changes the browser interface, not the language of websites. Automatic webpage translation is not supported yet.": "Tùy chọn này chỉ đổi ngôn ngữ giao diện, không dịch nội dung trang web. Tính năng dịch trang tự động chưa được hỗ trợ.",
  "This is an engineering preview and does not yet provide the isolation and security guarantees of production browsers.": "Đây là bản thử nghiệm kỹ thuật, chưa có cơ chế cô lập và bảo mật tương đương trình duyệt thương mại.",
  "Tiếng Việt": "Tiếng Việt",
  "Toolbar": "Thanh công cụ",
  "Unable to clear {item}: {error}": "Không thể xóa {item}: {error}",
  "Unable to save settings: {error}": "Không thể lưu cài đặt: {error}",
  "Untitled page": "Trang chưa có tiêu đề",
  "Use nova://home for NovaBrowser Home, or enter a website address. This won't close your current tabs.": "Dùng nova://home để mở Trang chủ NovaBrowser hoặc nhập địa chỉ website. Các tab hiện tại không bị đóng.",
  "Used when opening the homepage and, optionally, new tabs.": "Dùng cho nút Trang chủ và tab mới nếu bạn chọn.",
  "Version 0.1.0 · Desktop preview": "Phiên bản 0.1.0 · Bản xem trước desktop",
  "Version and browser engine information.": "Thông tin phiên bản và engine trình duyệt.",
  "View bookmarks": "Xem dấu trang",
  "View history": "Xem lịch sử",
  "Web page": "Trang web",
  "Welcome back": "Chào mừng quay lại",
  "When opening a new tab": "Khi mở tab mới",
  "When session restore is enabled, NovaBrowser reopens saved tabs. Inactive tabs load when selected.": "Khi bật khôi phục phiên, NovaBrowser mở lại các tab đã lưu. Tab nền chỉ tải khi bạn chọn.",
  "Where would you like to go?": "Bạn muốn truy cập đâu?",
  "Window controls": "Điều khiển cửa sổ",
  "Your bookmarks": "Dấu trang của bạn",
  "YOUR LIBRARY": "THƯ VIỆN CỦA BẠN",
  "Your saved pages will appear here": "Trang đã lưu sẽ xuất hiện ở đây",
  "Google search needs verification": "Tìm kiếm Google yêu cầu xác minh",
  "Google returned an unusual-traffic page for this search. NovaBrowser cannot automatically complete Google's verification. You can view Google's response or choose another search provider.": "Google đã hiển thị trang cảnh báo lưu lượng bất thường cho tìm kiếm này. NovaBrowser không thể tự động thực hiện bước xác minh của Google. Bạn có thể xem phản hồi của Google hoặc chọn công cụ tìm kiếm khác.",
  "Your search": "Từ khóa tìm kiếm",
  "Search with Bing": "Tìm kiếm bằng Bing",
  "Search with DuckDuckGo": "Tìm kiếm bằng DuckDuckGo",
  "Retry Google": "Thử lại Google",
  "View Google's response": "Xem phản hồi của Google",
  "Your default search engine will not change. A query is sent to another provider only when you choose it.": "Công cụ tìm kiếm mặc định sẽ không thay đổi. Từ khóa chỉ được gửi sang nhà cung cấp khác khi bạn chủ động chọn.",
  "Google search unavailable": "Không thể tìm kiếm Google",
  "Google requested traffic verification": "Google yêu cầu xác minh lưu lượng"
});
  const SOURCES = new WeakMap();
  const ATTR_SOURCES = new WeakMap();
  const SUBSCRIBERS = new Set();
  const OMIT = "#tab-list, #library-list, #bookmarks-bar, #status-bubble, #home-shortcuts-list, #home-recent-list";
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
