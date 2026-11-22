//! Windows-only native Servo child window hosted by the existing NovaBrowser.
//! This is a feature-gated integration. Tauri still owns the app and chrome;
//! Servo paints website pixels directly inside the same parent HWND.
//! Servo and its WebViews are non-Send and live only on the winit event thread.

use std::collections::HashMap;
use std::num::NonZeroIsize;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use servo::{
    InputEvent, Key, KeyState, KeyboardEvent, Modifiers, MouseButton, NavigationRequest,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, RenderingContext,
    Servo, ServoBuilder, WebView, WebViewBuilder, WheelDelta, WheelEvent,
    WheelMode, WindowRenderingContext,
};
use servo_embedder_traits::EventLoopWaker;
use tauri::{AppHandle, Manager};
use url::Url;
use webrender_api::units::DevicePoint;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton as WinitButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};
use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle,
};
use winit::window::Window;

use crate::Command;

#[derive(Debug)]
pub enum Control {
    Navigate { tab: String, url: Url },
    Select { tab: String },
    Close { tab: String },
    Back { tab: String },
    Forward { tab: String },
    Reload { tab: String },
    Layout { top: u32, width: u32, height: u32, visible: bool },
    Shutdown,
}

#[derive(Debug)]
enum Event {
    Wake,
    Control(Control),
}

#[derive(Clone)]
pub struct ServoBridge {
    proxy: EventLoopProxy<Event>,
}

impl ServoBridge {
    pub fn send(&self, control: Control) -> Result<(), String> {
        self.proxy.send_event(Event::Control(control))
            .map_err(|_| "Native Servo event loop stopped".into())
    }
}

/// The native parent HWND must outlive the child window. Shutdown is sent
/// from the same main-window CloseRequested handler as the legacy engine.
pub fn start(app: &AppHandle, sender: mpsc::Sender<Command>) -> Result<ServoBridge, String> {
    let window = app.get_webview_window("main")
        .ok_or("NovaBrowser window not found")?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let parent = hwnd.0 as isize;
    if parent == 0 { return Err("Invalid Win32 parent window".into()); }
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("nova-servo".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let mut builder = EventLoop::<Event>::with_user_event();
            builder.with_any_thread(true).with_dpi_aware(false);
            let loop_ = match builder.build() {
                Ok(loop_) => loop_,
                Err(error) => {
                    let _ = ready_tx.send(Err(format!("Servo event loop: {error}")));
                    return;
                }
            };
            let proxy = loop_.create_proxy();
            // Do not report the engine as ready before its native child window,
            // GL rendering context and Servo instance have initialized.
            let mut app = Embedder {
                parent, proxy, sender, state: None, ready: Some(ready_tx),
            };
            if let Err(error) = loop_.run_app(&mut app) {
                eprintln!("NovaBrowser Servo event loop: {error}");
            }
        })
        .map_err(|error| format!("Cannot start Servo: {error}"))?;
    ready_rx.recv_timeout(Duration::from_secs(5))
        .map_err(|_| "Servo event loop initialization timed out".to_string())?
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
    sender: mpsc::Sender<Command>,
}

impl servo::WebViewDelegate for Delegate {
    fn notify_new_frame_ready(&self, _view: WebView) {
        self.window.request_redraw();
    }
    fn notify_url_changed(&self, _view: WebView, url: Url) {
        let _ = self.sender.send(Command::ServoLocation(self.tab.clone(), url.to_string()));
    }
    fn notify_page_title_changed(&self, _view: WebView, title: Option<String>) {
        let _ = self.sender.send(Command::ServoTitle(
            self.tab.clone(), title.unwrap_or_default(),
        ));
    }
    fn notify_history_changed(&self, _view: WebView, entries: Vec<Url>, current: usize) {
        let _ = self.sender.send(Command::ServoHistory(
            self.tab.clone(), current > 0, current + 1 < entries.len(),
        ));
    }
    fn notify_crashed(&self, _view: WebView, reason: String, _: Option<String>) {
        let _ = self.sender.send(Command::ServoError(self.tab.clone(), reason));
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

    fn control(&mut self, control: Control, sender: &mpsc::Sender<Command>) -> bool {
        match control {
            Control::Navigate { tab, url } => {
                if !matches!(url.scheme(), "http" | "https") {
                    let _ = sender.send(Command::ServoError(tab, "Blocked URL scheme".into()));
                    return false;
                }
                if let Some(page) = self.pages.get(&tab) {
                    page.load(url);
                } else {
                    let delegate = Rc::new(Delegate {
                        window: self.window.clone(),
                        tab: tab.clone(),
                        sender: sender.clone(),
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
    sender: mpsc::Sender<Command>,
    state: Option<State>,
    ready: Option<mpsc::SyncSender<Result<ServoBridge, String>>>,
}

impl Embedder {
    fn fail_start(&mut self, error: String) {
        if let Some(tx) = self.ready.take() {
            let _ = tx.send(Err(error));
        }
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
        // SAFETY: this process owns the live Tauri parent HWND. The child
        // is dropped before the Tauri parent window is destroyed.
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
        if let Some(tx) = self.ready.take() {
            let _ = tx.send(Ok(ServoBridge { proxy: self.proxy.clone() }));
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Event) {
        let Some(state) = self.state.as_mut() else { return; };
        match event {
            Event::Wake => state.engine.spin_event_loop(),
            Event::Control(control) => {
                if state.control(control, &self.sender) {
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
                        let _ = self.sender.send(Command::ServoFocusAddress);
                        return;
                    }
                    if control && key_text == "t" {
                        let command = if state.modifiers.shift_key() {
                            Command::ReopenTab
                        } else {
                            Command::NewTab
                        };
                        let _ = self.sender.send(command);
                        return;
                    }
                    if control && key_text == "w" {
                        if let Some(tab) = state.active.as_ref() {
                            let _ = self.sender.send(Command::CloseTab(tab.clone()));
                        }
                        return;
                    }
                    if control && key_text == "r" {
                        let _ = self.sender.send(Command::Reload);
                        return;
                    }
                    if alt && matches!(event.logical_key, WinitKey::Named(NamedKey::ArrowLeft)) {
                        let _ = self.sender.send(Command::Back);
                        return;
                    }
                    if alt && matches!(event.logical_key, WinitKey::Named(NamedKey::ArrowRight)) {
                        let _ = self.sender.send(Command::Forward);
                        return;
                    }
                }
                let key: Key = event.logical_key.to_string().parse().unwrap_or_default();
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
