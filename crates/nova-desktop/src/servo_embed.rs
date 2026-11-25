//! IPC adapter for the Servo website rendering process.
//! Servo's SpiderMonkey/ICU cannot share the root Cargo.lock with Obscura/V8.
//! The renderer is compiled separately under engines/servo-native.
//! It creates a native child HWND; NovaBrowser remains the only application.

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command as ProcessCommand, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use url::Url;
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

struct Process {
    stdin: Mutex<ChildStdin>,
    child: Mutex<Child>,
    alive: Arc<AtomicBool>,
}
impl Drop for Process {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
        // Never leave an orphan with a window parented to NovaBrowser's HWND.
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Clone)]
pub struct ServoBridge {
    process: Arc<Process>,
}
impl ServoBridge {
    pub fn send(&self, command: Control) -> Result<(), String> {
        if !self.process.alive.load(Ordering::Acquire) {
            return Err("Servo rendering process is not running".into());
        }
        let msg = match command {
            Control::Navigate { tab, url } => json!({"op":"navigate","tab":tab,"url":url.to_string()}),
            Control::Select { tab } => json!({"op":"select","tab":tab}),
            Control::Close { tab } => json!({"op":"close","tab":tab}),
            Control::Back { tab } => json!({"op":"back","tab":tab}),
            Control::Forward { tab } => json!({"op":"forward","tab":tab}),
            Control::Reload { tab } => json!({"op":"reload","tab":tab}),
            Control::Layout { top, width, height, visible } => json!({
                "op":"layout","top":top,"width":width,"height":height,"visible":visible
            }),
            Control::Shutdown => json!({"op":"shutdown"}),
        };
        let mut stdin = self.process.stdin.lock()
            .map_err(|_| "Servo command pipe was poisoned".to_string())?;
        let mut bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        stdin.write_all(&bytes).and_then(|_| stdin.flush())
            .map_err(|e| format!("Servo command pipe unavailable: {e}"))
    }
}

fn binary_path() -> PathBuf {
    if let Some(path) = env::var_os("NOVA_SERVO_SIDECAR_BIN") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../engines/servo-native/target/release/nova-servo-native.exe")
}

fn dispatch_event(msg: &Value, browser: &mpsc::Sender<Command>) -> Result<bool, String> {
    let get = |field| msg.get(field).and_then(Value::as_str).unwrap_or("");
    let tab = get("tab").to_string();
    match get("event") {
        "ready" => return Ok(true),
        "fatal" => return Err(get("message").to_owned()),
        "location" if tab.len() <= 128 && get("url").len() <= 4096 =>
            { let _ = browser.send(Command::ServoLocation(tab, get("url").into())); },
        "title" if tab.len() <= 128 && get("title").len() <= 8192 =>
            { let _ = browser.send(Command::ServoTitle(tab, get("title").into())); },
        "history" if tab.len() <= 128 => {
            let back = msg.get("back").and_then(Value::as_bool).unwrap_or(false);
            let forward = msg.get("forward").and_then(Value::as_bool).unwrap_or(false);
            let _ = browser.send(Command::ServoHistory(tab, back, forward));
        }
        "error" if tab.len() <= 128 => {
            let message = get("message").chars().take(512).collect();
            let _ = browser.send(Command::ServoError(tab, message));
        }
        "focus_address" => { let _ = browser.send(Command::ServoFocusAddress); }
        "new_tab" => { let _ = browser.send(Command::NewTab); }
        "reopen_tab" => { let _ = browser.send(Command::ReopenTab); }
        "close_tab" if tab.len() <= 128 => { let _ = browser.send(Command::CloseTab(tab)); }
        "back" => { let _ = browser.send(Command::Back); }
        "forward" => { let _ = browser.send(Command::Forward); }
        "reload" => { let _ = browser.send(Command::Reload); }
        _ => {}
    }
    Ok(false)
}

pub fn start(app: &AppHandle, browser: mpsc::Sender<Command>) -> Result<ServoBridge, String> {
    let parent = app.get_webview_window("main")
        .ok_or("NovaBrowser window unavailable")?
        .hwnd().map_err(|e| e.to_string())?.0 as isize;
    if parent == 0 { return Err("Invalid NovaBrowser parent HWND".into()); }
    let binary = binary_path();
    if !binary.is_file() {
        return Err(format!("Missing Servo renderer: {}. Use run-nova-servo.cmd.",binary.display()));
    }
    let mut child = ProcessCommand::new(&binary)
        .arg("--parent-hwnd").arg(parent.to_string())
        .stdin(Stdio::piped()).stdout(Stdio::piped())
        .stderr(Stdio::inherit()).spawn()
        .map_err(|e| format!("Cannot start Servo renderer: {e}"))?;
    let stdin = child.stdin.take().ok_or("Missing Servo command pipe")?;
    let stdout = child.stdout.take().ok_or("Missing Servo event pipe")?;
    let alive = Arc::new(AtomicBool::new(true));
    let process = Arc::new(Process {
        stdin: Mutex::new(stdin), child: Mutex::new(child), alive: alive.clone()
    });
    let (tx, rx) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("nova-servo-events".into())
        .spawn(move || {
            let mut startup = Some(tx);
            let mut fatal = None;
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let Some(payload) = line.strip_prefix("NOVA\t") else { continue };
                if payload.len() > 65536 { continue; }
                let Ok(event) = serde_json::from_str::<Value>(payload) else { continue };
                match dispatch_event(&event, &browser) {
                    Ok(true) => { if let Some(tx) = startup.take() { let _ = tx.send(Ok(())); } }
                    Ok(false) => {}
                    Err(error) => {
                        fatal = Some(error);
                        break;
                    }
                }
            }
            alive.store(false, Ordering::Release);
            let reason = fatal.unwrap_or_else(|| "renderer process closed its event pipe".into());
            if let Some(tx) = startup.take() {
                let _ = tx.send(Err(format!("Servo renderer quit during startup: {reason}")));
            } else {
                // Surface unexpected sidecar termination in the existing
                // NovaBrowser status UI. Never silently strand a loading tab.
                let _ = browser.send(Command::ServoExited(reason));
            }
        }).map_err(|e| e.to_string())?;
    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok(())) => Ok(ServoBridge { process }),
        Ok(Err(error)) => Err(format!("Servo renderer startup failed: {error}")),
        Err(_) => Err("Servo renderer startup timed out; check stderr".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_tagged_native_renderer_events() {
        let (tx, rx) = mpsc::channel();
        assert!(dispatch_event(&json!({"event":"ready"}), &tx).unwrap());
        dispatch_event(&json!({
            "event":"location","tab":"nova-tab-1","url":"https://example.org/"
        }),&tx).unwrap();
        assert!(matches!(rx.try_recv(),Ok(Command::ServoLocation(tab,url))
            if tab == "nova-tab-1" && url == "https://example.org/"));
    }
}
