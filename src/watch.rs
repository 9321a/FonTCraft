//! Watching the libraries' and folders' files, so fonts added, removed or renamed outside FontCraft
//! (Finder, another app, a cloud folder) show up without restarting — like RightFont's RFFileWatcher
//! (`-[RFFileWatcher start]`, 0x10000c420): `FSEventStreamCreate(…, kFSEventStreamEventIdSinceNow,
//! latency 0.25 s, flags 0x1F)` = CF types | no defer | watch root | ignore self | file events.
//! "Ignore self" means FontCraft's own copies (Add…, new libraries) don't trigger it; those rescan anyway.
//! Other systems: a light check every 3 s of the folders' modification times (two levels deep).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// RightFont's FSEvents latency (0x10000c420: `fmov d0, #0.25`).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const LATENCY_S: f64 = 0.25;
/// How long after the last change the fonts are read again: RightFont's debounce timer
/// (`-[RFFileWatcher scheduleResolve]` 0x10000c794: `dispatch_time(0, 0x11e1a300)` = 300 ms,
/// one-shot, 50 ms leeway `0x2faf080`), restarted by every new event.
pub const DEBOUNCE_S: f64 = 0.3;
/// RightFont's stream flags (0x10000c420: `0x1f`).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const FLAGS: u32 = 0x1f;

pub struct Watcher {
    changed: Arc<AtomicBool>,
    #[cfg(target_os = "macos")]
    stream: mac::Stream,
    #[cfg(not(target_os = "macos"))]
    stop: Arc<AtomicBool>,
}

impl Watcher {
    /// Starts watching these folders.
    pub fn new(paths: &[PathBuf]) -> Option<Watcher> {
        let changed = Arc::new(AtomicBool::new(false));
        #[cfg(target_os = "macos")]
        {
            let stream = mac::Stream::start(paths, changed.clone())?;
            Some(Watcher { changed, stream })
        }
        #[cfg(not(target_os = "macos"))]
        {
            let stop = Arc::new(AtomicBool::new(false));
            let (c, s, paths) = (changed.clone(), stop.clone(), paths.to_vec());
            std::thread::spawn(move || {
                let mut last = signature(&paths);
                while !s.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let now = signature(&paths);
                    if now != last {
                        last = now;
                        c.store(true, Ordering::SeqCst);
                    }
                }
            });
            Some(Watcher { changed, stop })
        }
    }

    /// Whether something changed since the last call.
    pub fn take_changed(&self) -> bool {
        self.changed.swap(false, Ordering::SeqCst)
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        self.stream.stop();
        #[cfg(not(target_os = "macos"))]
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Modification times of the folders and their subfolders (adding or removing a file changes its folder's).
#[cfg(not(target_os = "macos"))]
fn signature(paths: &[PathBuf]) -> Vec<(PathBuf, Option<std::time::SystemTime>)> {
    let mut out = Vec::new();
    for p in paths {
        out.push((p.clone(), std::fs::metadata(p).and_then(|m| m.modified()).ok()));
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let q = e.path();
                if q.is_dir() {
                    out.push((q.clone(), std::fs::metadata(&q).and_then(|m| m.modified()).ok()));
                }
            }
        }
    }
    out.sort();
    out
}

#[cfg(target_os = "macos")]
mod mac {
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::string::CFString;
    use std::ffi::c_void;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    type StreamRef = *mut c_void;

    #[repr(C)]
    struct Context {
        version: isize,
        info: *mut c_void,
        retain: *const c_void,
        release: *const c_void,
        copy_description: *const c_void,
    }

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn FSEventStreamCreate(
            allocator: *const c_void,
            callback: extern "C" fn(StreamRef, *mut c_void, usize, *mut c_void, *const u32, *const u64),
            context: *const Context,
            paths: core_foundation::array::CFArrayRef,
            since: u64,
            latency: f64,
            flags: u32,
        ) -> StreamRef;
        fn FSEventStreamSetDispatchQueue(stream: StreamRef, queue: *mut c_void);
        fn FSEventStreamStart(stream: StreamRef) -> u8;
        fn FSEventStreamStop(stream: StreamRef);
        fn FSEventStreamInvalidate(stream: StreamRef);
        fn FSEventStreamRelease(stream: StreamRef);
    }
    extern "C" {
        fn dispatch_queue_create(label: *const std::os::raw::c_char, attr: *const c_void) -> *mut c_void;
    }

    const SINCE_NOW: u64 = 0xFFFF_FFFF_FFFF_FFFF;

    extern "C" fn on_event(_s: StreamRef, info: *mut c_void, n: usize, _paths: *mut c_void, _flags: *const u32, _ids: *const u64) {
        if n > 0 && !info.is_null() {
            unsafe { (*(info as *const AtomicBool)).store(true, Ordering::SeqCst) };
        }
    }

    pub struct Stream {
        stream: StreamRef,
        flag: *const AtomicBool,
    }

    impl Stream {
        pub fn start(paths: &[PathBuf], changed: Arc<AtomicBool>) -> Option<Stream> {
            let list: Vec<CFString> = paths.iter().filter(|p| p.exists()).map(|p| CFString::new(&p.to_string_lossy())).collect();
            if list.is_empty() {
                return None;
            }
            let array = CFArray::from_CFTypes(&list);
            let flag = Arc::into_raw(changed);
            let ctx = Context { version: 0, info: flag as *mut c_void, retain: std::ptr::null(), release: std::ptr::null(), copy_description: std::ptr::null() };
            unsafe {
                let stream = FSEventStreamCreate(std::ptr::null(), on_event, &ctx, array.as_concrete_TypeRef(), SINCE_NOW, super::LATENCY_S, super::FLAGS);
                if stream.is_null() {
                    drop(Arc::from_raw(flag));
                    return None;
                }
                let queue = dispatch_queue_create(c"app.fontcraft.watch".as_ptr(), std::ptr::null());
                FSEventStreamSetDispatchQueue(stream, queue);
                if FSEventStreamStart(stream) == 0 {
                    FSEventStreamInvalidate(stream);
                    FSEventStreamRelease(stream);
                    drop(Arc::from_raw(flag));
                    return None;
                }
                Some(Stream { stream, flag })
            }
        }

        pub fn stop(&mut self) {
            if self.stream.is_null() {
                return;
            }
            unsafe {
                FSEventStreamStop(self.stream);
                FSEventStreamInvalidate(self.stream);
                FSEventStreamRelease(self.stream);
                drop(Arc::from_raw(self.flag));
            }
            self.stream = std::ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices_a_new_file() {
        let dir = std::env::temp_dir().join(format!("fontcraft-watch-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        let w = Watcher::new(&[dir.clone()]).expect("watcher");
        std::thread::sleep(std::time::Duration::from_millis(500));
        // Written by another process, as Finder would ("ignore self" skips FontCraft's own writes).
        // (macOS: another process, as Finder would, since "ignore self" skips FontCraft's own writes;
        // elsewhere the folders' times are compared, so any writer counts.)
        if cfg!(target_os = "macos") {
            std::process::Command::new("cp").arg("/etc/hosts").arg(dir.join("sub").join("new.ttf")).status().unwrap();
        } else {
            std::fs::write(dir.join("sub").join("new.ttf"), b"font").unwrap();
        }
        let mut seen = false;
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if w.take_changed() {
                seen = true;
                break;
            }
        }
        drop(w);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(seen, "no change seen");
    }
}
