//! Where FontCraft keeps things on each system.
//!
//! - macOS: `~/Library/Application Support/FontCraft`, caches in `~/Library/Caches/FontCraft`.
//! - Windows: `%APPDATA%\FontCraft`, caches in `%LOCALAPPDATA%\FontCraft\Cache`.
//! - Linux: `$XDG_DATA_HOME/FontCraft` (`~/.local/share/FontCraft`), caches in `~/.cache/fontcraft`.
//! - Portable copy (Windows mostly): when a file named `portable.txt` sits next to the program,
//!   everything is kept in `FontCraftData` beside it, so the folder can travel on a USB stick.

use std::path::PathBuf;

/// The person's home folder (HOME, or USERPROFILE on Windows).
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// The folder of a portable copy, if this is one.
pub fn portable_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    dir.join("portable.txt").exists().then(|| dir.join("FontCraftData"))
}

fn env_dir(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// Settings, favourites, library list, downloaded Google Fonts.
pub fn data_dir() -> PathBuf {
    if let Some(p) = portable_dir() {
        return p;
    }
    if cfg!(target_os = "macos") {
        home().join("Library/Application Support/FontCraft")
    } else if cfg!(target_os = "windows") {
        env_dir("APPDATA").unwrap_or_else(|| home().join("AppData").join("Roaming")).join("FontCraft")
    } else {
        env_dir("XDG_DATA_HOME").unwrap_or_else(|| home().join(".local/share")).join("FontCraft")
    }
}

/// Files that can be made again (dragged glyph pictures).
pub fn cache_dir() -> PathBuf {
    if let Some(p) = portable_dir() {
        return p.join("Cache");
    }
    if cfg!(target_os = "macos") {
        home().join("Library/Caches/FontCraft")
    } else if cfg!(target_os = "windows") {
        env_dir("LOCALAPPDATA").unwrap_or_else(|| home().join("AppData").join("Local")).join("FontCraft").join("Cache")
    } else {
        env_dir("XDG_CACHE_HOME").unwrap_or_else(|| home().join(".cache")).join("fontcraft")
    }
}

/// Where Adobe Creative Cloud keeps the fonts it turns on.
pub fn adobe_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        env_dir("APPDATA").unwrap_or_else(|| home().join("AppData").join("Roaming")).join("Adobe").join("CoreSync").join("plugins").join("livetype").join("r")
    } else {
        home().join("Library/Application Support/Adobe/CoreSync/plugins/livetype/.r")
    }
}

/// The system's font folders: (folder, 0 = system, 1 = everyone on this computer, 2 = this person).
pub fn font_roots() -> Vec<(PathBuf, u8)> {
    let h = home();
    if cfg!(target_os = "android") {
        return vec![
            (PathBuf::from("/system/fonts"), 0),
            (PathBuf::from("/product/fonts"), 0),
            (PathBuf::from("/storage/emulated/0/Fonts"), 2),
            (PathBuf::from("/sdcard/Fonts"), 2),
            (PathBuf::from("/storage/emulated/0/Fonts/FontCraft"), 2),
            (PathBuf::from("/sdcard/Fonts/FontCraft"), 2),
            (PathBuf::from("/storage/emulated/0/Download/Fonts"), 2),
            (PathBuf::from("/storage/emulated/0/Android/media/app.fontcraft/Fonts"), 2),
            (h.join("fonts"), 2),
        ];
    }
    if cfg!(target_os = "windows") {
        let windir = env_dir("WINDIR").unwrap_or_else(|| PathBuf::from("C:\\Windows"));
        let mut v = vec![(windir.join("Fonts"), 0)];
        if let Some(l) = env_dir("LOCALAPPDATA") {
            v.push((l.join("Microsoft").join("Windows").join("Fonts"), 2));
        }
        v
    } else {
        vec![
            (PathBuf::from("/System/Library/Fonts"), 0),
            (PathBuf::from("/Library/Fonts"), 1),
            (h.join("Library/Fonts"), 2),
            (PathBuf::from("/usr/share/fonts"), 0),
            (PathBuf::from("/usr/local/share/fonts"), 1),
            (h.join(".local/share/fonts"), 2),
            (h.join(".fonts"), 2),
        ]
    }
}

/// Every outside program FontCraft runs goes through here. On Windows it is started without a
/// console window (CREATE_NO_WINDOW); otherwise each run (the app check every 1.5 s, the registry,
/// printing…) flashed an empty terminal window open and shut.
pub fn command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    #[allow(unused_mut)]
    let mut c = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}

/// Opens a web address or file with the system's default app (macOS, Windows, Linux).
pub fn open_external(target: &str) {
    let _ = if cfg!(target_os = "macos") {
        command("open").arg(target).spawn()
    } else if cfg!(target_os = "windows") {
        command("rundll32").args(["url.dll,FileProtocolHandler", target]).spawn()
    } else {
        command("xdg-open").arg(target).spawn()
    };
}

/// Shows a file selected in the system's file manager: macOS Finder (`open -R`, RightFont's
/// "Show in Finder"), Windows Explorer (`/select,`), Linux the file manager's ShowItems (the
/// freedesktop FileManager1 service), else its folder.
pub fn reveal(path: &std::path::Path) {
    if cfg!(target_os = "macos") {
        let _ = command("open").arg("-R").arg(path).spawn();
    } else if cfg!(target_os = "windows") {
        let _ = command("explorer").arg(format!("/select,{}", path.display())).spawn();
    } else {
        let uri = format!("file://{}", path.display());
        let shown = command("dbus-send")
            .args(["--session", "--print-reply", "--dest=org.freedesktop.FileManager1", "/org/freedesktop/FileManager1", "org.freedesktop.FileManager1.ShowItems"])
            .arg(format!("array:string:{uri}"))
            .arg("string:")
            .status()
            .is_ok_and(|s| s.success());
        if !shown {
            if let Some(dir) = path.parent() {
                open_external(&dir.display().to_string());
            }
        }
    }
}

/// Moves files or folders to the system's Trash / Recycle Bin (they can be brought back from there).
/// Returns how many went.
pub fn move_to_trash(items: &[PathBuf]) -> Result<usize, String> {
    let existing: Vec<&PathBuf> = items.iter().filter(|p| p.exists()).collect();
    #[cfg(not(target_os = "android"))]
    {
        trash::delete_all(&existing).map_err(|e| e.to_string())?;
        Ok(existing.len())
    }
    #[cfg(target_os = "android")]
    {
        let _ = existing;
        Err("no Trash on Android".into())
    }
}
