//! Fonts that other apps ask for (macOS), as RightFont answers them: FontCraft registers a CoreText
//! font-request source on the main run loop (-[RFFontActivator startFontRequestRunLoopSource],
//! 0x10004ee84: `CTFontManagerCreateFontRequestRunLoopSource(0, block)` then `CFRunLoopAddSource` on
//! `CFRunLoopGetMain`). When any app opens a document or picks a font that isn't installed, the system
//! asks FontCraft with the font's name (`NSFontNameAttribute`) and the asking process; a font of the
//! catalog (a library or Google Fonts) with that PostScript, full or family name is turned on at once
//! and handed back, so the app shows it without restarting. Names FontCraft doesn't have are kept for
//! the Missing Fonts window. Turned-on fonts stay on while the asking app runs (Auto Activation's
//! records) and go off when it quits.

// Only macOS has font requests; elsewhere the answers are never made.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// What happened with one request, for the app to pick up each frame.
pub enum Answer {
    /// Turned on for the app: (files, app's bundle id, app's name, font name).
    Activated(Vec<PathBuf>, String, String, String),
    /// Not in FontCraft's catalog: (font name, app's name).
    Missing(String, String),
}

struct State {
    /// Lower-case PostScript, full and family names → font files.
    index: HashMap<String, Vec<PathBuf>>,
    answers: Vec<Answer>,
    /// (app, name) pairs already handled: like RightFont (0x10004f89c keeps the names it handled
    /// per target app), each name is answered once per app.
    told: std::collections::HashSet<String>,
    /// Lower-case missing font name → replacement family.
    subs: HashMap<String, String>,
}

fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State { index: HashMap::new(), answers: Vec::new(), told: Default::default(), subs: HashMap::new() }))
}

/// Gives the request handler the catalog's names (after every scan).
pub fn set_index(index: HashMap<String, Vec<PathBuf>>) {
    if let Ok(mut s) = state().lock() {
        s.index = index;
    }
}

/// The requests answered since the last call.
pub fn take_answers() -> Vec<Answer> {
    state().lock().map(|mut s| std::mem::take(&mut s.answers)).unwrap_or_default()
}

/// The files for a font name, if FontCraft has it, or the replacement the person chose for it.
pub fn lookup(name: &str) -> Option<Vec<PathBuf>> {
    let s = state().lock().ok()?;
    let key = name.trim().to_lowercase();
    s.index.get(&key).cloned().or_else(|| s.subs.get(&key).and_then(|r| s.index.get(&r.to_lowercase())).cloned())
}

/// The system's own interface fonts, which apps ask for by name but are never missing: RightFont
/// skips requested names starting with these (-[RFFontActivator handleRequestFonts], 0x10004f89c,
/// six `hasPrefix:` checks against strings 0x10028c1de–0x10028c214). Names starting with "." are
/// CoreText's hidden system fonts (FontCraft's addition, for the same reason).
pub const SYSTEM_UI_PREFIXES: [&str; 6] = ["SFUIText", "SFUIDisplay", "SFCompact", "SFProText", "SFProDisplay", "HelveticaNeueUI"];

/// Whether a requested or listed font name is one of the system's own (never reported missing).
pub fn is_system_ui(name: &str) -> bool {
    let n = name.trim();
    n.starts_with('.') || SYSTEM_UI_PREFIXES.iter().any(|p| n.starts_with(p))
}

/// Missing font → replacement family (Missing Fonts' "Replace with…").
pub fn set_substitutes(subs: HashMap<String, String>) {
    if let Ok(mut s) = state().lock() {
        s.subs = subs.into_iter().map(|(k, v)| (k.to_lowercase(), v)).collect();
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use std::ffi::c_void;

    type Ptr = *const c_void;

    #[link(name = "CoreText", kind = "framework")]
    unsafe extern "C" {
        fn CTFontManagerCreateFontRequestRunLoopSource(order: isize, block: &block2::Block<dyn Fn(Ptr, i32) -> Ptr>) -> Ptr;
        fn CTFontManagerCreateFontDescriptorsFromURL(url: Ptr) -> Ptr;
        static kCTFontNameAttribute: Ptr;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRunLoopGetMain() -> Ptr;
        fn CFRunLoopAddSource(rl: Ptr, source: Ptr, mode: Ptr);
        static kCFRunLoopCommonModes: Ptr;
        fn CFDictionaryGetValue(dict: Ptr, key: Ptr) -> Ptr;
        fn CFArrayCreateMutable(alloc: Ptr, cap: isize, callbacks: Ptr) -> Ptr;
        fn CFArrayAppendArray(arr: Ptr, other: Ptr, range: CFRange);
        fn CFArrayGetCount(arr: Ptr) -> isize;
        fn CFRelease(p: Ptr);
        static kCFTypeArrayCallBacks: c_void;
    }

    #[repr(C)]
    struct CFRange {
        location: isize,
        length: isize,
    }

    fn cfstring_to_string(s: Ptr) -> Option<String> {
        if s.is_null() {
            return None;
        }
        let cf: core_foundation::string::CFString = unsafe { core_foundation::base::TCFType::wrap_under_get_rule(s as core_foundation::string::CFStringRef) };
        Some(cf.to_string())
    }

    fn app_of(pid: i32) -> (String, String) {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        unsafe {
            let app: *mut AnyObject = msg_send![class!(NSRunningApplication), runningApplicationWithProcessIdentifier: pid];
            if app.is_null() {
                return (String::new(), format!("#{pid}"));
            }
            let read = |sel: &str| -> String {
                let s: *mut AnyObject = if sel == "id" { msg_send![app, bundleIdentifier] } else { msg_send![app, localizedName] };
                if s.is_null() {
                    return String::new();
                }
                let c: *const std::os::raw::c_char = msg_send![s, UTF8String];
                if c.is_null() { String::new() } else { std::ffi::CStr::from_ptr(c).to_string_lossy().to_string() }
            };
            (read("id"), read("name"))
        }
    }

    /// One request: the font's name → its files turned on → their descriptors for the system.
    fn answer(attrs: Ptr, pid: i32) -> Ptr {
        let name = unsafe { cfstring_to_string(CFDictionaryGetValue(attrs, kCTFontNameAttribute)) }.unwrap_or_default();
        let out = unsafe { CFArrayCreateMutable(std::ptr::null(), 0, &kCFTypeArrayCallBacks as *const c_void) };
        if name.is_empty() || is_system_ui(&name) {
            return out;
        }
        let (bundle, app) = app_of(pid);
        match lookup(&name) {
            Some(files) => {
                let mut on = Vec::new();
                for f in &files {
                    if crate::activate::set(f, true) {
                        on.push(f.clone());
                    }
                    let url = core_foundation::url::CFURL::from_path(f, false);
                    if let Some(url) = url {
                        let descs = unsafe { CTFontManagerCreateFontDescriptorsFromURL(core_foundation::base::TCFType::as_concrete_TypeRef(&url) as Ptr) };
                        if !descs.is_null() {
                            unsafe {
                                let n = CFArrayGetCount(descs);
                                CFArrayAppendArray(out, descs, CFRange { location: 0, length: n });
                                CFRelease(descs);
                            }
                        }
                    }
                }
                if let Ok(mut s) = state().lock() {
                    s.answers.push(Answer::Activated(on, bundle, app, name));
                }
            }
            None => {
                if let Ok(mut s) = state().lock() {
                    if s.told.insert(format!("{bundle}\u{1f}{}", name.to_lowercase())) {
                        s.answers.push(Answer::Missing(name, app));
                    }
                }
            }
        }
        out
    }

    /// Registers the request source once (main thread).
    pub fn start() {
        static STARTED: OnceLock<()> = OnceLock::new();
        STARTED.get_or_init(|| {
            let block = block2::RcBlock::new(|attrs: Ptr, pid: i32| -> Ptr {
                match std::panic::catch_unwind(|| answer(attrs, pid)) {
                    Ok(p) => p,
                    Err(_) => std::ptr::null(),
                }
            });
            unsafe {
                let source = CTFontManagerCreateFontRequestRunLoopSource(0, &block);
                if !source.is_null() {
                    CFRunLoopAddSource(CFRunLoopGetMain(), source, kCFRunLoopCommonModes);
                }
            }
            // CoreText keeps a copy of the block; this one may go.
            drop(block);
        });
    }
}

/// Starts answering other apps' font requests (macOS; other systems have no such service).
pub fn start() {
    #[cfg(target_os = "macos")]
    mac::start();
}

#[cfg(test)]
mod tests {
    #[test]
    fn system_ui_fonts_are_never_missing() {
        assert!(super::is_system_ui("SFProText-Regular"));
        assert!(super::is_system_ui(".SFNS-Regular"));
        assert!(super::is_system_ui("HelveticaNeueUI-Bold"));
        assert!(!super::is_system_ui("Lobster-Regular"));
        assert!(!super::is_system_ui("Helvetica Neue"));
    }
}
