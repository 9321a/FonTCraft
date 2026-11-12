//! Starting with the computer, staying in the menu bar, and the shortcut that opens FontCraft.

use eframe::egui;
use std::path::PathBuf;
#[cfg_attr(target_os = "android", allow(unused_imports))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const AGENT_LABEL: &str = "app.fontcraft.login";

fn agent_path() -> PathBuf {
    crate::paths::home().join(format!("Library/LaunchAgents/{AGENT_LABEL}.plist"))
}

/// Adds (or removes) the login item that starts FontCraft in the background when the person logs in:
/// a LaunchAgent on macOS, the Run key in the registry on Windows, an autostart entry on Linux.
pub fn set_login_item(on: bool) {
    let Ok(exe) = std::env::current_exe() else { return };
    #[cfg(target_os = "windows")]
    {
        let key = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let _ = if on {
            crate::paths::command("reg")
                .args(["add", key, "/v", "FontCraft", "/t", "REG_SZ", "/f", "/d", &format!("\"{}\" --background", exe.display())])
                .status()
        } else {
            crate::paths::command("reg").args(["delete", key, "/v", "FontCraft", "/f"]).status()
        };
        return;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let path = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| crate::paths::home().join(".config"))
            .join("autostart/fontcraft.desktop");
        if on {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let entry = format!("[Desktop Entry]\nType=Application\nName=FontCraft\nExec=\"{}\" --background\nX-GNOME-Autostart-enabled=true\n", exe.display());
            let _ = std::fs::write(&path, entry);
        } else {
            let _ = std::fs::remove_file(&path);
        }
        return;
    }
    #[allow(unreachable_code)]
    {
        let path = agent_path();
        if !on {
            let _ = std::fs::remove_file(&path);
            return;
        }
        let esc = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n<dict>\n\
  <key>Label</key><string>{AGENT_LABEL}</string>\n\
  <key>ProgramArguments</key><array><string>{}</string><string>--background</string></array>\n\
  <key>RunAtLoad</key><true/>\n\
  <key>ProcessType</key><string>Interactive</string>\n\
</dict>\n</plist>\n",
            esc(&exe.display().to_string())
        );
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, plist);
    }
}

/// "Cmd+Shift+F" → "⌘⇧F".
pub fn pretty_hotkey(s: &str) -> String {
    s.split('+')
        .map(|p| match p {
            "Cmd" => "⌘".to_string(),
            "Shift" => "⇧".to_string(),
            "Alt" => "⌥".to_string(),
            "Ctrl" => "⌃".to_string(),
            other => other.to_string(),
        })
        .collect()
}

/// While recording: the first key pressed together with ⌘, ⌥ or ⌃, as "Cmd+Shift+F".
pub fn read_hotkey(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::Key { key, pressed: true, modifiers, .. } => {
                if !(modifiers.mac_cmd || modifiers.command || modifiers.alt || modifiers.ctrl) {
                    return None;
                }
                let name = key.name();
                let ok = (name.len() == 1 && name.chars().all(|c| c.is_ascii_alphanumeric()))
                    || (name.starts_with('F') && name[1..].parse::<u8>().is_ok())
                    || name == "Space";
                if !ok {
                    return None;
                }
                let mut parts = Vec::new();
                if modifiers.mac_cmd || modifiers.command {
                    parts.push("Cmd");
                }
                if modifiers.ctrl && !modifiers.mac_cmd {
                    parts.push("Ctrl");
                }
                if modifiers.alt {
                    parts.push("Alt");
                }
                if modifiers.shift {
                    parts.push("Shift");
                }
                Some(format!("{}+{}", parts.join("+"), name.to_uppercase()))
            }
            _ => None,
        })
    })
}

/// "Cmd+Shift+F" → the global shortcut library's own format.
#[cfg(not(target_os = "android"))]
fn to_hotkey(s: &str) -> Option<global_hotkey::hotkey::HotKey> {
    let mut parts: Vec<String> = Vec::new();
    for p in s.split('+') {
        parts.push(match p {
            "Cmd" => "super".into(),
            "Shift" => "shift".into(),
            "Alt" => "alt".into(),
            "Ctrl" => "control".into(),
            k if k.len() == 1 && k.chars().all(|c| c.is_ascii_alphabetic()) => format!("Key{k}"),
            k if k.len() == 1 && k.chars().all(|c| c.is_ascii_digit()) => format!("Digit{k}"),
            k => k.to_string(),
        });
    }
    parts.join("+").parse().ok()
}

/// Brings FontCraft back from the background (call on the main thread).
#[cfg_attr(target_os = "android", allow(dead_code))]
pub fn show_app() {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        app.unhide(None);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
    }
}

/// Hides FontCraft's window; it keeps running in the menu bar.
pub fn hide_app() {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        app.hide(None);
    }
}

/// A small asterisk (FontCraft's activation sign) as a black menu bar image.
#[cfg(not(target_os = "android"))]
fn tray_image() -> tray_icon::Icon {
    let n = 36usize;
    let mut rgba = vec![0u8; n * n * 4];
    let c = n as f32 / 2.0;
    let r = n as f32 * 0.40;
    let w = n as f32 * 0.085;
    let seg_dist = |px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32| {
        let (dx, dy) = (bx - ax, by - ay);
        let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
    };
    for y in 0..n {
        for x in 0..n {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let mut d = f32::MAX;
            for k in 0..3 {
                let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 3.0;
                let (ox, oy) = (a.cos() * r, a.sin() * r);
                d = d.min(seg_dist(px, py, c - ox, c - oy, c + ox, c + oy));
            }
            let alpha = (w - d + 0.5).clamp(0.0, 1.0);
            rgba[(y * n + x) * 4 + 3] = (alpha * 255.0) as u8;
        }
    }
    tray_icon::Icon::from_rgba(rgba, n as u32, n as u32).expect("menu bar image")
}

/// Android: no menu bar icon or global shortcut.
#[cfg(target_os = "android")]
#[derive(Default)]
pub struct Background {
    pub wake: Arc<AtomicBool>,
}

#[cfg(target_os = "android")]
impl Background {
    pub fn sync(&mut self, _ctx: &egui::Context, _keep_in_menu_bar: bool, _hotkey: &str) {}
}

#[cfg(not(target_os = "android"))]
#[derive(Default)]
pub struct Background {
    tray: Option<tray_icon::TrayIcon>,
    hotkeys: Option<global_hotkey::GlobalHotKeyManager>,
    registered: Option<(String, global_hotkey::hotkey::HotKey)>,
    handlers: bool,
    /// Set by the menu bar icon or the shortcut: open the window full screen.
    pub wake: Arc<AtomicBool>,
}

#[cfg(not(target_os = "android"))]
impl Background {
    /// Keeps the menu bar icon and the shortcut in line with the settings.
    pub fn sync(&mut self, ctx: &egui::Context, keep_in_menu_bar: bool, hotkey: &str) {
        if !self.handlers {
            self.handlers = true;
            let wake = self.wake.clone();
            let c = ctx.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |e: tray_icon::TrayIconEvent| {
                if let tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, button_state: tray_icon::MouseButtonState::Up, .. } = e {
                    show_app();
                    wake.store(true, Ordering::SeqCst);
                    c.request_repaint();
                }
            }));
            let wake = self.wake.clone();
            let c = ctx.clone();
            global_hotkey::GlobalHotKeyEvent::set_event_handler(Some(move |e: global_hotkey::GlobalHotKeyEvent| {
                if e.state == global_hotkey::HotKeyState::Pressed {
                    show_app();
                    wake.store(true, Ordering::SeqCst);
                    c.request_repaint();
                }
            }));
        }
        if keep_in_menu_bar && self.tray.is_none() {
            let builder = tray_icon::TrayIconBuilder::new();
            // macOS draws a template icon in the menu bar's own colour; elsewhere the icon as it is.
            #[cfg(target_os = "macos")]
            let builder = builder.with_icon_templated(tray_image());
            #[cfg(not(target_os = "macos"))]
            let builder = builder.with_icon(tray_image());
            self.tray = builder
                .with_tooltip("FontCraft")
                .with_menu_on_left_click(false)
                .build()
                .ok();
        } else if !keep_in_menu_bar && self.tray.is_some() {
            self.tray = None;
        }
        let wanted = hotkey.to_string();
        if self.registered.as_ref().map(|r| r.0.as_str()) != Some(wanted.as_str()) && !(wanted.is_empty() && self.registered.is_none()) {
            if self.hotkeys.is_none() {
                self.hotkeys = global_hotkey::GlobalHotKeyManager::new().ok();
            }
            if let Some(m) = &self.hotkeys {
                if let Some((_, old)) = self.registered.take() {
                    let _ = m.unregister(old);
                }
                if let Some(hk) = to_hotkey(&wanted) {
                    if m.register(hk).is_ok() {
                        self.registered = Some((wanted, hk));
                    }
                }
            }
        }
    }
}
