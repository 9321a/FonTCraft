//! Starting with the computer, staying in the menu bar, and the shortcut that opens FontCraft.

use eframe::egui;
use std::path::PathBuf;
#[cfg_attr(any(target_os = "android", target_os = "ios"), allow(unused_imports))]
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
#[cfg(not(any(target_os = "android", target_os = "ios")))]
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

static MAIN_VIEW: std::sync::atomic::AtomicPtr<std::ffi::c_void> = std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

pub fn set_main_view(view: *mut std::ffi::c_void) {
    MAIN_VIEW.store(view, Ordering::SeqCst);
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn main_view() -> *mut std::ffi::c_void {
    MAIN_VIEW.load(Ordering::SeqCst)
}

/// FontCraft works in the background now: its main window is put away and only the menu bar
/// asterisk is left (the asterisk, the shortcut or closing the window put it there).
static IN_BACKGROUND: AtomicBool = AtomicBool::new(false);
/// "Keep FontCraft in the menu bar" (Settings › General), as last seen by `Background::sync`: while
/// it is on, FontCraft in the background also leaves the Dock, as RightFont does.
static KEEP_IN_MENU_BAR: AtomicBool = AtomicBool::new(false);
/// Each trip to the background or back counts one; a trip still waiting for full screen to end
/// is called off when another one starts.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
static TRIP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// The trip waiting for macOS to finish taking the window out of full screen (0: none).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
static HIDE_WAITING: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// How FontCraft's window asks for another pass from Cocoa callbacks.
static REPAINT: std::sync::OnceLock<egui::Context> = std::sync::OnceLock::new();
/// A FontCraft dialog (Missing Fonts) is up over the other app while FontCraft works in the
/// background (`unhide_quietly`): FontCraft coming to the front then is for that dialog.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
static QUIET: AtomicBool = AtomicBool::new(false);
/// FontCraft just came back into the Dock: macOS made a new Dock tile for it, which shows the
/// bundle's icon, or the generic "exec" icon for a program started outside a .app (as Craft
/// Anything starts the build), so FontCraft's own icon is set again on the next pass.
static DOCK_ICON_DUE: AtomicBool = AtomicBool::new(false);

/// Whether FontCraft's Dock icon should be set again now (see `DOCK_ICON_DUE`).
pub fn take_dock_icon_due() -> bool {
    DOCK_ICON_DUE.swap(false, Ordering::SeqCst)
}

/// When the last trip to or from the menu bar happened (ms since FontCraft started).
static LAST_TRIP_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Since when (ms since FontCraft started) macOS has been moving a window into or out of full
/// screen; 0 when no move is under way.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
static FS_MOVING_SINCE_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// The self-test's log is on (FONTCRAFT_TEST_MENUBAR): each trip says how it ended on stderr.
static TEST_LOG: AtomicBool = AtomicBool::new(false);

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn stamp_trip() {
    LAST_TRIP_MS.store(now_ms(), Ordering::SeqCst);
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn test_note(what: &str) {
    if TEST_LOG.load(Ordering::Relaxed) {
        eprintln!("{:>6} ms {what}", now_ms());
    }
}

/// Whether FontCraft works in the background now (its window put away in the menu bar).
pub fn in_background() -> bool {
    IN_BACKGROUND.load(Ordering::SeqCst)
}

/// Marks FontCraft as working in the background (or not) without moving any window.
pub fn set_in_background(on: bool) {
    IN_BACKGROUND.store(on, Ordering::SeqCst);
}

/// "Keep FontCraft in the menu bar" as it is now (also kept up to date by `Background::sync`).
pub fn set_keep_in_menu_bar(on: bool) {
    KEEP_IN_MENU_BAR.store(on, Ordering::SeqCst);
}

/// A background start: the main window is made see-through before eframe shows it after its
/// first frame, so nothing flashes before it is put away (macOS).
pub fn see_through_until_put_away() {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        use objc2::msg_send;
        let run = || unsafe {
            let win = mac::main_window();
            if !win.is_null() {
                let _: () = msg_send![win, setAlphaValue: 0.0f64];
            }
        };
        let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
    }
}

/// Whether FontCraft's main window is on screen now (macOS; true elsewhere).
pub fn main_window_up() -> bool {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        use objc2::msg_send;
        let run = || unsafe {
            let win = mac::main_window();
            if win.is_null() {
                return false;
            }
            let visible: bool = msg_send![win, isVisible];
            visible
        };
        return objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or(false);
    }
    true
}

/// Lets Cocoa callbacks (the menu bar asterisk, the end of a full-screen exit) ask for a pass.
pub fn set_repaint_ctx(ctx: &egui::Context) {
    let _ = REPAINT.set(ctx.clone());
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn repaint() {
    if let Some(c) = REPAINT.get() {
        c.request_repaint();
    }
}

/// Runs work on the main thread from anywhere (macOS: Grand Central Dispatch, now or after a delay),
/// where AppKit's windows may be touched. The menu bar asterisk's handler hands its work over this
/// way, so it never runs inside the status item's own event handling.
#[cfg(target_os = "macos")]
mod gcd {
    use std::ffi::c_void;

    #[repr(C)]
    pub struct Queue {
        _private: [u8; 0],
    }

    unsafe extern "C" {
        static _dispatch_main_q: Queue;
        fn dispatch_async_f(queue: *const Queue, context: *mut c_void, work: extern "C" fn(*mut c_void));
        fn dispatch_after_f(when: u64, queue: *const Queue, context: *mut c_void, work: extern "C" fn(*mut c_void));
        fn dispatch_time(when: u64, delta: i64) -> u64;
    }

    type Work = Box<dyn FnOnce() + Send>;

    extern "C" fn run(context: *mut c_void) {
        let work: Box<Work> = unsafe { Box::from_raw(context as *mut Work) };
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || work()));
    }

    pub fn main_async(f: impl FnOnce() + Send + 'static) {
        let work: Box<Work> = Box::new(Box::new(f));
        unsafe { dispatch_async_f(&raw const _dispatch_main_q, Box::into_raw(work) as *mut c_void, run) }
    }

    pub fn main_after(seconds: f64, f: impl FnOnce() + Send + 'static) {
        let work: Box<Work> = Box::new(Box::new(f));
        unsafe {
            let when = dispatch_time(0, (seconds * 1e9) as i64);
            dispatch_after_f(when, &raw const _dispatch_main_q, Box::into_raw(work) as *mut c_void, run)
        }
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    /// NSApplicationActivationPolicy: in the Dock with its menus, or only in the menu bar.
    const POLICY_REGULAR: isize = 0;
    const POLICY_ACCESSORY: isize = 1;
    /// NSWindowStyleMaskFullScreen.
    const FULL_SCREEN: usize = 1 << 14;
    /// NSWindowOcclusionStateVisible.
    const OCCLUSION_VISIBLE: usize = 1 << 1;
    /// NSApplicationActivateAllWindows | NSApplicationActivateIgnoringOtherApps.
    const ACTIVATE_ALL_WINDOWS: usize = (1 << 0) | (1 << 1);

    fn ns_app() -> *mut AnyObject {
        unsafe { msg_send![objc2::class!(NSApplication), sharedApplication] }
    }

    fn class_name(obj: *mut AnyObject) -> String {
        if obj.is_null() {
            return String::new();
        }
        unsafe { (*obj).class().name().to_string_lossy().to_string() }
    }

    /// FontCraft's main window (the one holding the main view), else the first window the window
    /// library made.
    pub fn main_window() -> *mut AnyObject {
        unsafe {
            let view = main_view() as *mut AnyObject;
            if !view.is_null() {
                let win: *mut AnyObject = msg_send![view, window];
                if !win.is_null() {
                    return win;
                }
            }
            let windows: *mut AnyObject = msg_send![ns_app(), windows];
            let n: usize = if windows.is_null() { 0 } else { msg_send![windows, count] };
            for i in 0..n {
                let w: *mut AnyObject = msg_send![windows, objectAtIndex: i];
                if class_name(w).starts_with("Winit") {
                    return w;
                }
            }
            std::ptr::null_mut()
        }
    }

    pub fn is_fullscreen(win: *mut AnyObject) -> bool {
        if win.is_null() {
            return false;
        }
        let mask: usize = unsafe { msg_send![win, styleMask] };
        mask & FULL_SCREEN != 0
    }

    /// FontCraft is the active app and its main window is up (whether or not macOS is still
    /// animating it into full screen): the menu bar asterisk is green then.
    pub fn active_with_window() -> bool {
        if in_background() {
            return false;
        }
        unsafe {
            let app = ns_app();
            let active: bool = msg_send![app, isActive];
            let hidden: bool = msg_send![app, isHidden];
            if !active || hidden {
                return false;
            }
            let win = main_window();
            if win.is_null() {
                return true;
            }
            let visible: bool = msg_send![win, isVisible];
            let alpha: f64 = msg_send![win, alphaValue];
            visible && alpha > 0.05
        }
    }

    /// FontCraft is the app in front and its main window is up on this screen.
    pub fn up_front() -> bool {
        if in_background() {
            return false;
        }
        unsafe {
            let app = ns_app();
            let active: bool = msg_send![app, isActive];
            let hidden: bool = msg_send![app, isHidden];
            if !active || hidden {
                return false;
            }
            let win = main_window();
            if win.is_null() {
                return true;
            }
            let visible: bool = msg_send![win, isVisible];
            let mini: bool = msg_send![win, isMiniaturized];
            let alpha: f64 = msg_send![win, alphaValue];
            let occlusion: usize = msg_send![win, occlusionState];
            visible && !mini && alpha > 0.05 && occlusion & OCCLUSION_VISIBLE != 0
        }
    }

    /// The menu bar asterisk (or the shortcut): FontCraft in front goes to the background,
    /// otherwise it comes forward, full screen. RightFont's -[RFUIScenesRouter toggleMainWindow:]
    /// (0x1000c0614) toggles the same way.
    pub fn toggle() {
        if up_front() {
            to_background();
        } else {
            to_front();
        }
    }

    /// Out of the menu bar: back in the Dock (Regular, as RightFont at 0x1000c0690), the main window
    /// up, key and in front, FontCraft active, then full screen (`enter_fullscreen`).
    pub fn to_front() {
        let trip = TRIP.fetch_add(1, Ordering::SeqCst) + 1;
        HIDE_WAITING.store(0, Ordering::SeqCst);
        IN_BACKGROUND.store(false, Ordering::SeqCst);
        QUIET.store(false, Ordering::SeqCst);
        DOCK_ICON_DUE.store(true, Ordering::SeqCst);
        stamp_trip();
        test_note("to_front");
        let run = || unsafe {
            let app = ns_app();
            let _: bool = msg_send![app, setActivationPolicy: POLICY_REGULAR];
            let _: () = msg_send![app, unhide: std::ptr::null::<AnyObject>()];
            let win = main_window();
            if !win.is_null() {
                let mini: bool = msg_send![win, isMiniaturized];
                if mini {
                    let _: () = msg_send![win, deminiaturize: std::ptr::null::<AnyObject>()];
                }
                let _: () = msg_send![win, setAlphaValue: 1.0f64];
                let _: () = msg_send![win, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
                let _: () = msg_send![win, orderFrontRegardless];
            }
            let modern: bool = msg_send![app, respondsToSelector: objc2::sel!(activate)];
            if modern {
                let _: () = msg_send![app, activate];
            }
            let _: () = msg_send![app, activateIgnoringOtherApps: true];
        };
        let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
        gcd::main_after(crate::mechanics::FULLSCREEN_LOOK_S, move || enter_fullscreen(trip, 0, 0));
        repaint();
    }

    /// Full screen for the window just brought out of the menu bar (the person's wish: it opens
    /// "a pantalla completa"). macOS gives up on a full-screen move started while it is still
    /// bringing the app forward (FontCraft opened again from the Finder: willEnterFullScreen and
    /// no didEnterFullScreen, the window left small), so it is looked at again every
    /// FULLSCREEN_LOOK_S and asked once more if needed. The window library learns of each move
    /// from its window delegate (windowWillEnterFullScreen:), as when the green button is used.
    fn enter_fullscreen(trip: u64, looked: u32, asked: u32) {
        if TRIP.load(Ordering::SeqCst) != trip || in_background() {
            return;
        }
        let win = main_window();
        if win.is_null() || is_fullscreen(win) {
            return;
        }
        let since = FS_MOVING_SINCE_MS.load(Ordering::SeqCst);
        let moving = since != 0 && now_ms().saturating_sub(since) < crate::mechanics::FULLSCREEN_MOVE_STALE_MS;
        let mut asked = asked;
        if !moving && asked < crate::mechanics::FULLSCREEN_ASKS {
            let run = || unsafe {
                let visible: bool = msg_send![win, isVisible];
                if visible {
                    let _: () = msg_send![win, toggleFullScreen: std::ptr::null::<AnyObject>()];
                }
                visible
            };
            if objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or(false) {
                asked += 1;
                test_note(&format!("asking for full screen (time {asked})"));
            }
        }
        if looked < crate::mechanics::FULLSCREEN_LOOKS {
            gcd::main_after(crate::mechanics::FULLSCREEN_LOOK_S * 2.0, move || enter_fullscreen(trip, looked + 1, asked));
        }
    }

    /// Into the menu bar. A full-screen window first leaves full screen, see-through, and is put
    /// away when macOS says that is over (NSWindowDidExitFullScreenNotification): a full-screen
    /// window that is simply hidden is taken out of full screen by macOS and left on screen,
    /// smaller — the window "only shrank". At most LEAVE_FULLSCREEN_MAX_S for the animation, then
    /// it goes anyway.
    pub fn to_background() {
        let trip = TRIP.fetch_add(1, Ordering::SeqCst) + 1;
        IN_BACKGROUND.store(true, Ordering::SeqCst);
        QUIET.store(false, Ordering::SeqCst);
        stamp_trip();
        watch_app_notices();
        let win = main_window();
        if is_fullscreen(win) {
            test_note("to_background: leaving full screen first");
            HIDE_WAITING.store(trip, Ordering::SeqCst);
            let run = || unsafe {
                let _: () = msg_send![win, setAlphaValue: 0.0f64];
                let _: () = msg_send![win, toggleFullScreen: std::ptr::null::<AnyObject>()];
            };
            let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
            gcd::main_after(crate::mechanics::LEAVE_FULLSCREEN_MAX_S, move || {
                if HIDE_WAITING.load(Ordering::SeqCst) == trip {
                    test_note("full screen did not report its end in time: putting the window away anyway");
                    finish_background(trip);
                }
            });
        } else {
            test_note("to_background: window put away at once");
            finish_background(trip);
        }
    }

    /// Puts every FontCraft window away (never the menu bar's own), leaves the Dock while the
    /// menu bar asterisk is kept (Accessory, as RightFont's -[RFMainWindowController
    /// windowWillClose:] at 0x10003b030) and hands the front to the app that was there before.
    fn finish_background(trip: u64) {
        if TRIP.load(Ordering::SeqCst) != trip {
            return;
        }
        HIDE_WAITING.store(0, Ordering::SeqCst);
        let run = || unsafe {
            let app = ns_app();
            let windows: *mut AnyObject = msg_send![app, windows];
            let n: usize = if windows.is_null() { 0 } else { msg_send![windows, count] };
            for i in 0..n {
                let w: *mut AnyObject = msg_send![windows, objectAtIndex: i];
                if !class_name(w).starts_with("Winit") {
                    continue;
                }
                let visible: bool = msg_send![w, isVisible];
                if visible {
                    let _: () = msg_send![w, orderOut: std::ptr::null::<AnyObject>()];
                }
            }
            let win = main_window();
            if !win.is_null() {
                let _: () = msg_send![win, orderOut: std::ptr::null::<AnyObject>()];
                let _: () = msg_send![win, setAlphaValue: 1.0f64];
            }
            if KEEP_IN_MENU_BAR.load(Ordering::SeqCst) {
                let _: bool = msg_send![app, setActivationPolicy: POLICY_ACCESSORY];
            }
            let _: () = msg_send![app, hide: std::ptr::null::<AnyObject>()];
        };
        let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
        stamp_trip();
        test_note("in the background: window put away, out of the Dock");
        repaint();
    }

    /// Whether any FontCraft window (the main one or a dialog) is on screen.
    fn any_window_up() -> bool {
        unsafe {
            let windows: *mut AnyObject = msg_send![ns_app(), windows];
            let n: usize = if windows.is_null() { 0 } else { msg_send![windows, count] };
            (0..n).any(|i| {
                let w: *mut AnyObject = msg_send![windows, objectAtIndex: i];
                let visible: bool = msg_send![w, isVisible];
                visible && class_name(w).starts_with("Winit")
            })
        }
    }

    /// Listens (once) for two notices from macOS:
    /// - a window finished leaving full screen: the main window is put away then, when a trip to
    ///   the menu bar is waiting for it;
    /// - FontCraft became the active app while it works in the menu bar with nothing on screen
    ///   (opened again from the Finder, the Dock, Launchpad or Spotlight): its window comes up, as
    ///   RightFont's -[RFAppDelegate applicationShouldHandleReopen:hasVisibleWindows:] (0x1000db7fc)
    ///   shows its main window when none is visible. Not in the first moments after a trip (macOS
    ///   activates an app it has just started), nor for the Missing Fonts window.
    pub fn watch_app_notices() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let run = || unsafe {
                let center: *mut AnyObject = msg_send![objc2::class!(NSNotificationCenter), defaultCenter];
                let any: *const AnyObject = std::ptr::null();
                let queue: *const AnyObject = std::ptr::null();
                let name = objc2_foundation::NSString::from_str("NSWindowDidExitFullScreenNotification");
                let block = block2::RcBlock::new(|_note: *mut AnyObject| {
                    let trip = HIDE_WAITING.load(Ordering::SeqCst);
                    if trip != 0 {
                        // A moment after the animation's last frame, so macOS has settled the window.
                        gcd::main_after(0.05, move || {
                            if HIDE_WAITING.load(Ordering::SeqCst) == trip {
                                test_note("full screen ended");
                                finish_background(trip);
                            }
                        });
                    }
                });
                // The observers live as long as FontCraft.
                let observer: *mut AnyObject = msg_send![center, addObserverForName: &*name, object: any, queue: queue, usingBlock: &*block];
                let _: *mut AnyObject = msg_send![observer, retain];
                // Full-screen moves under way (macOS refuses a new one while one is animating).
                for (notice, moving) in [
                    ("NSWindowWillEnterFullScreenNotification", true),
                    ("NSWindowDidEnterFullScreenNotification", false),
                    ("NSWindowWillExitFullScreenNotification", true),
                    ("NSWindowDidExitFullScreenNotification", false),
                ] {
                    let name = objc2_foundation::NSString::from_str(notice);
                    let block = block2::RcBlock::new(move |_note: *mut AnyObject| {
                        FS_MOVING_SINCE_MS.store(if moving { now_ms().max(1) } else { 0 }, Ordering::SeqCst);
                        test_note(notice);
                    });
                    let observer: *mut AnyObject = msg_send![center, addObserverForName: &*name, object: any, queue: queue, usingBlock: &*block];
                    let _: *mut AnyObject = msg_send![observer, retain];
                }
                let name = objc2_foundation::NSString::from_str("NSApplicationDidBecomeActiveNotification");
                let block = block2::RcBlock::new(|_note: *mut AnyObject| {
                    let settled = now_ms().saturating_sub(LAST_TRIP_MS.load(Ordering::SeqCst)) > crate::mechanics::REOPEN_SETTLE_MS;
                    if in_background() && settled && HIDE_WAITING.load(Ordering::SeqCst) == 0 && !QUIET.load(Ordering::SeqCst) {
                        gcd::main_after(crate::mechanics::REOPEN_DELAY_S, || {
                            if in_background() && !QUIET.load(Ordering::SeqCst) && !any_window_up() {
                                test_note("opened again while in the menu bar: window up");
                                to_front();
                            }
                        });
                    }
                });
                let observer: *mut AnyObject = msg_send![center, addObserverForName: &*name, object: any, queue: queue, usingBlock: &*block];
                let _: *mut AnyObject = msg_send![observer, retain];
            };
            let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
        });
    }

    /// Brings another app (by bundle id) to the front at once: the window server is asked directly
    /// (NSRunningApplication), nothing waits for the app to answer, and no permission is needed.
    /// macOS 14 and later want the app in front to hand the front over first
    /// (yieldActivationToApplication:). Not running: it is opened, off the main thread.
    pub fn activate_app(bundle: &str) -> bool {
        let id = objc2_foundation::NSString::from_str(bundle);
        let run = || unsafe {
            let list: *mut AnyObject = msg_send![objc2::class!(NSRunningApplication), runningApplicationsWithBundleIdentifier: &*id];
            let n: usize = if list.is_null() { 0 } else { msg_send![list, count] };
            if n == 0 {
                return false;
            }
            let target: *mut AnyObject = msg_send![list, objectAtIndex: 0usize];
            let app = ns_app();
            let yields: bool = msg_send![app, respondsToSelector: objc2::sel!(yieldActivationToApplication:)];
            if yields {
                let _: () = msg_send![app, yieldActivationToApplication: target];
            }
            let ok: bool = msg_send![target, activateWithOptions: ACTIVATE_ALL_WINDOWS];
            ok
        };
        let running = objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or(false);
        if !running {
            let b = bundle.to_string();
            std::thread::spawn(move || {
                let _ = crate::paths::command("open").args(["-b", &b]).status();
            });
        }
        running
    }
}

/// Puts FontCraft's windows away; it keeps running in the menu bar (call on the main thread).
pub fn hide_app() {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        mac::to_background();
    }
}

/// The menu bar asterisk or the shortcut was used (main thread).
#[cfg(target_os = "macos")]
pub fn toggle_now() {
    if objc2_foundation::MainThreadMarker::new().is_some() {
        mac::toggle();
    }
}

/// Brings the app with this bundle id to the front at once, without waiting for it (macOS).
/// Whether it was running.
pub fn activate_app(bundle: &str) -> bool {
    #[cfg(target_os = "macos")]
    if !bundle.is_empty() && objc2_foundation::MainThreadMarker::new().is_some() {
        return mac::activate_app(bundle);
    }
    let _ = bundle;
    false
}

/// Self-test of the menu bar trips on this Mac (FONTCRAFT_TEST_MENUBAR=<log file>): the same calls
/// the asterisk makes, timed, with the window's state as macOS reports it written after each step.
#[cfg(target_os = "macos")]
pub fn menu_bar_selftest(log: std::path::PathBuf, probe_bundle: String) {
    use std::io::Write;
    fn state(step: &str) -> String {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        let run = || unsafe {
            let app: *mut AnyObject = msg_send![objc2::class!(NSApplication), sharedApplication];
            let active: bool = msg_send![app, isActive];
            let hidden: bool = msg_send![app, isHidden];
            let policy: isize = msg_send![app, activationPolicy];
            let win = mac::main_window();
            let (visible, full, alpha, occl, frame) = if win.is_null() {
                (false, false, 0.0, 0usize, String::new())
            } else {
                let v: bool = msg_send![win, isVisible];
                let a: f64 = msg_send![win, alphaValue];
                let o: usize = msg_send![win, occlusionState];
                let f: objc2_foundation::NSRect = msg_send![win, frame];
                (v, mac::is_fullscreen(win), a, o, format!("{:.0}x{:.0}@{:.0},{:.0}", f.size.width, f.size.height, f.origin.x, f.origin.y))
            };
            let ws: *mut AnyObject = msg_send![objc2::class!(NSWorkspace), sharedWorkspace];
            let front: *mut AnyObject = msg_send![ws, frontmostApplication];
            let bid: *mut AnyObject = if front.is_null() { std::ptr::null_mut() } else { msg_send![front, bundleIdentifier] };
            let mut front_id = if bid.is_null() { String::new() } else { (*(bid as *mut objc2_foundation::NSString)).to_string() };
            // Whose menus the menu bar shows.
            let owner: *mut AnyObject = msg_send![ws, menuBarOwningApplication];
            let oid: *mut AnyObject = if owner.is_null() { std::ptr::null_mut() } else { msg_send![owner, bundleIdentifier] };
            let owner_id = if oid.is_null() { String::from("-") } else { (*(oid as *mut objc2_foundation::NSString)).to_string() };
            front_id = format!("{front_id} menus_of={owner_id}");
            // Where the menu bar asterisk is (Cocoa screen points), for a driver that clicks it.
            let mut status = String::new();
            let windows: *mut AnyObject = msg_send![app, windows];
            let n: usize = if windows.is_null() { 0 } else { msg_send![windows, count] };
            for i in 0..n {
                let w: *mut AnyObject = msg_send![windows, objectAtIndex: i];
                if (*w).class().name().to_string_lossy().contains("StatusBar") {
                    let f: objc2_foundation::NSRect = msg_send![w, frame];
                    status = format!("{:.0},{:.0},{:.0},{:.0}", f.origin.x, f.origin.y, f.size.width, f.size.height);
                }
            }
            let screen: *mut AnyObject = msg_send![objc2::class!(NSScreen), mainScreen];
            let sh: f64 = if screen.is_null() {
                0.0
            } else {
                let f: objc2_foundation::NSRect = msg_send![screen, frame];
                f.size.height
            };
            format!(
                "{step}: active={active} hidden={hidden} policy={policy} window_visible={visible} fullscreen={full} alpha={alpha:.2} on_screen={} frame={frame} in_background={} front_app={front_id} statusitem={status} screen_h={sh:.0}",
                occl & 2 != 0,
                in_background()
            )
        };
        objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or_else(|_| format!("{step}: (state unreadable)"))
    }
    TEST_LOG.store(true, Ordering::Relaxed);
    let file = std::sync::Arc::new(std::sync::Mutex::new(std::fs::File::create(&log).ok()));
    let say = move |line: String| {
        if let Some(f) = file.lock().unwrap().as_mut() {
            let _ = writeln!(f, "{line}");
        }
    };
    let say = std::sync::Arc::new(say);
    // (seconds after start, what to do). FONTCRAFT_TEST_CLICKS=driver: the asterisk is clicked by
    // a driver posting real mouse clicks (as a person would); FontCraft only looks, every second,
    // and switches to the probe app at 14.5 s. Otherwise it toggles by itself.
    let driven = std::env::var("FONTCRAFT_TEST_CLICKS").as_deref() == Ok("driver");
    let steps: Vec<(f64, &'static str)> = if driven {
        let mut s: Vec<(f64, &'static str)> = (1..=26).map(|t| (t as f64, "look")).collect();
        s.push((14.5, "switch"));
        s.push((15.5, "look"));
        s.push((27.0, "quit"));
        s
    } else {
        vec![
            (2.0, "look"),
            (2.5, "click"),
            (5.5, "look"),
            (6.0, "click"),
            (8.5, "look"),
            (9.0, "click"),
            (12.0, "look"),
            (12.5, "switch"),
            (13.5, "look"),
            (14.0, "click"),
            (17.0, "look"),
            (17.5, "click"),
            (20.0, "look"),
            (20.5, "quit"),
        ]
    };
    for (at, what) in steps {
        let say = say.clone();
        let probe = probe_bundle.clone();
        gcd::main_after(at, move || match what {
            "look" => say(state(&format!("{at:>5.1}s look"))),
            "click" => {
                let t = std::time::Instant::now();
                mac::toggle();
                say(format!("{at:>5.1}s click (toggle took {} ms, now in_background={})", t.elapsed().as_millis(), in_background()));
            }
            "switch" => {
                let t = std::time::Instant::now();
                let ok = activate_app(&probe);
                say(format!("{at:>5.1}s switch to {probe}: running={ok}, call took {} ms", t.elapsed().as_millis()));
            }
            _ => std::process::exit(0),
        });
    }
}

/// FontCraft's activation asterisk (the same drawing as the font rows, `uiicons` "asterisk") at
/// menu bar size: 18 points (36 pixels for Retina screens), the height of the system's own menu
/// bar icons. `lit`: in the activation green; otherwise black, for macOS to draw as a template in
/// the menu bar's own colour (System Settings › Appearance).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn tray_image(lit: bool) -> tray_icon::Icon {
    let n = 36u32;
    let c = if lit { crate::libs::LIME } else { eframe::egui::Color32::BLACK };
    let rgba = crate::uiicons::rgba("asterisk", [c.r(), c.g(), c.b()], n).unwrap_or_else(|| vec![0; (n * n * 4) as usize]);
    tray_icon::Icon::from_rgba(rgba, n, n).expect("menu bar image")
}

/// Whether FontCraft's window is up and in front (not put away in the menu bar).
#[cfg_attr(any(target_os = "android", target_os = "ios"), allow(dead_code))]
pub fn app_in_front() -> bool {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        return objc2::exception::catch(std::panic::AssertUnwindSafe(mac::up_front)).unwrap_or(false);
    }
    false
}

/// Whether FontCraft is the active app with its main window up (the menu bar asterisk's green).
#[cfg_attr(any(target_os = "android", target_os = "ios"), allow(dead_code))]
pub fn window_active() -> bool {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        return objc2::exception::catch(std::panic::AssertUnwindSafe(mac::active_with_window)).unwrap_or(false);
    }
    app_in_front() && !in_background()
}

/// Whether FontCraft is hidden in the menu bar.
pub fn app_hidden() -> bool {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
        return in_background() || objc2_app_kit::NSApplication::sharedApplication(mtm).isHidden();
    }
    false
}

/// Shows FontCraft's windows again without bringing it to the front (so one dialog can be raised).
pub fn unhide_quietly() {
    #[cfg(target_os = "macos")]
    if let Some(mtm) = objc2_foundation::MainThreadMarker::new() {
        QUIET.store(true, Ordering::SeqCst);
        objc2_app_kit::NSApplication::sharedApplication(mtm).unhideWithoutActivation();
    }
}

/// Starts listening for macOS's notices about FontCraft's windows and activation (macOS; once,
/// on the main thread), and counts the start as the first trip (see `watch_app_notices`).
pub fn install() {
    #[cfg(target_os = "macos")]
    if objc2_foundation::MainThreadMarker::new().is_some() {
        stamp_trip();
        mac::watch_app_notices();
    }
}

/// Android and iOS: no desktop menu bar icon or global shortcut.
#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Default)]
pub struct Background {
    pub wake: Arc<AtomicBool>,
    pub sleep: Arc<AtomicBool>,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl Background {
    pub fn sync(&mut self, _ctx: &egui::Context, _keep_in_menu_bar: bool, _hotkey: &str) {}
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Default)]
pub struct Background {
    tray: Option<tray_icon::TrayIcon>,
    hotkeys: Option<global_hotkey::GlobalHotKeyManager>,
    registered: Option<(String, global_hotkey::hotkey::HotKey)>,
    handlers: bool,
    /// Windows and Linux: set by the menu bar icon or the shortcut to open the window full screen
    /// (macOS does it at once, in Cocoa, see `toggle_now`).
    pub wake: Arc<AtomicBool>,
    /// Windows and Linux: set while the window is in front, to put it away.
    pub sleep: Arc<AtomicBool>,
    /// Whether the menu bar asterisk is shown in green now.
    lit: Option<bool>,
}

/// What the menu bar asterisk and the shortcut do. macOS: on the main thread, at once, with
/// AppKit's own calls (the window library runs FontCraft's screen code only while the window is
/// up, so a trip to the menu bar that waited for it never finished). Elsewhere: through `wake` and
/// `sleep`, which the next pass reads.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn toggle_handler(wake: Arc<AtomicBool>, sleep: Arc<AtomicBool>, ctx: egui::Context) -> impl Fn() + Send + Sync + 'static {
    move || {
        #[cfg(target_os = "macos")]
        {
            let _ = (&wake, &sleep);
            // Still inside the click: macOS lets an app come to the front in answer to the
            // person's own click (since macOS 14 it may refuse it later, from a timer).
            if objc2_foundation::MainThreadMarker::new().is_some() {
                toggle_now();
                ctx.request_repaint();
            } else {
                let c = ctx.clone();
                gcd::main_async(move || {
                    toggle_now();
                    c.request_repaint();
                });
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            if !in_background() && app_in_front() {
                set_in_background(true);
                sleep.store(true, Ordering::SeqCst);
            } else {
                set_in_background(false);
                wake.store(true, Ordering::SeqCst);
            }
            ctx.request_repaint();
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl Background {
    /// Keeps the menu bar icon and the shortcut in line with the settings.
    pub fn sync(&mut self, ctx: &egui::Context, keep_in_menu_bar: bool, hotkey: &str) {
        KEEP_IN_MENU_BAR.store(keep_in_menu_bar, Ordering::SeqCst);
        if !self.handlers {
            self.handlers = true;
            set_repaint_ctx(ctx);
            // The menu bar asterisk: opens FontCraft full screen, or, while its window is in
            // front, sends it back to the menu bar.
            let toggle = toggle_handler(self.wake.clone(), self.sleep.clone(), ctx.clone());
            tray_icon::TrayIconEvent::set_event_handler(Some(move |e: tray_icon::TrayIconEvent| {
                if let tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, button_state: tray_icon::MouseButtonState::Up, .. } = e {
                    toggle();
                }
            }));
            let toggle = toggle_handler(self.wake.clone(), self.sleep.clone(), ctx.clone());
            global_hotkey::GlobalHotKeyEvent::set_event_handler(Some(move |e: global_hotkey::GlobalHotKeyEvent| {
                if e.state == global_hotkey::HotKeyState::Pressed {
                    toggle();
                }
            }));
        }
        if keep_in_menu_bar && self.tray.is_none() {
            let builder = tray_icon::TrayIconBuilder::new();
            // macOS draws a template icon in the menu bar's own colour; elsewhere the icon as it is.
            #[cfg(target_os = "macos")]
            let builder = builder.with_icon_templated(tray_image(false));
            #[cfg(not(target_os = "macos"))]
            let builder = builder.with_icon(tray_image(false));
            self.tray = builder
                .with_tooltip("FontCraft")
                .with_menu_on_left_click(false)
                .build()
                .ok();
            self.lit = Some(false);
        } else if !keep_in_menu_bar && self.tray.is_some() {
            self.tray = None;
            self.lit = None;
        }
        // Green while FontCraft's window is in front (the colour of a font turned on); the menu
        // bar's own colour while it works in the background.
        if let Some(tray) = &self.tray {
            let front = window_active();
            if self.lit != Some(front) {
                #[cfg(target_os = "macos")]
                let _ = if front { tray.set_icon(Some(tray_image(true))) } else { tray.set_icon_templated(Some(tray_image(false))) };
                #[cfg(not(target_os = "macos"))]
                let _ = tray.set_icon(Some(tray_image(front)));
                self.lit = Some(front);
                test_note(if front { "menu bar asterisk: green (window in front)" } else { "menu bar asterisk: menu bar colour" });
            }
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
