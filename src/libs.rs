//! The person's own libraries (FontCraft .fcl libraries and plain folders) and switching fonts on and off.

use crate::fonts::{self, Source};
use crate::i18n::{self, K};
use crate::{activate, App, Row};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use crate::libscan;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub const LIME: Color32 = Color32::from_rgb(0x32, 0xcd, 0x32);
pub const ASTERISK_GRAY: Color32 = Color32::from_rgb(0x9a, 0x9a, 0xa2);

/// A list inside a library: one of its subfolders, under the name it had when added.
pub struct LibSet {
    pub name: String,
    pub rel: PathBuf,
    /// How deep the list is nested (0 = top level).
    pub depth: usize,
    /// For lists read from RightFont metadata: the font file names in it (otherwise its folder decides).
    pub files: Vec<String>,
}

impl LibSet {
    pub(crate) fn from_sub(s: &libscan::Sub) -> Self {
        LibSet { name: s.name.clone(), rel: s.rel.clone(), depth: s.depth, files: s.files.clone() }
    }
}

pub struct UserLibrary {
    pub name: String,
    pub path: PathBuf,
    /// False for a library added only for this run (automatic checks); it isn't written to disk.
    pub saved: bool,
    /// A FontCraft library (.fcl, under "My Libraries") or a plain folder (under "My Folders").
    pub kind: LibKind,
    pub sets: Vec<LibSet>,
    /// Font file names (without extension) in the library's own order.
    pub order: Vec<String>,
}

/// Which families each library and list holds, and their order (worked out after each scan).
#[derive(Default)]
pub struct LibMembers {
    pub sets: Vec<HashSet<usize>>,
    pub order: HashMap<usize, usize>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LibKind {
    /// A FontCraft library: ~/FontCraft/<name>.fcl, fonts sorted into folders A–Z.
    Fcl,
    /// A folder on the computer, used where it is.
    Folder,
}

#[derive(Default)]
pub struct NewLib {
    pub path: String,
    pub use_folder_name: bool,
    pub custom: String,
    /// Making a FontCraft library (.fcl) rather than adding a folder.
    pub fcl: bool,
    pub(crate) preview: Option<libscan::Preview>,
    pub(crate) previewed_for: String,
}

impl NewLib {
    /// Add a folder (My Folders).
    pub fn new(path: String) -> Self {
        NewLib { path, use_folder_name: true, ..Default::default() }
    }
    /// Make a FontCraft library (My Libraries).
    pub fn library() -> Self {
        NewLib { fcl: true, use_folder_name: true, ..Default::default() }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActState {
    LockedSystem,
    LockedInstalled,
    /// Turned on and off by Adobe Creative Cloud.
    LockedAdobe,
    On,
    Off,
}

pub(crate) fn data_dir() -> PathBuf {
    crate::paths::data_dir()
}

fn write(name: &str, text: String) {
    let dir = data_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join(name), text);
}

/// The lists shown under a library in the sidebar. A folder: its subfolders. A FontCraft library:
/// its font families by their full names, in name order (inside the .fcl the files are kept in
/// A–Z folders, but that's only how the file is organised).
pub(crate) fn sets_for(path: &std::path::Path, kind: LibKind) -> Vec<LibSet> {
    match kind {
        LibKind::Folder => libscan::preview(path).subs.iter().map(LibSet::from_sub).collect(),
        LibKind::Fcl => {
            let mut files = Vec::new();
            fonts::walk(path, &mut files);
            let mut by_family: std::collections::BTreeMap<String, (String, Vec<String>)> = Default::default();
            for f in files {
                let ext = f.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
                if !matches!(ext.as_deref(), Some("ttf" | "otf" | "ttc" | "otc")) {
                    continue;
                }
                let family = crate::fcl::family_name(&f);
                let Some(file) = f.file_name().map(|n| n.to_string_lossy().to_string()) else { continue };
                by_family.entry(family.to_lowercase()).or_insert_with(|| (family.clone(), Vec::new())).1.push(file);
            }
            by_family.into_values().map(|(name, files)| LibSet { name, rel: PathBuf::new(), depth: 0, files }).collect()
        }
    }
}

pub(crate) fn library_from(name: String, path: PathBuf, kind: LibKind, saved: bool, pv: &libscan::Preview) -> UserLibrary {
    let sets = if kind == LibKind::Fcl { sets_for(&path, kind) } else { pv.subs.iter().map(LibSet::from_sub).collect() };
    UserLibrary {
        name,
        path,
        saved,
        kind,
        sets,
        order: pv.order.clone(),
    }
}

/// Saved libraries. Each starts with an "L" line; its lists ("S") and font order ("O") follow.
/// (Older files with just "name<TAB>path" lines still load.)
pub fn load_libraries() -> Vec<UserLibrary> {
    let text = std::fs::read_to_string(data_dir().join("libraries.txt")).unwrap_or_default();
    let mut v: Vec<UserLibrary> = Vec::new();
    // Lines that belong to a dropped entry (RightFont libraries are no longer supported).
    let mut skipping = false;
    for line in text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if skipping && matches!(parts.first(), Some(&"S") | Some(&"O")) {
            continue;
        }
        skipping = false;
        match parts.as_slice() {
            ["L", _, _, "rightfont"] => skipping = true,
            ["L", name, path, kind] => {
                let kind = if *kind == "fcl" { LibKind::Fcl } else { LibKind::Folder };
                let path = PathBuf::from(path);
                // A FontCraft library's lists are its letter folders: read them again each time.
                let sets = if kind == LibKind::Fcl { sets_for(&path, kind) } else { Vec::new() };
                skipping = kind == LibKind::Fcl;
                v.push(UserLibrary { name: name.to_string(), path, saved: true, kind, sets, order: Vec::new() })
            }
            ["S", name, rel, depth, files] => {
                if let Some(l) = v.last_mut() {
                    l.sets.push(LibSet {
                        name: name.to_string(),
                        rel: PathBuf::from(rel),
                        depth: depth.parse().unwrap_or(0),
                        files: files.split('|').filter(|f| !f.is_empty()).map(String::from).collect(),
                    });
                }
            }
            ["S", name, rel] => {
                // Saved before lists were nested: read the folder again.
                if let Some(l) = v.last_mut() {
                    if l.sets.is_empty() {
                        let pv = libscan::preview(&l.path);
                        l.sets = pv.subs.iter().map(LibSet::from_sub).collect();
                    }
                    let _ = (name, rel);
                }
            }
            ["O", stem] => {
                if let Some(l) = v.last_mut() {
                    l.order.push(stem.to_string());
                }
            }
            [name, path] => {
                // Older format: build the lists from the folder now.
                let path = PathBuf::from(path);
                let pv = libscan::preview(&path);
                v.push(library_from(name.to_string(), path, LibKind::Folder, true, &pv));
            }
            _ => {}
        }
    }
    // FontCraft libraries other users of this computer share with this one (Settings › General).
    for (name, path) in crate::fcl::shared_with_me() {
        if !v.iter().any(|l| l.path == path) {
            let pv = libscan::preview(&path);
            v.push(library_from(name, path, LibKind::Fcl, false, &pv));
        }
    }
    // Lets the automatic checks show a library without saving it.
    for (var, kind) in [("FONTCRAFT_TEST_LIBRARY", LibKind::Folder), ("FONTCRAFT_TEST_FCL", LibKind::Fcl)] {
        if let Ok(p) = std::env::var(var) {
            let path = PathBuf::from(p);
            let pv = libscan::preview(&path);
            v.push(library_from(libscan::library_name(&path), path, kind, false, &pv));
        }
    }
    v
}

pub fn load_active() -> HashSet<PathBuf> {
    std::fs::read_to_string(data_dir().join("active.txt"))
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// The row icons (asterisk, padlock, format, star) share one design: an 18-point square, 1.6–2.6
/// point strokes with round ends, grey when off and coloured when on.
pub const ICON: f32 = 18.0;
pub const STAR_ON: Color32 = Color32::from_rgb(0xf5, 0xa6, 0x23);

/// Gray asterisk (off) or lime-green asterisk (on), 18 points across; points follow the screen's scale.
pub fn paint_asterisk(p: &egui::Painter, c: Pos2, col: Color32) {
    let r = 7.5;
    for k in 0..3 {
        let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::PI / 3.0;
        let d = Vec2::angled(a) * r;
        p.line_segment([c - d, c + d], Stroke::new(2.6_f32, col));
    }
    p.circle_filled(c, 1.8, col);
}

/// A closed padlock: this font can't be switched (same 18-point square as the asterisk).
pub fn paint_padlock(p: &egui::Painter, c: Pos2, col: Color32) {
    let body = Rect::from_center_size(c + Vec2::new(0.0, 3.0), Vec2::new(13.0, 10.0));
    p.rect_filled(body, 2.5, col);
    let top = c + Vec2::new(0.0, -2.5);
    let r = 4.2;
    let mut pts = vec![Pos2::new(top.x - r, body.top())];
    for i in 0..=12 {
        let a = std::f32::consts::PI + i as f32 * std::f32::consts::PI / 12.0;
        pts.push(top + Vec2::new(r * a.cos(), r * a.sin()));
    }
    pts.push(Pos2::new(top.x + r, body.top()));
    p.add(egui::Shape::line(pts, Stroke::new(2.0_f32, col)));
}

/// The font's format: a page with a folded corner and its short name ("OTF", "TTF", "VAR"…).
pub fn paint_format(p: &egui::Painter, c: Pos2, label: &str, col: Color32) {
    let r = Rect::from_center_size(c, Vec2::new(14.0, ICON));
    let fold = 4.5;
    let st = Stroke::new(1.6_f32, col);
    let pts = vec![
        r.left_top(),
        Pos2::new(r.right() - fold, r.top()),
        Pos2::new(r.right(), r.top() + fold),
        r.right_bottom(),
        r.left_bottom(),
    ];
    p.add(egui::Shape::closed_line(pts, st));
    p.line_segment([Pos2::new(r.right() - fold, r.top()), Pos2::new(r.right() - fold, r.top() + fold)], st);
    p.line_segment([Pos2::new(r.right() - fold, r.top() + fold), Pos2::new(r.right(), r.top() + fold)], st);
    p.text(c + Vec2::new(0.0, 2.5), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(8.0), col);
}

/// A five-pointed star (favourite): an outline when off, filled when on.
pub fn paint_star(p: &egui::Painter, c: Pos2, filled: bool, col: Color32) {
    let (outer, inner) = (8.6, 3.7);
    let pts: Vec<Pos2> = (0..10)
        .map(|i| {
            let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            c + Vec2::angled(a) * if i % 2 == 0 { outer } else { inner }
        })
        .collect();
    if filled {
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(c, col);
        for q in &pts {
            mesh.colored_vertex(*q, col);
        }
        for i in 0..10u32 {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % 10);
        }
        p.add(mesh);
    }
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.6_f32, col)));
}

/// A FontCraft library: three books on a shelf (one leaning).
pub(crate) fn paint_library(p: &egui::Painter, c: Pos2, col: Color32) {
    let st = Stroke::new(1.3_f32, col);
    let base = c.y + 6.0;
    p.rect_stroke(Rect::from_min_max(Pos2::new(c.x - 7.0, base - 11.0), Pos2::new(c.x - 3.5, base)), 0.8, st, eframe::egui::StrokeKind::Inside);
    p.rect_stroke(Rect::from_min_max(Pos2::new(c.x - 3.0, base - 13.0), Pos2::new(c.x + 0.5, base)), 0.8, st, eframe::egui::StrokeKind::Inside);
    let lean = [Pos2::new(c.x + 1.5, base), Pos2::new(c.x + 5.0, base), Pos2::new(c.x + 8.0, base - 10.0), Pos2::new(c.x + 4.5, base - 11.0)];
    p.add(egui::Shape::closed_line(lean.to_vec(), st));
    p.line_segment([Pos2::new(c.x - 8.0, base + 0.8), Pos2::new(c.x + 8.5, base + 0.8)], Stroke::new(1.4_f32, col));
}

/// A font family inside a FontCraft library: "Aa" on a small page.
pub(crate) fn paint_family(p: &egui::Painter, c: Pos2, col: Color32) {
    p.rect_stroke(Rect::from_center_size(c, Vec2::new(14.0, 11.0)), 2.0, Stroke::new(1.1_f32, col), eframe::egui::StrokeKind::Inside);
    p.text(c + Vec2::new(0.0, 0.3), egui::Align2::CENTER_CENTER, "Aa", egui::FontId::proportional(9.5), col);
}

pub(crate) fn paint_folder(p: &egui::Painter, c: Pos2, col: Color32) {
    let body = Rect::from_center_size(c + Vec2::new(0.0, 1.0), Vec2::new(14.0, 10.0));
    p.rect_stroke(body, 2.0, Stroke::new(1.3_f32, col), eframe::egui::StrokeKind::Inside);
    p.line_segment([body.left_top() + Vec2::new(1.0, -2.0), body.left_top() + Vec2::new(6.0, -2.0)], Stroke::new(1.6_f32, col));
}

impl App {
    pub(crate) fn library_paths(&self) -> Vec<PathBuf> {
        self.libraries.iter().map(|l| l.path.clone()).collect()
    }

    pub(crate) fn save_libraries(&self) {
        let clean = |s: &str| s.replace(['\t', '\n'], " ");
        let mut text = String::new();
        for l in self.libraries.iter().filter(|l| l.saved) {
            let kind = if l.kind == LibKind::Fcl { "fcl" } else { "folder" };
            text += &format!("L\t{}\t{}\t{kind}\n", clean(&l.name), l.path.display());
            for s in &l.sets {
                text += &format!("S\t{}\t{}\t{}\t{}\n", clean(&s.name), s.rel.display(), s.depth, clean(&s.files.join("|")));
            }
            for o in &l.order {
                text += &format!("O\t{}\n", clean(o));
            }
        }
        write("libraries.txt", text);
    }

    /// Works out which families are in each library's lists, and the library's order.
    pub(crate) fn compute_members(&mut self) {
        let mut by_file: HashMap<PathBuf, usize> = HashMap::new();
        for (i, f) in self.lib.files.iter().enumerate() {
            by_file.insert(std::fs::canonicalize(&f.path).unwrap_or(f.path.clone()), i);
        }
        let mut family_of_file: HashMap<usize, Vec<usize>> = HashMap::new();
        for (fi, fam) in self.lib.families.iter().enumerate() {
            for &face in &fam.faces {
                family_of_file.entry(self.lib.faces[face].file).or_default().push(fi);
            }
        }
        let mut members = Vec::new();
        for (li, l) in self.libraries.iter().enumerate() {
            let mut m = LibMembers::default();
            // Font files inside the library, by file name (for lists read from RightFont metadata).
            let mut lib_paths = Vec::new();
            fonts::walk(&l.path, &mut lib_paths);
            let by_name: HashMap<String, PathBuf> = lib_paths
                .iter()
                .filter_map(|p| Some((p.file_name()?.to_str()?.to_lowercase(), p.clone())))
                .collect();
            for set in &l.sets {
                let paths: Vec<PathBuf> = if set.files.is_empty() {
                    let mut paths = Vec::new();
                    fonts::walk(&l.path.join(&set.rel), &mut paths);
                    paths
                } else {
                    set.files.iter().filter_map(|f| by_name.get(&f.to_lowercase()).cloned()).collect()
                };
                let fams: HashSet<usize> = paths
                    .iter()
                    .filter_map(|p| by_file.get(&std::fs::canonicalize(p).unwrap_or(p.clone())))
                    .flat_map(|fi| family_of_file.get(fi).cloned().unwrap_or_default())
                    .collect();
                m.sets.push(fams);
            }
            // A list also holds everything in the lists under it (RightFont's recursive query, 0x10028a4b4).
            for s in (0..l.sets.len()).rev() {
                let depth = l.sets[s].depth;
                let mut extra = HashSet::new();
                for c in s + 1..l.sets.len() {
                    if l.sets[c].depth <= depth {
                        break;
                    }
                    extra.extend(m.sets[c].iter().copied());
                }
                m.sets[s].extend(extra);
            }
            let pos: HashMap<String, usize> = l.order.iter().enumerate().map(|(i, s)| (s.to_lowercase(), i)).collect();
            for (fi, fam) in self.lib.families.iter().enumerate() {
                if !fam.libs.contains(&li) {
                    continue;
                }
                let best = fam
                    .faces
                    .iter()
                    .filter_map(|&f| {
                        let p = &self.lib.files[self.lib.faces[f].file].path;
                        p.file_stem().and_then(|s| s.to_str()).and_then(|s| pos.get(&s.to_lowercase()).copied())
                    })
                    .min();
                if let Some(b) = best {
                    m.order.insert(fi, b);
                }
            }
            members.push(m);
        }
        self.members = members;
    }

    pub(crate) fn save_active(&self) {
        let mut v: Vec<String> = self.active.iter().map(|p| p.display().to_string()).collect();
        v.sort();
        write("active.txt", v.join("\n"));
    }

    /// Reads all fonts again (after libraries change), right away: for changes the person just made
    /// and expects to see at once (a library added, fonts imported).
    pub(crate) fn rescan(&mut self) {
        let lib = fonts::scan(&self.library_paths());
        self.use_scan(lib);
    }

    /// Reads all fonts again in the background (a folder changed on disk, a download finished):
    /// reading some 12,000 styles takes over a second, and the window keeps answering meanwhile.
    /// A change that comes while one is being read is read again after it.
    pub(crate) fn rescan_soon(&mut self) {
        if self.scanning.is_some() {
            self.scan_again = true;
            return;
        }
        let paths = self.library_paths();
        let slot = std::sync::Arc::new(std::sync::Mutex::new(None));
        let out = slot.clone();
        std::thread::spawn(move || {
            let lib = fonts::scan(&paths);
            *out.lock().unwrap() = Some(lib);
        });
        self.scanning = Some(slot);
    }

    /// Each frame: puts a background reading in place once it's done.
    pub(crate) fn finish_rescan(&mut self, ctx: &egui::Context) {
        let Some(slot) = &self.scanning else { return };
        let got = slot.lock().unwrap().take();
        match got {
            Some(lib) => {
                self.scanning = None;
                let view = self.view;
                self.use_scan(lib);
                self.view = view;
                if std::mem::take(&mut self.scan_again) {
                    self.rescan_soon();
                }
                ctx.request_repaint();
            }
            None => ctx.request_repaint_after(std::time::Duration::from_millis(200)),
        }
    }

    /// Puts a fresh reading of the fonts in place, keeping what the person had chosen.
    fn use_scan(&mut self, lib: fonts::Library) {
        // Keep the selected family selected (families are found again by name).
        let was = match self.selected {
            Some(Row::Family(f)) | Some(Row::Face(f, _)) => self.lib.families.get(f).map(|f| f.name.clone()),
            None => None,
        };
        self.lib = lib;
        self.compute_members();
        self.publish_font_names();
        self.cache.clear();
        self.extents.clear();
        self.mark_cache.clear();
        self.search_index = std::cell::OnceCell::new();
        self.expanded.clear();
        self.face_data = None;
        self.restore_open_state();
        let again = was.and_then(|n| self.lib.families.iter().position(|f| f.name == n));
        self.selected = match again {
            Some(i) => Some(Row::Family(i)),
            None if self.lib.families.is_empty() => None,
            None => Some(Row::Family(0)),
        };
    }

    /// Hands the names of the fonts FontCraft can turn on (libraries and Google Fonts) to the
    /// font-request handler and the Missing Fonts check: each face's PostScript, full and family
    /// name, lower-case, read in the background.
    pub(crate) fn publish_font_names(&self) {
        let faces: Vec<(PathBuf, std::sync::Arc<memmap2::Mmap>, u32, String, String)> = self
            .lib
            .faces
            .iter()
            .filter_map(|f| {
                let file = &self.lib.files[f.file];
                matches!(file.source, Source::Library | Source::Google).then(|| (file.path.clone(), file.data.clone(), f.index, f.family.clone(), f.style.clone()))
            })
            .collect();
        std::thread::spawn(move || {
            let mut index: std::collections::HashMap<String, Vec<PathBuf>> = std::collections::HashMap::new();
            let add = |k: String, p: &PathBuf, index: &mut std::collections::HashMap<String, Vec<PathBuf>>| {
                let v = index.entry(k.trim().to_lowercase()).or_default();
                if !v.contains(p) {
                    v.push(p.clone());
                }
            };
            for (path, data, i, family, style) in &faces {
                add(family.clone(), path, &mut index);
                add(format!("{family} {style}"), path, &mut index);
                if let Ok(face) = ttf_parser::Face::parse(data, *i) {
                    for n in face.names() {
                        if matches!(n.name_id, ttf_parser::name_id::POST_SCRIPT_NAME | ttf_parser::name_id::FULL_NAME) {
                            if let Some(s) = n.to_string() {
                                add(s, path, &mut index);
                            }
                        }
                    }
                }
            }
            crate::fontreq::set_index(index);
        });
    }

    /// Turns on again the fonts that were on when the app last ran.
    pub(crate) fn restore_active(&mut self) {
        let known: HashSet<PathBuf> = self
            .lib
            .files
            .iter()
            .filter(|f| matches!(f.source, Source::Library | Source::Google))
            .map(|f| f.path.clone())
            .collect();
        let wanted: Vec<PathBuf> = self.active.iter().cloned().collect();
        // What a crash or an earlier session left in FontCraft's user font folder goes first.
        if self.screenshot.is_none() {
            activate::sweep(&wanted);
        }
        for p in wanted {
            // Test pictures show the fonts kept on without turning anything on in the system.
            if self.screenshot.is_some() {
                if !known.contains(&p) {
                    self.active.remove(&p);
                }
                continue;
            }
            if !known.contains(&p) || !activate::set(&p, true) {
                self.active.remove(&p);
            }
        }
        // Fonts an earlier version left on for good in the system (and not kept on now) go off.
        let leftovers: Vec<PathBuf> = known.into_iter().filter(|p| !self.active.contains(p)).collect();
        if self.screenshot.is_none() {
            std::thread::spawn(move || activate::clear_leftovers(&leftovers));
        }
    }

    pub(crate) fn file_state(&self, file: usize) -> ActState {
        let f = &self.lib.files[file];
        match f.source {
            Source::System => ActState::LockedSystem,
            Source::Computer | Source::User => ActState::LockedInstalled,
            Source::Adobe => ActState::LockedAdobe,
            Source::Library | Source::Google => {
                // Switched on by hand, or for an app (Auto Activation): both show as on.
                if self.active.contains(&f.path) || self.temp_active.contains(&f.path) || self.auto_on_now(&f.path) {
                    ActState::On
                } else {
                    ActState::Off
                }
            }
        }
    }

    fn row_files(&self, row: Row) -> Vec<usize> {
        let mut files: Vec<usize> = match row {
            Row::Family(f) => self.lib.families[f].faces.iter().map(|&i| self.lib.faces[i].file).collect(),
            Row::Face(_, face) => vec![self.lib.faces[face].file],
        };
        files.sort_unstable();
        files.dedup();
        files
    }

    pub(crate) fn row_state(&self, row: Row) -> ActState {
        let states: Vec<ActState> = self.row_files(row).into_iter().map(|f| self.file_state(f)).collect();
        if states.iter().any(|s| *s == ActState::On) {
            ActState::On
        } else if states.iter().any(|s| *s == ActState::Off) {
            ActState::Off
        } else if states.iter().any(|s| *s == ActState::LockedSystem) {
            ActState::LockedSystem
        } else if states.iter().any(|s| *s == ActState::LockedAdobe) {
            ActState::LockedAdobe
        } else {
            ActState::LockedInstalled
        }
    }

    /// Clicking the asterisk: everything switchable in the row goes on (or off if it was on).
    pub(crate) fn toggle_row(&mut self, row: Row) {
        let target = self.row_state(row) != ActState::On;
        let mut failed = false;
        for f in self.row_files(row) {
            if matches!(self.file_state(f), ActState::On | ActState::Off) {
                let path = self.lib.files[f].path.clone();
                if activate::set(&path, target) {
                    if target {
                        self.active.insert(path);
                    } else {
                        self.forget_auto(&path);
                        self.active.remove(&path);
                    }
                } else {
                    failed = true;
                }
            }
        }
        self.notice = if failed { Some(i18n::tr(self.settings.lang, K::ActivateFailed).to_string()) } else { None };
        self.save_active();
    }

    /// The asterisk or padlock for one row, with its own click and tooltip.
    pub(crate) fn activation_control(&mut self, ui: &mut egui::Ui, row: Row, center: Pos2) {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let state = self.row_state(row);
        let rect = Rect::from_center_size(center, Vec2::splat(ICON + 4.0));
        let id = ui.scope_id().with(("activate", match row {
            Row::Family(f) => (0, f),
            Row::Face(_, f) => (1, f),
        }));
        let switchable = matches!(state, ActState::On | ActState::Off);
        let resp = ui.interact(rect, id, if switchable { Sense::click() } else { Sense::hover() });
        let p = ui.painter();
        match state {
            ActState::On => paint_asterisk(p, center, LIME),
            ActState::Off => paint_asterisk(p, center, if resp.hovered() { LIME.gamma_multiply(0.6) } else { ASTERISK_GRAY }),
            _ => paint_padlock(p, center, ASTERISK_GRAY),
        }
        let tip = match state {
            ActState::On => t(K::ClickDeactivate),
            ActState::Off => t(K::ClickActivate),
            ActState::LockedSystem => t(K::LockedSystem),
            ActState::LockedInstalled => t(K::LockedInstalled),
            ActState::LockedAdobe => t(K::ManagedByAdobe),
        };
        if resp.on_hover_text(tip).clicked() && switchable {
            self.toggle_row(row);
        }
    }

}
