//! Working with other apps on Windows and Linux, the way FontCraft does on macOS:
//! - Windows: the target app's window is found by its program (exe) name and brought to the front;
//!   a glyph is typed into it as a Unicode keystroke (SendInput with KEYEVENTF_UNICODE, so no smart
//!   paste adds spaces); a font change copies the selection (Ctrl+C), sets every font in the copied
//!   rich text (RTF, else HTML) to the new one and pastes it back (Ctrl+V), keeping sizes; the
//!   person's clipboard comes back afterwards.
//! - Linux (X11): with xdotool, the app's window is activated and the glyph typed. Wayland gives
//!   no way to type into other apps, so there the glyph stays on the clipboard to paste.

/// Brings the app to the front and types `text` into it. Ok(true) when typed.
#[cfg(target_os = "windows")]
pub fn insert(exes: &[&str], text: &str) -> bool {
    match win::front(exes) {
        Some(_) => {
            std::thread::sleep(std::time::Duration::from_millis(250));
            win::type_text(text);
            true
        }
        None => false,
    }
}

/// Changes the font of the selection in the app (copy, swap the fonts, paste). Ok(false) when
/// nothing was copied (no selection).
#[cfg(target_os = "windows")]
pub fn apply_font(exes: &[&str], font: &str) -> Result<bool, String> {
    if win::front(exes).is_none() {
        return Err("window".into());
    }
    std::thread::sleep(std::time::Duration::from_millis(250));
    let saved = win::clip_get();
    win::clip_set(&win::Clip { text: Some("\u{2063}fontcraft\u{2063}".into()), rtf: None, html: None });
    win::chord(0x43); // Ctrl+C
    let mut got = None;
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let c = win::clip_get();
        if c.text.as_deref() != Some("\u{2063}fontcraft\u{2063}") {
            got = Some(c);
            break;
        }
    }
    let Some(copied) = got else {
        win::clip_set(&saved);
        return Ok(false);
    };
    let plain = copied.text.clone().unwrap_or_default();
    let rtf = copied.rtf.as_deref().filter(|r| r.starts_with("{\\rtf"));
    let out = match (rtf, copied.html.as_deref()) {
        (Some(r), _) if r.contains("\\fs") || copied.html.is_none() => win::Clip { text: Some(plain.clone()), rtf: Some(crate::autoact::retarget_rtf(r, font).unwrap_or_else(|| crate::autoact::rtf_for(&plain, font))), html: None },
        (_, Some(h)) => win::Clip { text: Some(plain.clone()), rtf: None, html: Some(win::cf_html(&crate::autoact::retarget_html(&win::cf_html_fragment(h), font))) },
        _ => win::Clip { text: Some(plain.clone()), rtf: Some(crate::autoact::rtf_for(&plain, font)), html: None },
    };
    win::clip_set(&out);
    win::chord(0x56); // Ctrl+V
    std::thread::sleep(std::time::Duration::from_millis(1200));
    win::clip_set(&saved);
    Ok(true)
}

/// Selects the `n` characters before the cursor (Shift+Left), for a font change after typing.
#[cfg(target_os = "windows")]
pub fn select_back(n: usize) {
    for _ in 0..n {
        win::key_with_shift(0x25);
    }
}

#[cfg(target_os = "windows")]
pub fn caret_right() {
    win::key(0x27);
}

#[cfg(all(target_os = "linux", not(target_os = "android")))]
pub fn insert(names: &[&str], text: &str) -> bool {
    // X11 only: Wayland lets no app type into another.
    if std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var_os("DISPLAY").is_none() {
        return false;
    }
    let run = |args: &[&str]| crate::paths::command("xdotool").args(args).status().is_ok_and(|s| s.success());
    let found = names.iter().any(|n| {
        let class = n.rsplit('.').next().unwrap_or(n);
        run(&["search", "--onlyvisible", "--class", class, "windowactivate", "--sync"])
    });
    if !found {
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    run(&["type", "--clearmodifiers", "--delay", "20", "--", text])
}

#[cfg(target_os = "windows")]
mod win {
    use std::ffi::c_void;

    pub struct Clip {
        pub text: Option<String>,
        pub rtf: Option<String>,
        pub html: Option<String>,
    }

    #[repr(C)]
    struct KeybdInput {
        vk: u16,
        scan: u16,
        flags: u32,
        time: u32,
        extra: usize,
    }
    #[repr(C)]
    struct Input {
        kind: u32,
        ki: KeybdInput,
        _pad: [u8; 8],
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, l: isize) -> i32;
        fn IsWindowVisible(h: isize) -> i32;
        fn GetWindow(h: isize, cmd: u32) -> isize;
        fn GetWindowThreadProcessId(h: isize, pid: *mut u32) -> u32;
        fn SetForegroundWindow(h: isize) -> i32;
        fn ShowWindow(h: isize, cmd: i32) -> i32;
        fn IsIconic(h: isize) -> i32;
        fn SendInput(n: u32, inputs: *const Input, size: i32) -> u32;
        fn OpenClipboard(h: isize) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn GetClipboardData(f: u32) -> isize;
        fn SetClipboardData(f: u32, h: isize) -> isize;
        fn RegisterClipboardFormatW(name: *const u16) -> u32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
        fn CloseHandle(h: isize) -> i32;
        fn QueryFullProcessImageNameW(h: isize, flags: u32, buf: *mut u16, size: *mut u32) -> i32;
        fn GlobalAlloc(flags: u32, bytes: usize) -> isize;
        fn GlobalLock(h: isize) -> *mut c_void;
        fn GlobalUnlock(h: isize) -> i32;
        fn GlobalSize(h: isize) -> usize;
    }

    const INPUT_KEYBOARD: u32 = 1;
    const KEYEVENTF_KEYUP: u32 = 0x2;
    const KEYEVENTF_UNICODE: u32 = 0x4;
    const VK_CONTROL: u16 = 0x11;
    const VK_SHIFT: u16 = 0x10;
    const VK_MENU: u16 = 0x12;
    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 2;

    fn send(keys: &[(u16, u16, u32)]) {
        let inputs: Vec<Input> = keys.iter().map(|&(vk, scan, flags)| Input { kind: INPUT_KEYBOARD, ki: KeybdInput { vk, scan, flags, time: 0, extra: 0 }, _pad: [0; 8] }).collect();
        unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<Input>() as i32) };
    }

    pub fn type_text(text: &str) {
        for u in text.encode_utf16() {
            send(&[(0, u, KEYEVENTF_UNICODE), (0, u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP)]);
        }
    }

    pub fn chord(vk: u16) {
        send(&[(VK_CONTROL, 0, 0), (vk, 0, 0), (vk, 0, KEYEVENTF_KEYUP), (VK_CONTROL, 0, KEYEVENTF_KEYUP)]);
    }

    pub fn key(vk: u16) {
        send(&[(vk, 0, 0), (vk, 0, KEYEVENTF_KEYUP)]);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    pub fn key_with_shift(vk: u16) {
        send(&[(VK_SHIFT, 0, 0), (vk, 0, 0), (vk, 0, KEYEVENTF_KEYUP), (VK_SHIFT, 0, KEYEVENTF_KEYUP)]);
        std::thread::sleep(std::time::Duration::from_millis(30));
    }

    fn exe_of(pid: u32) -> String {
        unsafe {
            let h = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
            if h == 0 {
                return String::new();
            }
            let mut buf = vec![0u16; 1024];
            let mut n = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut n) != 0;
            CloseHandle(h);
            if !ok {
                return String::new();
            }
            let path = String::from_utf16_lossy(&buf[..n as usize]);
            path.rsplit(['\\', '/']).next().unwrap_or("").to_lowercase()
        }
    }

    struct Search {
        exes: Vec<String>,
        found: isize,
    }

    extern "system" fn each(h: isize, l: isize) -> i32 {
        let s = unsafe { &mut *(l as *mut Search) };
        // Visible top-level windows that aren't owned by another (the app's main windows).
        if unsafe { IsWindowVisible(h) } == 0 || unsafe { GetWindow(h, 4) } != 0 {
            return 1;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(h, &mut pid) };
        if s.exes.contains(&exe_of(pid)) {
            s.found = h;
            return 0;
        }
        1
    }

    /// The app's window, brought to the front (an Alt tap first: Windows lets a program bring
    /// another to the front only right after keyboard input).
    pub fn front(exes: &[&str]) -> Option<isize> {
        let mut s = Search { exes: exes.iter().map(|e| e.to_lowercase()).collect(), found: 0 };
        unsafe { EnumWindows(each, &mut s as *mut Search as isize) };
        if s.found == 0 {
            return None;
        }
        unsafe {
            if IsIconic(s.found) != 0 {
                ShowWindow(s.found, 9); // SW_RESTORE
            }
            send(&[(VK_MENU, 0, 0), (VK_MENU, 0, KEYEVENTF_KEYUP)]);
            SetForegroundWindow(s.found);
        }
        Some(s.found)
    }

    fn format(name: &str) -> u32 {
        let w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe { RegisterClipboardFormatW(w.as_ptr()) }
    }

    fn open() -> bool {
        for _ in 0..10 {
            if unsafe { OpenClipboard(0) } != 0 {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        false
    }

    fn read_bytes(f: u32) -> Option<Vec<u8>> {
        unsafe {
            let h = GetClipboardData(f);
            if h == 0 {
                return None;
            }
            let p = GlobalLock(h) as *const u8;
            if p.is_null() {
                return None;
            }
            let v = std::slice::from_raw_parts(p, GlobalSize(h)).to_vec();
            GlobalUnlock(h);
            Some(v)
        }
    }

    fn put_bytes(f: u32, bytes: &[u8]) {
        unsafe {
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
            if h == 0 {
                return;
            }
            let p = GlobalLock(h) as *mut u8;
            if p.is_null() {
                return;
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
            GlobalUnlock(h);
            SetClipboardData(f, h);
        }
    }

    fn until_nul(b: &[u8]) -> String {
        let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
        String::from_utf8_lossy(&b[..end]).to_string()
    }

    pub fn clip_get() -> Clip {
        let mut c = Clip { text: None, rtf: None, html: None };
        if !open() {
            return c;
        }
        if let Some(b) = read_bytes(CF_UNICODETEXT) {
            let u: Vec<u16> = b.chunks_exact(2).map(|p| u16::from_le_bytes([p[0], p[1]])).take_while(|&x| x != 0).collect();
            c.text = Some(String::from_utf16_lossy(&u));
        }
        c.rtf = read_bytes(format("Rich Text Format")).map(|b| until_nul(&b));
        c.html = read_bytes(format("HTML Format")).map(|b| until_nul(&b));
        unsafe { CloseClipboard() };
        c
    }

    pub fn clip_set(c: &Clip) {
        if !open() {
            return;
        }
        unsafe { EmptyClipboard() };
        if let Some(t) = &c.text {
            let mut b: Vec<u8> = t.encode_utf16().chain(std::iter::once(0)).flat_map(|u| u.to_le_bytes()).collect();
            b.extend_from_slice(&[0, 0]);
            put_bytes(CF_UNICODETEXT, &b);
        }
        if let Some(r) = &c.rtf {
            let mut b = r.as_bytes().to_vec();
            b.push(0);
            put_bytes(format("Rich Text Format"), &b);
        }
        if let Some(h) = &c.html {
            let mut b = h.as_bytes().to_vec();
            b.push(0);
            put_bytes(format("HTML Format"), &b);
        }
        unsafe { CloseClipboard() };
    }

    /// The HTML inside Windows' "HTML Format" (between StartFragment and EndFragment markers).
    pub fn cf_html_fragment(cf: &str) -> String {
        match (cf.find("<!--StartFragment-->"), cf.find("<!--EndFragment-->")) {
            (Some(a), Some(b)) if b > a => cf[a + 20..b].to_string(),
            _ => cf.split_once("<html").map(|(_, r)| format!("<html{r}")).unwrap_or_else(|| cf.to_string()),
        }
    }

    /// HTML wrapped in Windows' "HTML Format" header with its byte offsets.
    pub fn cf_html(fragment: &str) -> String {
        let pre = "<html><body><!--StartFragment-->";
        let post = "<!--EndFragment--></body></html>";
        let header_len = "Version:0.9\r\nStartHTML:0000000000\r\nEndHTML:0000000000\r\nStartFragment:0000000000\r\nEndFragment:0000000000\r\n".len();
        let start_html = header_len;
        let start_frag = start_html + pre.len();
        let end_frag = start_frag + fragment.len();
        let end_html = end_frag + post.len();
        format!("Version:0.9\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start_frag:010}\r\nEndFragment:{end_frag:010}\r\n{pre}{fragment}{post}")
    }
}
