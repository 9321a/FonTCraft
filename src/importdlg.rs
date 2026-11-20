//! The Import Fonts window:
//! - "In": a pop-up menu offering only "Library ▸" and "Folder ▸"; nothing is chosen in advance.
//!   Each shows the whole tree of the libraries or folders with their icons (`treemenu`), so fonts
//!   can go straight into one of them or one of their subfolders.
//!   "Add" copies chosen font files and/or folders there (a FontCraft library files them into A–Z
//!   folders itself; every library lists its fonts alphabetically).
//! - "Create": a pop-up menu (Library / Folder) and "Add", which opens the same window as the
//!   sidebar's "Libraries +" (a new library, or a .fcl made earlier) or "Folders +".

use crate::i18n::{self, K};
use crate::libs::{LibKind, NewLib};
use crate::{fcl, oslook, App, View};
use eframe::egui;
use std::path::PathBuf;

/// Width of both pop-up menus in the window (they match).
const POPUP_W: f32 = 300.0;

#[derive(Default)]
pub struct ImportDlg {
    /// Where fonts go: a library (index) and, for a folder, one of its subfolders (relative path).
    /// None until chosen: the first library or folder in alphabetical order is used.
    pub target: Option<usize>,
    pub sub: Option<PathBuf>,
    /// What the last "Add" did.
    pub status: Option<String>,
    /// "Create": a library (false) or a folder (true).
    pub create_folder: Option<bool>,
}

impl ImportDlg {
    /// The window opened from a library's or list's right-click menu, already pointing at it.
    pub fn into_target(lib: usize, sub: Option<PathBuf>) -> ImportDlg {
        ImportDlg { target: Some(lib), sub, ..Default::default() }
    }
}

impl App {
    /// Copies fonts (files or whole folders) into library `target` (or one of its subfolders) and
    /// reads it again.
    pub(crate) fn add_to_library(&mut self, target: usize, sub: Option<&std::path::Path>, picked: &[PathBuf]) -> usize {
        let (path, kind) = (self.libraries[target].path.clone(), self.libraries[target].kind);
        let copied = match (kind, sub) {
            // A plain folder: into the chosen subfolder.
            (LibKind::Folder, Some(rel)) if !rel.as_os_str().is_empty() => fcl::add_fonts(&path.join(rel), LibKind::Folder, picked),
            _ => fcl::add_fonts(&path, kind, picked),
        };
        // The library's lists follow its families (a library) or folders (a folder), so read them again.
        self.libraries[target].sets = crate::libs::sets_for(&path, kind);
        self.save_libraries();
        self.rescan();
        self.view = View::Lib(target);
        copied
    }

    pub(crate) fn import_window(&mut self, ctx: &egui::Context) {
        let Some(mut dlg) = self.import_dlg.take() else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        // Libraries first, then folders, each in alphabetical order. Nothing is chosen in advance.
        let order = crate::treemenu::order(&self.libraries);
        if dlg.target.is_some_and(|t| t >= self.libraries.len()) {
            dlg.target = None;
            dlg.sub = None;
        }
        // What the pop-up shows: "Library / Folder" until one is chosen, then it and its subfolder.
        let shown = dlg
            .target
            .map(|i| {
                let name = self.libraries[i].name.clone();
                match &dlg.sub {
                    Some(rel) if !rel.as_os_str().is_empty() => format!("{name} › {}", rel.display()),
                    _ => name,
                }
            })
            .unwrap_or_else(|| format!("{} / {}", t(K::LibraryTab), t(K::Folder)));
        // Fonts or folders dropped on the window go into the chosen place.
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        let mut add: Vec<PathBuf> = Vec::new();
        if dlg.target.is_some() {
            add.extend(dropped);
        }
        let mut create: Option<NewLib> = None;
        let mut cancel = false;
        let (_, closed) = oslook::dialog(ctx, "import-fonts", t(K::ImportFonts), 560.0, pal.panel, None, |ui| {
            egui::Grid::new("import-grid").num_columns(3).spacing([12.0, 12.0]).show(ui, |ui| {
                ui.label(egui::RichText::new(t(K::InShort)).strong().color(pal.ink));
                if order.is_empty() {
                    ui.label(egui::RichText::new(t(K::NoLibraries)).color(pal.warn));
                    ui.label("");
                } else {
                    let chosen = dlg.target.map(|i| (i, dlg.sub.clone()));
                    let libs = &self.libraries;
                    // The system's own menu where it offers one (Library ▸ / Folder ▸, each with every
                    // library or folder and its lists), else FontCraft's tree menu.
                    let native_items = || {
                        use crate::nativemenu::{MIcon, NItem};
                        let mut v = Vec::new();
                        for (kind, label) in [(LibKind::Fcl, t(K::LibraryTab)), (LibKind::Folder, t(K::Folder))] {
                            let mut items = Vec::new();
                            for &i in order.iter().filter(|&&i| libs[i].kind == kind) {
                                let l = &libs[i];
                                let top = if kind == LibKind::Fcl { "library" } else { "folder" };
                                let mut sub = vec![NItem::Item { id: format!("{i}"), label: t(K::RootLevel).into(), enabled: true, icon: Some(MIcon::line(top)) }];
                                if !l.sets.is_empty() {
                                    sub.push(NItem::Sep);
                                    for (si, set) in l.sets.iter().enumerate() {
                                        sub.push(NItem::Item { id: format!("{i}.{si}"), label: format!("{}{}", "   ".repeat(set.depth.min(6)), set.name), enabled: true, icon: Some(MIcon::line(if kind == LibKind::Fcl { "library" } else { "set" })) });
                                    }
                                }
                                items.push(NItem::Sub { label: l.name.clone(), icon: Some(MIcon::line(top)), items: sub });
                            }
                            v.push(NItem::Sub { label: label.into(), icon: Some(MIcon::line(if kind == LibKind::Fcl { "library" } else { "folder" })), items });
                        }
                        v
                    };
                    let (picked, _) = oslook::native_popup(ui, "import-in", &shown, POPUP_W, native_items, |ui| {
                        for (kind, label) in [(LibKind::Fcl, t(K::LibraryTab)), (LibKind::Folder, t(K::Folder))] {
                            ui.menu_button(label, |ui| {
                                if let Some((i, sub)) = crate::treemenu::tree(ui, &pal, libs, &order, kind, chosen.as_ref()) {
                                    dlg.target = Some(i);
                                    dlg.sub = sub;
                                }
                            });
                        }
                    });
                    if let Some(id) = picked {
                        let mut parts = id.split('.');
                        if let Some(i) = parts.next().and_then(|v| v.parse::<usize>().ok()).filter(|&i| i < libs.len()) {
                            dlg.target = Some(i);
                            dlg.sub = parts.next().and_then(|v| v.parse::<usize>().ok()).and_then(|si| libs[i].sets.get(si)).map(|s| s.rel.clone());
                        }
                    }
                    // Font files one by one, or whole folders.
                    if ui.add_enabled(dlg.target.is_some(), egui::Button::new(t(K::AddBtn))).clicked() {
                        add.extend(fcl::pick_fonts_or_folders());
                    }
                }
                ui.end_row();
            });
            if let Some(s) = &dlg.status {
                ui.label(egui::RichText::new(s).size(12.5).color(pal.accent));
            }
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            egui::Grid::new("create-grid").num_columns(3).spacing([12.0, 12.0]).show(ui, |ui| {
                ui.label(egui::RichText::new(t(K::CreateLabel)).strong().color(pal.ink));
                // "Library / Folder" until one is chosen, as "In" shows it.
                let both = format!("{} / {}", t(K::LibraryTab), t(K::Folder));
                let kind_label: &str = match dlg.create_folder {
                    Some(true) => t(K::Folder),
                    Some(false) => t(K::LibraryTab),
                    None => &both,
                };
                // The same kind of pop-up as "In", the same size.
                let native_items = || {
                    use crate::nativemenu::{MIcon, NItem};
                    vec![
                        NItem::Item { id: "lib".into(), label: t(K::LibraryTab).into(), enabled: true, icon: Some(MIcon::line("library")) },
                        NItem::Item { id: "folder".into(), label: t(K::Folder).into(), enabled: true, icon: Some(MIcon::line("folder")) },
                    ]
                };
                let (picked, _) = oslook::native_popup(ui, "import-create", kind_label, POPUP_W, native_items, |ui| {
                    if ui.selectable_label(dlg.create_folder == Some(false), t(K::LibraryTab)).clicked() {
                        dlg.create_folder = Some(false);
                        ui.close();
                    }
                    if ui.selectable_label(dlg.create_folder == Some(true), t(K::Folder)).clicked() {
                        dlg.create_folder = Some(true);
                        ui.close();
                    }
                });
                if let Some(id) = picked {
                    dlg.create_folder = Some(id == "folder");
                }
                if ui.add_enabled(dlg.create_folder.is_some(), egui::Button::new(t(K::AddBtn))).clicked() {
                    create = dlg.create_folder.map(|f| if f { NewLib::new(String::new()) } else { NewLib::library() });
                }
                ui.end_row();
            });
            ui.add_space(10.0);
            // One row high, button on the right (the window keeps the height of its content).
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), oslook::metrics().control_h + 4.0),
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    if ui.button(t(K::Cancel)).clicked() {
                        cancel = true;
                    }
                },
            );
        });
        if !add.is_empty() {
            if let Some(target) = dlg.target {
                let sub = dlg.sub.clone();
                let n = self.add_to_library(target, sub.as_deref(), &add);
                let msg = t(K::ImportedN).replace("{n}", &n.to_string());
                dlg.status = Some(msg.clone());
                self.notice = Some(msg);
            }
        }
        if let Some(nl) = create {
            // The sidebar's New Library / New Folder window takes over.
            self.new_lib = Some(nl);
            self.settings_open = false;
            return;
        }
        if !closed && !cancel {
            self.import_dlg = Some(dlg);
        }
    }
}
