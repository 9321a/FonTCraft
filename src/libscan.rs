//! Looking at a folder (or a FontCraft .fcl library) before it is added: its subfolders and the
//! fonts in each.
//!
//! Rules taken from RightFont (see MECHANICS.md):
//! - a folder's subfolders become nested lists, each named after its own folder (last part of the
//!   path), hanging under its parent folder's list (-[RFFontService loadCollectionsAndFontsFromURLs:
//!   andParentCollection:toLiveLibrary:] at 0x100080b44);
//! - a library's lists are shown parent first, then children, each level ordered by when the list
//!   was created (the query at 0x100296ae0: "… WHERE parent IS NULL … ORDER BY created");
//! - a list includes the fonts of all the lists under it (the recursive query at 0x10028a4b4).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Sub {
    /// The list's name as it is now (a folder's own name; kept even if renamed later).
    pub name: String,
    /// Where its folder is inside the library.
    pub rel: PathBuf,
    /// How deep it is nested (0 = top level).
    pub depth: usize,
    /// Fonts shown for it in the New Library window.
    pub fonts: Vec<String>,
    /// Unused (kept so older saved lists still load).
    pub files: Vec<String>,
}

#[derive(Default)]
pub struct Preview {
    pub root_fonts: Vec<String>,
    pub subs: Vec<Sub>,
    pub total: usize,
    /// Kept for older saved libraries; fonts are listed by name, so this stays empty.
    pub order: Vec<String>,
}

fn is_font(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("ttf" | "otf" | "ttc" | "otc")
    )
}

/// Every file under `dir`, in name order.
fn walk_sorted(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    entries.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default());
    for p in entries {
        if p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.')) {
            continue;
        }
        if p.is_dir() {
            walk_sorted(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// A font's full name ("Founders Grotesk Bold"), or its file name if it has none.
fn font_name(path: &Path) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let Ok(f) = std::fs::File::open(path) else { return stem };
    let Ok(map) = (unsafe { memmap2::Mmap::map(&f) }) else { return stem };
    let Ok(face) = ttf_parser::Face::parse(&map, 0) else { return stem };
    let mut fallback = None;
    for n in face.names() {
        if n.name_id == ttf_parser::name_id::FULL_NAME {
            if let Some(s) = n.to_string() {
                if n.language() == ttf_parser::Language::English_UnitedStates {
                    return s;
                }
                fallback.get_or_insert(s);
            }
        }
    }
    fallback.unwrap_or(stem)
}

/// A library's name from its folder: "Clients.fcl" becomes "Clients".
pub fn library_name(path: &Path) -> String {
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("Library");
    match name.rsplit_once('.') {
        Some((base, ext)) if !base.is_empty() && ext.len() <= 20 && !ext.contains(' ') => base.to_string(),
        _ => name.to_string(),
    }
}

pub fn preview(root: &Path) -> Preview {
    let mut files = Vec::new();
    walk_sorted(root, &mut files);
    let fonts: Vec<PathBuf> = files.into_iter().filter(|p| is_font(p)).collect();

    // A plain folder: every folder holding fonts (at any depth) becomes a list, nested like the
    // folders, named after its own folder. RightFont stamps each with the time it was added, in the
    // order it reads them; here they are read in name order.
    let mut pv = Preview { total: fonts.len(), ..Default::default() };
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut direct: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for f in &fonts {
        let rel_dir = f.parent().and_then(|d| d.strip_prefix(root).ok()).map(Path::to_path_buf).unwrap_or_default();
        if rel_dir.as_os_str().is_empty() {
            pv.root_fonts.push(font_name(f));
            continue;
        }
        direct.entry(rel_dir.clone()).or_default().push(font_name(f));
        // The folder and every folder above it (below the root) are lists.
        let mut d = rel_dir.clone();
        loop {
            if !dirs.contains(&d) {
                dirs.push(d.clone());
            }
            match d.parent() {
                Some(p) if !p.as_os_str().is_empty() => d = p.to_path_buf(),
                _ => break,
            }
        }
    }
    let key = |p: &PathBuf| p.iter().map(|c| c.to_string_lossy().to_lowercase()).collect::<Vec<_>>();
    dirs.sort_by_key(|d| key(d));
    pv.subs = dirs
        .into_iter()
        .map(|rel| Sub {
            name: rel.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            depth: rel.iter().count().saturating_sub(1),
            fonts: direct.remove(&rel).unwrap_or_default(),
            files: Vec::new(),
            rel,
        })
        .collect();
    pv
}
