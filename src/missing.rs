//! Missing Fonts, as in RightFont ("RightFont detected %lu missing %@ in this document.",
//! -[RFFontActivator handleMissingFonts:targetID:appName:] 0x10004faf8; "%lu missing %@ activated").
//! FontCraft notices missing fonts two ways, on every system it can:
//! - the document open in front in a supported app is read (`docfonts`) whenever it changes: its
//!   fonts that aren't installed or on are listed;
//! - on macOS, fonts an app asks the system for that FontCraft doesn't have (`fontreq`).
//! The window lists them in two groups: in the catalog (one click, "Activate All", turns them on for
//! that app and its document; they go off when the document or the app closes) and not in the
//! catalog, each with "Replace with…" to pick a catalog font that is turned on and answers for the
//! missing one from then on.

use crate::i18n::{self, K};
use crate::{oslook, App};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MissingState {
    pub open: bool,
    /// The document (or app) the fonts are missing from.
    pub doc: String,
    /// The app (bundle id / key) it is open in.
    pub bundle: String,
    pub names: Vec<String>,
    /// Missing font → the catalog family chosen to stand in for it.
    pub replaced: HashMap<String, String>,
    pub search: String,
    /// Documents already read (path and modification time).
    seen: HashSet<(PathBuf, u64)>,
    /// A document being read in the background: (path, app key) → its fonts.
    job: Arc<Mutex<Option<(PathBuf, String, Vec<String>)>>>,
    /// A document is being read (RightFont's scanning flag, self+0x20 in
    /// -[RFFontActivator scanUsedFontsWithCommonCode:…] 0x10004dae4).
    busy: bool,
    /// A document that turned up while one was being read; read as soon as that one finishes
    /// (RightFont's rescan flag, self+0x21, run again by -[RFFontActivator finishMissingFontScan]
    /// 0x10004d564).
    next: Option<(PathBuf, String)>,
}

impl MissingState {
    /// With the replacements chosen in earlier runs.
    pub fn new() -> Self {
        MissingState { replaced: load_substitutes(), ..Default::default() }
    }
}

fn substitutes_path() -> PathBuf {
    crate::paths::data_dir().join("replacements.txt")
}

/// Missing font → replacement family, as the person chose them (kept between runs).
pub fn load_substitutes() -> HashMap<String, String> {
    std::fs::read_to_string(substitutes_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once('\t').map(|(a, b)| (a.to_string(), b.to_string())))
        .collect()
}

impl App {
    /// The family names that are installed or on now (they aren't missing).
    fn available_names(&self) -> HashSet<String> {
        use crate::fonts::Source;
        let mut v = HashSet::new();
        for fam in &self.lib.families {
            let on = fam.faces.iter().any(|&i| {
                let f = &self.lib.files[self.lib.faces[i].file];
                match f.source {
                    Source::Library | Source::Google => self.active.contains(&f.path) || self.temp_active.contains(&f.path) || self.auto_on_now(&f.path),
                    _ => true,
                }
            });
            if on {
                v.insert(fam.name.to_lowercase());
            }
        }
        v
    }

    /// A font an app asked for that FontCraft doesn't have (macOS font requests).
    pub(crate) fn add_missing(&mut self, font: String, app: String) {
        if !self.missing.open {
            self.missing.names.clear();
            self.missing.doc = app;
            self.missing.bundle.clear();
        }
        if !self.missing.names.contains(&font) {
            self.missing.names.push(font);
        }
        self.missing.open = true;
    }

    /// Each frame: reads the front document of the app FontCraft works with when it changes, and
    /// opens the window when fonts are missing from it.
    pub(crate) fn missing_tick(&mut self) {
        if self.screenshot.is_some() {
            // Test pictures: FONTCRAFT_TEST_MISSING="Font A,Font B" opens the window with those names.
            if self.frame == 20 {
                if let Ok(v) = std::env::var("FONTCRAFT_TEST_MISSING") {
                    for n in v.split(',') {
                        self.add_missing(n.trim().to_string(), "Test.docx".into());
                    }
                }
            }
            return;
        }
        // A finished read.
        let done = self.missing.job.lock().unwrap().take();
        if done.is_some() {
            self.missing.busy = false;
            // The one that waited is read now.
            if let Some((path, bundle)) = self.missing.next.take() {
                self.read_document(path, bundle);
            }
        }
        if let Some((path, bundle, fonts)) = done {
            let have = self.available_names();
            // The system's own interface fonts are never missing (RightFont's rule, 0x10004f89c).
            let names: Vec<String> = fonts.into_iter().filter(|f| !have.contains(&f.to_lowercase()) && !crate::fontreq::is_system_ui(f)).collect();
            if !names.is_empty() {
                self.missing.names = names;
                self.missing.doc = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                self.missing.bundle = bundle;
                self.missing.open = true;
            }
        }
        // The front document, as the watcher saw it.
        let front = self.autoact.front_document();
        if let Some((path, bundle)) = front {
            if !crate::docfonts::readable(&path) {
                return;
            }
            let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
            if self.missing.seen.insert((path.clone(), mtime)) {
                if self.missing.busy {
                    self.missing.next = Some((path, bundle));
                } else {
                    self.read_document(path, bundle);
                }
            }
        }
    }

    /// Reads one document's fonts in the background; one at a time.
    fn read_document(&mut self, path: PathBuf, bundle: String) {
        self.missing.busy = true;
        let job = self.missing.job.clone();
        std::thread::spawn(move || {
            let fonts = crate::docfonts::fonts_in(&path);
            *job.lock().unwrap() = Some((path, bundle, fonts));
        });
    }

    /// Turns on the catalog fonts for these names, for the app and document they are missing from.
    fn activate_missing(&mut self, names: &[String]) -> usize {
        let mut files: Vec<PathBuf> = Vec::new();
        for n in names {
            let key = self.missing.replaced.get(n).cloned().unwrap_or_else(|| n.clone());
            if let Some(f) = crate::fontreq::lookup(&key) {
                files.extend(f);
            }
        }
        files.sort();
        files.dedup();
        let mut on = Vec::new();
        for f in files {
            if self.active.contains(&f) || crate::activate::set(&f, true) {
                on.push(f);
            }
        }
        let n = on.len();
        if !on.is_empty() {
            let bundle = self.missing.bundle.clone();
            let doc = (!self.missing.doc.is_empty() && !bundle.is_empty()).then(|| self.missing.doc.clone());
            self.autoact.record(on, bundle, doc);
        }
        n
    }

    pub(crate) fn missing_window(&mut self, ctx: &egui::Context) {
        if !self.missing.open || self.missing.names.is_empty() {
            self.missing.open = false;
            return;
        }
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let names = self.missing.names.clone();
        let (found, not_found): (Vec<String>, Vec<String>) = names.iter().cloned().partition(|n| crate::fontreq::lookup(n).is_some() || self.missing.replaced.contains_key(n));
        let families: Vec<String> = {
            let q = self.missing.search.to_lowercase();
            self.lib.families.iter().filter(|f| matches!(f.source, crate::fonts::Source::Library | crate::fonts::Source::Google)).map(|f| f.name.clone()).filter(|n| q.is_empty() || n.to_lowercase().contains(&q)).take(300).collect()
        };
        let mut activate_all = false;
        let mut cancel = false;
        let mut pick: Option<(String, String)> = None;
        let doc = self.missing.doc.clone();
        let mut search = std::mem::take(&mut self.missing.search);
        let replaced = self.missing.replaced.clone();
        let (_, closed) = oslook::dialog_centered(ctx, "missing-fonts", t(K::MissingTitle), 560.0, pal.panel, None, |ui| {
            ui.add(egui::Label::new(egui::RichText::new(t(K::MissingIntro).replace("{n}", &names.len().to_string()).replace("{doc}", &doc)).color(pal.ink)).wrap());
            ui.add_space(8.0);
            if !found.is_empty() {
                ui.label(egui::RichText::new(t(K::MissingInCatalog)).size(12.0).strong().color(pal.muted));
                for n in &found {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                        crate::libs::paint_asterisk(ui.painter(), r.center(), crate::libs::LIME);
                        let label = match replaced.get(n) {
                            Some(r) => t(K::ReplacedWith).replace("{old}", n).replace("{new}", r),
                            None => n.clone(),
                        };
                        ui.add(egui::Label::new(egui::RichText::new(label).color(pal.ink)).truncate());
                    });
                }
                ui.add_space(6.0);
            }
            if !not_found.is_empty() {
                ui.label(egui::RichText::new(t(K::MissingNotFound)).size(12.0).strong().color(pal.muted));
                for n in &not_found {
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::hover());
                        crate::libs::paint_asterisk(ui.painter(), r.center(), crate::libs::ASTERISK_GRAY);
                        let room = (ui.available_width() - 190.0).max(60.0);
                        ui.add_sized([room, 20.0], egui::Label::new(egui::RichText::new(n).color(pal.warn)).truncate());
                        oslook::menu_popup(ui, t(K::ReplaceWith), 170.0, |ui| {
                            oslook::field(ui, egui::TextEdit::singleline(&mut search).hint_text("…").desired_width(240.0));
                            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                                for f in &families {
                                    if ui.add(egui::Button::new(f).frame(false)).clicked() {
                                        pick = Some((n.clone(), f.clone()));
                                        ui.close();
                                    }
                                }
                            });
                        });
                    });
                }
            }
            ui.add_space(12.0);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), oslook::metrics().control_h + 4.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if oslook::default_button(ui, !found.is_empty(), t(K::ActivateAll)).clicked() {
                    activate_all = true;
                }
                if ui.button(t(K::Cancel)).clicked() {
                    cancel = true;
                }
            });
        });
        self.missing.search = search;
        if let Some((old, new)) = pick {
            self.missing.replaced.insert(old.clone(), new.clone());
            let lines: Vec<String> = self.missing.replaced.iter().map(|(a, b)| format!("{a}\t{b}")).collect();
            let _ = std::fs::write(substitutes_path(), lines.join("\n"));
            crate::fontreq::set_substitutes(self.missing.replaced.clone());
            let n = self.activate_missing(std::slice::from_ref(&old));
            let _ = n;
            self.notice = Some(t(K::ReplacedWith).replace("{old}", &old).replace("{new}", &new));
        }
        if activate_all {
            let n = self.activate_missing(&found);
            self.notice = Some(t(K::MissingActivatedN).replace("{n}", &n.to_string()));
            self.missing.open = false;
        }
        if closed || cancel {
            self.missing.open = false;
        }
    }
}
