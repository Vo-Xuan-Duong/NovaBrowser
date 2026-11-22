//! Servo render-only process, attached to NovaBrowser's native HWND.
//! This intentionally uses an independent Cargo workspace and Cargo.lock:
//! Servo/SpiderMonkey and the legacy Obscura/V8 dependency trees cannot
//! resolve their incompatible ICU native bindings in a single lockfile.
//! The website engine owns no NovaBrowser chrome, profile or Settings.

use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::io::{self, BufRead, Write};
use serde::Deserialize;
use serde_json::json;

#[cfg(not(target_os = "windows"))]
compile_error!("The native Servo child-window helper currently supports Windows only.");

use servo::{
    InputEvent, Key, KeyState, KeyboardEvent, Modifiers, MouseButton, NavigationRequest,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, RenderingContext,
    Servo, ServoBuilder, WebView, WebViewBuilder, WheelDelta, WheelEvent,
    WheelMode, WindowRenderingContext,
};
use servo_embedder_traits::EventLoopWaker;
use url::Url;
use webrender_api::units::DevicePoint;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton as WinitButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle,
};
use winit::window::Window;

/// Only messages received from the trusted NovaBrowser parent pipe.
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Control {
    Navigate { tab: String, url: String },
    Select { tab: String },
    Close { tab: String },
    Back { tab: String },
    Forward { tab: String },
    Reload { tab: String },
    Layout { top: u32, width: u32, height: u32, visible: bool },
    Shutdown,
}

fn emit(event: serde_json::Value) {
    // Distinct prefix distinguishes protocol events from engine log messages.
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "NOVA\t{event}");
    let _ = stdout.flush();
}

fn main() {
    // Match the Servo upstream minimal embedder: choose the rustls provider
    // before initializing networking and the WebView engine.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let mut args = std::env::args().skip(1);
    let parent = match (args.next().as_deref(), args.next()) {
        (Some("--parent-hwnd"), Some(value)) => value.parse::<isize>().unwrap_or(0),
        _ => 0,
    };
    if parent == 0 {
        emit(json!({"event":"fatal","message":"Invalid or missing native parent HWND"}));
        std::process::exit(2);
    }
    let event_loop = match EventLoop::<Event>::with_user_event().build() {
        Ok(loop_) => loop_,
        Err(error) => {
            emit(json!({"event":"fatal","message":format!("winit event loop: {error}")}));
            std::process::exit(3);
        }
    };
    let proxy = event_loop.create_proxy();
    let input_proxy = proxy.clone();
    // Blocking stdin is read on a separate thread; all Servo handles stay on
    // the native event loop main thread.
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let Ok(line) = line else { break; };
            if line.len() > 65536 { continue; }
            match serde_json::from_str::<Control>(&line) {
                Ok(command) => {
                    if input_proxy.send_event(Event::Control(command)).is_err() {
                        break;
                    }
                }
                Err(error) => eprintln!("Invalid NovaBrowser renderer command: {error}"),
            }
        }
        let _ = input_proxy.send_event(Event::Control(Control::Shutdown));
    });
    let mut app = Embedder { parent, proxy, state: None };
    if let Err(error) = event_loop.run_app(&mut app) {
        emit(json!({"event":"fatal","message":format!("Servo event loop: {error}")}));
    }
}

#[derive(Debug)]
enum Event {
    Wake,
    Control(Control),
}

#[derive(Clone)]
struct Wake(EventLoopProxy<Event>);
impl EventLoopWaker for Wake {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {
        let _ = self.0.send_event(Event::Wake);
    }
}

struct Delegate {
    window: Rc<Window>,
    tab: String,
}

impl servo::WebViewDelegate for Delegate {
    fn notify_new_frame_ready(&self, _view: WebView) {
        self.window.request_redraw();
    }
    fn notify_url_changed(&self, _view: WebView, url: Url) {
        emit(json!({"event":"location","tab":self.tab,"url":url.to_string()}));
    }
    fn notify_page_title_changed(&self, _view: WebView, title: Option<String>) {
        emit(json!({"event":"title","tab":self.tab,"title":title.unwrap_or_default()}));
    }
    fn notify_history_changed(&self, _view: WebView, entries: Vec<Url>, current: usize) {
        emit(json!({"event":"history","tab":self.tab,
            "back":current > 0,"forward":current + 1 < entries.len()}));
    }
    fn notify_crashed(&self, _view: WebView, reason: String, _: Option<String>) {
        emit(json!({"event":"error","tab":self.tab,"message":reason}));
    }

    fn request_navigation(&self, _view: WebView, request: NavigationRequest) {
        // A remote document must not navigate its view to privileged local
        // schemes. In particular, file:// must never become a browser tab.
        if matches!(request.url.scheme(), "http" | "https" | "about") {
            request.allow();
        } else {
            request.deny();
        }
    }
}

struct State {
    window: Rc<Window>,
    context: Rc<WindowRenderingContext>,
    engine: Servo,
    pages: HashMap<String, WebView>,
    active: Option<String>,
    visible: bool,
    cursor: DevicePoint,
    modifiers: ModifiersState,
}

impl State {
    fn active_page(&self) -> Option<&WebView> {
        self.active.as_ref().and_then(|id| self.pages.get(id))
    }

    fn show_active(&self) {
        for (id, page) in &self.pages {
            if self.visible && self.active.as_ref() == Some(id) {
                page.show();
            } else {
                page.hide();
            }
        }
        self.window.set_visible(self.visible && self.active_page().is_some());
        if self.visible { self.window.request_redraw(); }
    }

    fn control(&mut self, control: Control) -> bool {
        match control {
            Control::Navigate { tab, url } => {
                let url = match Url::parse(&url) {
                    Ok(url) if matches!(url.scheme(), "http" | "https") => url,
                    _ => {
                        emit(json!({"event":"error","tab":tab,
                            "message":"Blocked or invalid website URL"}));
                        return false;
                    }
                };
                if let Some(page) = self.pages.get(&tab) {
                    page.load(url);
                } else {
                    let delegate = Rc::new(Delegate {
                        window: self.window.clone(),
                        tab: tab.clone(),
                     });
                    let page = WebViewBuilder::new(&self.engine, self.context.clone())
                        .url(url)
                        .hidpi_scale_factor(euclid::Scale::new(self.window.scale_factor() as f32))
                        .delegate(delegate)
                        .build();
                    page.resize(self.window.inner_size());
                    self.pages.insert(tab.clone(), page);
                }
                self.active = Some(tab);
                self.show_active();
            }
            Control::Select { tab } => {
                self.active = Some(tab);
                self.show_active();
            }
            Control::Close { tab } => {
                self.pages.remove(&tab);
                if self.active.as_ref() == Some(&tab) {
                    self.active = None;
                }
                self.show_active();
            }
            Control::Back { tab } => {
                if let Some(page) = self.pages.get(&tab) { page.go_back(); }
            }
            Control::Forward { tab } => {
                if let Some(page) = self.pages.get(&tab) { page.go_forward(); }
            }
            Control::Reload { tab } => {
                if let Some(page) = self.pages.get(&tab) { page.reload(); }
            }
            Control::Layout { top, width, height, visible } => {
                self.window.set_outer_position(PhysicalPosition::new(0, top as i32));
                let size = PhysicalSize::new(width.max(1), height.max(1));
                let _ = self.window.request_inner_size(size);
                for page in self.pages.values() { page.resize(size); }
                self.visible = visible;
                self.show_active();
            }
            Control::Shutdown => return true,
        }
        self.engine.spin_event_loop();
        false
    }

    fn input(&self, event: InputEvent) {
        if let Some(page) = self.active_page() {
            page.notify_input_event(event);
        }
    }
}

struct Embedder {
    parent: isize,
    proxy: EventLoopProxy<Event>,
    state: Option<State>,
}

impl Embedder {
    fn fail_start(&mut self, error: String) {
        emit(json!({"event":"fatal","message":error}));
    }
}

impl ApplicationHandler<Event> for Embedder {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() { return; }
        let Some(hwnd) = NonZeroIsize::new(self.parent) else {
            self.fail_start("Missing native NovaBrowser parent HWND".into());
            event_loop.exit();
            return;
        };
        let parent = RawWindowHandle::Win32(Win32WindowHandle::new(hwnd));
        let attrs = Window::default_attributes()
            .with_title("NovaBrowser page")
            .with_decorations(false)
            .with_visible(false)
            .with_position(PhysicalPosition::new(0, 120))
            .with_inner_size(PhysicalSize::new(1100, 650));
        // SAFETY: the trusted NovaBrowser parent process provides a live
        // HWND and keeps it alive until this helper receives Shutdown.
        // Windows permits cross-process native child windows, subject to
        // DPI-awareness restrictions verified during local integration.
        let attrs = unsafe { attrs.with_parent_window(Some(parent)) };
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Rc::new(window),
            Err(error) => {
                self.fail_start(format!("Servo child HWND creation failed: {error}"));
                event_loop.exit();
                return;
            }
        };
        let Ok(display) = event_loop.display_handle() else {
            self.fail_start("Servo display handle is unavailable".into());
            event_loop.exit();
            return;
        };
        let Ok(handle) = window.window_handle() else {
            self.fail_start("Servo child window handle is unavailable".into());
            event_loop.exit();
            return;
        };
        let context = match WindowRenderingContext::new(display, handle, window.inner_size()) {
            Ok(context) => Rc::new(context),
            Err(error) => {
                self.fail_start(format!("Servo graphics context failed: {error:?}"));
                event_loop.exit();
                return;
            }
        };
        let _ = context.make_current();
        let engine = ServoBuilder::default()
            .event_loop_waker(Box::new(Wake(self.proxy.clone())))
            .build();
        self.state = Some(State {
            window, context, engine,
            pages: HashMap::new(), active: None, visible: false,
            cursor: DevicePoint::new(0.0, 0.0),
            modifiers: ModifiersState::empty(),
        });
        emit(json!({"event":"ready"}));
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        let Some(state) = self.state.as_mut() else { return; };
        match event {
            Event::Wake => state.engine.spin_event_loop(),
            Event::Control(control) => {
                if state.control(control) {
                    state.pages.clear();
                    self.state = None;
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self, _event_loop: &ActiveEventLoop,
        _window: winit::window::WindowId, event: WindowEvent,
    ) {
        let Some(state) = self.state.as_mut() else { return; };
        match event {
            WindowEvent::CloseRequested => state.window.set_visible(false),
            WindowEvent::Resized(size) => {
                for page in state.pages.values() { page.resize(size); }
            }
            WindowEvent::RedrawRequested => {
                if state.visible {
                    if let Some(page) = state.active_page() {
                        page.paint();
                        state.context.present();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.cursor = DevicePoint::new(position.x as f32, position.y as f32);
                state.input(InputEvent::MouseMove(MouseMoveEvent::new(state.cursor.into())));
            }
            WindowEvent::MouseInput { state: pressed, button, .. } => {
                let button = match button {
                    WinitButton::Left => MouseButton::Primary,
                    WinitButton::Right => MouseButton::Secondary,
                    WinitButton::Middle => MouseButton::Auxiliary,
                    WinitButton::Back => MouseButton::Back,
                    WinitButton::Forward => MouseButton::Forward,
                    WinitButton::Other(n) => MouseButton::Other(n),
                };
                let action = if pressed == ElementState::Pressed {
                    MouseButtonAction::Down
                } else { MouseButtonAction::Up };
                state.input(InputEvent::MouseButton(MouseButtonEvent::new(
                    action, button, state.cursor.into(),
                )));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y, mode) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        ((x * 76.0) as f64, (y * 76.0) as f64, WheelMode::DeltaLine)
                    }
                    MouseScrollDelta::PixelDelta(p) => (p.x, p.y, WheelMode::DeltaPixel),
                };
                state.input(InputEvent::Wheel(WheelEvent::new(
                    WheelDelta { x, y, z: 0.0, mode },
                    state.cursor.into(), Modifiers::default(),
                )));
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let control = state.modifiers.control_key();
                    let alt = state.modifiers.alt_key();
                    let key_text = event.logical_key.to_text().unwrap_or("").to_lowercase();
                    if control && key_text == "l" {
                        emit(json!({"event":"focus_address"}));
                        return;
                    }
                    if control && key_text == "t" {
                        let event = if state.modifiers.shift_key() {
                            "reopen_tab"
                        } else {
                            "new_tab"
                        };
                        emit(json!({"event":event}));
                        return;
                    }
                    if control && key_text == "w" {
                        if let Some(tab) = state.active.as_ref() {
                            emit(json!({"event":"close_tab","tab":tab}));
                        }
                        return;
                    }
                    if control && key_text == "r" {
                        emit(json!({"event":"reload"}));
                        return;
                    }
                    if alt && matches!(event.logical_key.as_ref(), WinitKey::Named(NamedKey::ArrowLeft)) {
                        emit(json!({"event":"back"}));
                        return;
                    }
                    if alt && matches!(event.logical_key.as_ref(), WinitKey::Named(NamedKey::ArrowRight)) {
                        emit(json!({"event":"forward"}));
                        return;
                    }
                }
                // Winit's Key type is not Display; keep the original text
                // and named-key variant instead of calling to_string() on it.
                let key: Key = match event.logical_key.as_ref() {
                    WinitKey::Character(text) => text.parse().unwrap_or_default(),
                    WinitKey::Named(named) => format!("{named:?}").parse().unwrap_or_default(),
                    _ => Key::default(),
                };
                let pressed = if event.state == ElementState::Pressed {
                    KeyState::Down
                } else { KeyState::Up };
                state.input(InputEvent::Keyboard(KeyboardEvent::from_state_and_key(pressed, key)));
            }
            WindowEvent::Focused(focused) => {
                if let Some(page) = state.active_page() { page.set_focused(focused); }
            }
            _ => {}
        }
        state.engine.spin_event_loop();
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    #[test]
    fn decodes_browser_commands_without_running_a_window() {
        let command = serde_json::from_str::<Control>(
            r#"{"op":"navigate","tab":"nova-tab-3","url":"https://example.org/"}"#
        ).unwrap();
        assert!(matches!(command, Control::Navigate { tab, url }
            if tab == "nova-tab-3" && url == "https://example.org/"));
        let stop = serde_json::from_str::<Control>(r#"{"op":"shutdown"}"#).unwrap();
        assert!(matches!(stop, Control::Shutdown));
        assert!(serde_json::from_str::<Control>(r#"{"op":"run_code"}"#).is_err());
    }
}
