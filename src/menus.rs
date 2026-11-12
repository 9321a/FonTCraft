//! Right-click menus: on a library or folder in the sidebar (and each of its lists), and on a font
//! in the central list. "Activate" turns on every font there; "Add" opens the Import Fonts window
//! pointing at it; "Delete" asks first. Deleting a library or folder takes it out of FontCraft (its
//! files stay); deleting a list or a font moves its files to the system's Trash, so they can be
//! brought back.

use crate::i18n::{self, K};
use crate::importdlg::ImportDlg;
use crate::libs::LibKind;
use crate::nativemenu::{MIcon, NItem};
use crate::{activate, fonts::Source, oslook, App, Row, View};
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;

/// The activation asterisk in the menus, in the colours the font list shows it in.
const LIME_ASTERISK: MIcon = MIcon::colored("asterisk", [0x32, 0xcd, 0x32]);
const GRAY_ASTERISK: MIcon = MIcon::colored("asterisk", [0x9a, 0x9a, 0xa2]);

/// What the font menu needs to know about a family or style.
struct FontFacts {
    family: usize,
    fam_name: String,
    label: String,
    /// Every file of it.
    all: Vec<PathBuf>,
    /// The ones FontCraft can switch (libraries and Google Fonts).
    files: Vec<PathBuf>,
    all_on: bool,
    fav: bool,
    delete_label: &'static str,
}

/// Something waiting for the person to confirm.
pub enum Confirm {
    /// Take library/folder `i` out of FontCraft (files stay).
    RemoveLibrary(usize),
    /// Move these files or folders to the Trash.
    Trash { label: String, items: Vec<PathBuf> },
}

/// What a menu asked for (done after the menu closes).
pub enum MenuAction {
    Activate(Vec<PathBuf>),
    Deactivate(Vec<PathBuf>),
    /// On until FontCraft quits (RightFont's "Activate Temporarily": its fonts are turned off by
    /// -[RFFontService deactivateTemporarilyActivatedFonts], 0x100072ea0, when it quits).
    ActivateTemp(Vec<PathBuf>),
    Add(usize, Option<PathBuf>),
    Confirm(Confirm),
    Favorite(String, bool),
    /// Put a family into one of the smart lists (key from `smart_key`, family, list name).
    AddToSmart(&'static str, String, String),
    /// Copy files into a library or folder (and subfolder).
    AddToLibrary(crate::treemenu::Place, Vec<PathBuf>),
    /// Open the window that picks one of a big library's lists for these files.
    PickList(usize, Vec<PathBuf>),
    /// Copy files into a new folder named after the font, in a place the person picks.
    Export(String, Vec<PathBuf>),
    Reveal(PathBuf),
}

/// Lists of one library shown in the right-click menu; a library with more is picked from a window.
pub const MAX_LISTS_IN_MENU: usize = 40;
/// The list chooser's icons and indentation per nesting level, as RightFont's (0x10015ee30:
/// 16 x 16 icons; NSMenuItem indentation, about 12 points a level).
const PICK_ICON: f32 = 16.0;
const PICK_INDENT: f32 = 12.0;

/// The window picking one of a big library's lists: the library, the files and what is searched.
pub struct ListPick {
    pub lib: usize,
    pub files: Vec<PathBuf>,
    pub search: String,
}

/// The drawn icons of the font menu, in FontCraft's line style.
#[derive(Clone, Copy)]
enum Icon {
    On,
    Off,
    Clock,
    Star(bool),
    Smart,
    Library,
    Export,
    Reveal,
    Trash,
}

fn paint_icon(p: &egui::Painter, c: egui::Pos2, icon: Icon, col: egui::Color32) {
    use egui::{Stroke, Vec2};
    let st = Stroke::new(1.3_f32, col);
    match icon {
        Icon::On => crate::libs::paint_asterisk(p, c, crate::libs::LIME),
        Icon::Off => crate::libs::paint_asterisk(p, c, crate::libs::ASTERISK_GRAY),
        Icon::Clock => {
            p.circle_stroke(c, 6.5, st);
            p.line_segment([c, c + Vec2::new(0.0, -4.0)], st);
            p.line_segment([c, c + Vec2::new(3.0, 1.5)], st);
        }
        Icon::Star(filled) => crate::libs::paint_star(p, c, filled, col),
        Icon::Smart => {
            for (i, y) in [-4.0, 0.0, 4.0].into_iter().enumerate() {
                p.line_segment([c + Vec2::new(if i == 0 { -1.0 } else { -3.0 }, y), c + Vec2::new(6.0, y)], st);
            }
            p.circle_filled(c + Vec2::new(-5.5, -4.0), 1.6, col);
        }
        Icon::Library => crate::libs::paint_library(p, c, col),
        Icon::Export => {
            p.line_segment([c + Vec2::new(-6.0, -1.0), c + Vec2::new(-6.0, 6.0)], st);
            p.line_segment([c + Vec2::new(-6.0, 6.0), c + Vec2::new(6.0, 6.0)], st);
            p.line_segment([c + Vec2::new(6.0, 6.0), c + Vec2::new(6.0, -1.0)], st);
            p.line_segment([c + Vec2::new(0.0, 3.0), c + Vec2::new(0.0, -7.0)], st);
            p.line_segment([c + Vec2::new(-3.0, -4.0), c + Vec2::new(0.0, -7.0)], st);
            p.line_segment([c + Vec2::new(3.0, -4.0), c + Vec2::new(0.0, -7.0)], st);
        }
        Icon::Reveal => {
            p.circle_stroke(c + Vec2::new(-1.5, -1.5), 4.5, st);
            p.line_segment([c + Vec2::new(1.8, 1.8), c + Vec2::new(6.0, 6.0)], Stroke::new(1.8_f32, col));
        }
        Icon::Trash => {
            p.line_segment([c + Vec2::new(-6.0, -4.5), c + Vec2::new(6.0, -4.5)], st);
            p.line_segment([c + Vec2::new(-2.0, -6.5), c + Vec2::new(2.0, -6.5)], st);
            p.line_segment([c + Vec2::new(-4.5, -4.5), c + Vec2::new(-3.5, 6.5)], st);
            p.line_segment([c + Vec2::new(-3.5, 6.5), c + Vec2::new(3.5, 6.5)], st);
            p.line_segment([c + Vec2::new(3.5, 6.5), c + Vec2::new(4.5, -4.5)], st);
        }
    }
}

/// Room left before an item's text for its icon.
const ICON_PAD: &str = "          ";

/// A menu item: the system's own kind of button, with FontCraft's icon drawn before its text.
fn item(ui: &mut egui::Ui, icon: Icon, text: &str, enabled: bool) -> egui::Response {
    let r = ui.add_enabled(enabled, egui::Button::new(format!("{ICON_PAD}{text}")).frame(false).min_size(egui::vec2(220.0, 0.0)));
    let col = if enabled { ui.visuals().text_color() } else { ui.visuals().weak_text_color() };
    paint_icon(ui.painter(), r.rect.left_center() + egui::vec2(12.0, 0.0), icon, col);
    r
}

/// A menu item that opens a submenu, with its icon.
fn sub_item<R>(ui: &mut egui::Ui, icon: Icon, text: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> Option<R> {
    let (resp, inner) = egui::containers::menu::SubMenuButton::new(format!("{ICON_PAD}{text}")).ui(ui, add);
    let col = ui.visuals().text_color();
    paint_icon(ui.painter(), resp.rect.left_center() + egui::vec2(12.0, 0.0), icon, col);
    inner.map(|i| i.inner)
}

/// Saved names of the smart lists a family can be added to.
pub fn smart_key(view: View) -> Option<&'static str> {
    use crate::fonts::Class;
    Some(match view {
        View::Class(Class::Sans) => "sans",
        View::Class(Class::Serif) => "serif",
        View::Class(Class::Slab) => "slab",
        View::Class(Class::Script) => "script",
        View::Class(Class::Blackletter) => "blackletter",
        View::Class(Class::Display) => "display",
        View::Class(Class::Symbol) => "symbol",
        View::Mono => "mono",
        _ => return None,
    })
}

impl App {
    /// The font files that FontCraft may switch, under `dir` (a library, folder or subfolder).
    fn switchable_under(&self, dir: &std::path::Path) -> Vec<PathBuf> {
        self.lib
            .files
            .iter()
            .filter(|f| matches!(f.source, Source::Library | Source::Google) && f.path.starts_with(dir))
            .map(|f| f.path.clone())
            .collect()
    }

    /// The files of one list: a FontCraft library's family (by file name), or a folder's subfolder.
    fn set_files(&self, lib: usize, set: usize) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let l = &self.libraries[lib];
        let s = &l.sets[set];
        if l.kind == LibKind::Fcl || !s.files.is_empty() {
            let names: std::collections::HashSet<String> = s.files.iter().map(|f| f.to_lowercase()).collect();
            let files: Vec<PathBuf> = self
                .switchable_under(&l.path)
                .into_iter()
                .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| names.contains(&n.to_lowercase())))
                .collect();
            (files.clone(), files)
        } else {
            let dir = l.path.join(&s.rel);
            (self.switchable_under(&dir), vec![dir])
        }
    }

    /// The menu of a library or folder (`set` = None) or of one of its lists.
    pub(crate) fn library_menu(&self, ui: &mut egui::Ui, lib: usize, set: Option<usize>) -> Option<MenuAction> {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let l = &self.libraries[lib];
        let mut out = None;
        if ui.button(t(K::ActivateBtn)).clicked() {
            let files = match set {
                None => self.switchable_under(&l.path),
                Some(s) => self.set_files(lib, s).0,
            };
            out = Some(MenuAction::Activate(files));
            ui.close();
        }
        if ui.button(t(K::AddBtn)).clicked() {
            let sub = match set {
                Some(s) if l.kind == LibKind::Folder => Some(l.sets[s].rel.clone()),
                _ => None,
            };
            out = Some(MenuAction::Add(lib, sub));
            ui.close();
        }
        ui.separator();
        if ui.button(t(K::DeleteBtn)).clicked() {
            out = Some(MenuAction::Confirm(match set {
                None => Confirm::RemoveLibrary(lib),
                Some(s) => Confirm::Trash { label: format!("“{}”", l.sets[s].name), items: self.set_files(lib, s).1 },
            }));
            ui.close();
        }
        out
    }

    /// The menu of a font (family or one style) in the central list: Activate / Deactivate,
    /// Activate Temporarily; Add to / Remove from Favorites, Add to Smart List ▸, Add to Library ▸;
    /// Export, Show Location; Delete from the library or folder it is in. Every item has its icon.
    /// What the font menu needs to know about a family or style.
    fn font_facts(&self, row: Row) -> FontFacts {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let (family, faces): (usize, Vec<usize>) = match row {
            Row::Family(f) => (f, self.lib.families[f].faces.clone()),
            Row::Face(f, face) => (f, vec![face]),
        };
        let fam_name = self.lib.families[family].name.clone();
        let label = match row {
            Row::Family(_) => fam_name.clone(),
            Row::Face(_, face) => format!("{} {}", fam_name, self.lib.faces[face].style),
        };
        let all: Vec<PathBuf> = faces.iter().map(|&i| self.lib.files[self.lib.faces[i].file].path.clone()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
        // Only fonts from libraries and Google Fonts can be switched or deleted from FontCraft.
        let files: Vec<PathBuf> = faces
            .iter()
            .map(|&i| &self.lib.files[self.lib.faces[i].file])
            .filter(|f| matches!(f.source, Source::Library | Source::Google))
            .map(|f| f.path.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let all_on = !files.is_empty() && files.iter().all(|p| self.active.contains(p) || self.temp_active.contains(p) || self.auto_on_now(p));
        let home = all.first().and_then(|p| self.libraries.iter().find(|l| p.starts_with(&l.path)).map(|l| l.kind));
        let delete_label = match home {
            Some(LibKind::Fcl) => t(K::DeleteFromLibrary),
            Some(LibKind::Folder) => t(K::DeleteFromFolder),
            None => t(K::DeleteBtn),
        };
        FontFacts { family, fav: self.favorites.contains(&fam_name), fam_name, label, all, files, all_on, delete_label }
    }

    /// The font menu as the system's own menu (macOS, Windows): the items and what each one does.
    pub(crate) fn font_menu_native(&self, row: Row) -> (Vec<NItem>, HashMap<String, MenuAction>) {
        let t = |k: K| i18n::tr(self.settings.lang, k);
        let f = self.font_facts(row);
        let mut acts: HashMap<String, MenuAction> = HashMap::new();
        let mut item = |id: &str, label: &str, enabled: bool, icon: MIcon, act: MenuAction| -> NItem {
            acts.insert(id.to_string(), act);
            NItem::Item { id: id.to_string(), label: label.to_string(), enabled, icon: Some(icon) }
        };
        let mut v = Vec::new();
        if f.all_on {
            v.push(item("off", t(K::DeactivateBtn), true, GRAY_ASTERISK, MenuAction::Deactivate(f.files.clone())));
        } else {
            v.push(item("on", t(K::ActivateBtn), !f.files.is_empty(), LIME_ASTERISK, MenuAction::Activate(f.files.clone())));
        }
        v.push(item("temp", t(K::ActivateTemp), !f.files.is_empty() && !f.all_on, MIcon::line("clock"), MenuAction::ActivateTemp(f.files.clone())));
        v.push(NItem::Sep);
        v.push(item("fav", if f.fav { t(K::RemoveFavorite) } else { t(K::AddFavorite) }, true, MIcon::line(if f.fav { "star-filled" } else { "star" }), MenuAction::Favorite(f.fam_name.clone(), !f.fav)));
        let mut smart = Vec::new();
        for (i, (view, name)) in self.smart_lists().into_iter().enumerate() {
            let Some(key) = smart_key(view) else { continue };
            let has = self.in_smart_list(f.family, view);
            smart.push(item(&format!("smart{i}"), name, !has, MIcon::line(crate::uiicons::smart(key)), MenuAction::AddToSmart(key, f.fam_name.clone(), name.to_string())));
        }
        v.push(NItem::Sub { label: t(K::AddToSmart).to_string(), icon: Some(MIcon::line("add")), items: smart });
        // Every library and folder, each with its top level and all its lists.
        let order = crate::treemenu::order(&self.libraries);
        let mut libs = Vec::new();
        for (n, &i) in order.iter().enumerate() {
            let l = &self.libraries[i];
            if n > 0 && l.kind != self.libraries[order[n - 1]].kind {
                libs.push(NItem::Sep);
            }
            let mut sub = vec![item(&format!("lib{i}"), t(K::RootLevel), true, MIcon::line(if l.kind == LibKind::Fcl { "library" } else { "folder" }), MenuAction::AddToLibrary((i, None), f.all.clone()))];
            if !l.sets.is_empty() {
                sub.push(NItem::Sep);
            }
            // A library with many lists (one per family, thousands of them) would make a menu too
            // long to use and slow to open (3,575 entries took 2 s to build): its lists are picked
            // in a window with a search instead.
            if l.sets.len() > MAX_LISTS_IN_MENU {
                sub.push(item(&format!("pick{i}"), t(K::ChooseList), true, MIcon::line("add"), MenuAction::PickList(i, f.all.clone())));
                libs.push(NItem::Sub { label: l.name.clone(), icon: Some(MIcon::line(if l.kind == LibKind::Fcl { "library" } else { "folder" })), items: sub });
                continue;
            }
            for (s, set) in l.sets.iter().enumerate() {
                let place = if l.kind == LibKind::Folder { Some(set.rel.clone()) } else { None };
                let indent = "   ".repeat(set.depth.min(6));
                sub.push(item(&format!("lib{i}.{s}"), &format!("{indent}{}", set.name), true, MIcon::line(if l.kind == LibKind::Fcl { "family" } else { "set" }), MenuAction::AddToLibrary((i, place), f.all.clone())));
            }
            libs.push(NItem::Sub { label: l.name.clone(), icon: Some(MIcon::line(if l.kind == LibKind::Fcl { "library" } else { "folder" })), items: sub });
        }
        v.push(NItem::Sub { label: t(K::AddToLibrary).to_string(), icon: Some(MIcon::line("add")), items: libs });
        v.push(NItem::Sep);
        v.push(item("export", t(K::ExportBtn), !f.all.is_empty(), MIcon::line("export"), MenuAction::Export(f.label.clone(), f.all.clone())));
        if let Some(first) = f.all.first() {
            v.push(item("reveal", t(K::ShowLocation), true, MIcon::line("reveal"), MenuAction::Reveal(first.clone())));
        }
        v.push(NItem::Sep);
        v.push(item("delete", f.delete_label, !f.files.is_empty(), MIcon::line("trash"), MenuAction::Confirm(Confirm::Trash { label: format!("“{}”", f.label), items: f.files.clone() })));
        (v, acts)
    }

    /// The library/folder menu as the system's own menu (macOS, Windows).
    pub(crate) fn library_menu_native(&self, lib: usize, set: Option<usize>) -> (Vec<NItem>, HashMap<String, MenuAction>) {
        let t = |k: K| i18n::tr(self.settings.lang, k);
        let l = &self.libraries[lib];
        let files = match set {
            None => self.switchable_under(&l.path),
            Some(s) => self.set_files(lib, s).0,
        };
        let sub = match set {
            Some(s) if l.kind == LibKind::Folder => Some(l.sets[s].rel.clone()),
            _ => None,
        };
        let delete = match set {
            None => Confirm::RemoveLibrary(lib),
            Some(s) => Confirm::Trash { label: format!("“{}”", l.sets[s].name), items: self.set_files(lib, s).1 },
        };
        let mut acts = HashMap::new();
        acts.insert("on".to_string(), MenuAction::Activate(files));
        acts.insert("add".to_string(), MenuAction::Add(lib, sub));
        acts.insert("delete".to_string(), MenuAction::Confirm(delete));
        let v = vec![
            NItem::Item { id: "on".into(), label: t(K::ActivateBtn).into(), enabled: true, icon: Some(LIME_ASTERISK) },
            NItem::Item { id: "add".into(), label: t(K::AddBtn).into(), enabled: true, icon: Some(MIcon::line("add")) },
            NItem::Sep,
            NItem::Item { id: "delete".into(), label: t(K::DeleteBtn).into(), enabled: true, icon: Some(MIcon::line("trash")) },
        ];
        (v, acts)
    }

    /// Opens a menu as the system's own when the system offers one; true if it did (its answer then
    /// arrives through `native_menu_tick`).
    pub(crate) fn open_native_menu(&mut self, items: Vec<NItem>, acts: HashMap<String, MenuAction>) -> bool {
        if crate::nativemenu::popup("fontcraft-menu", items) {
            self.native_menu = acts;
            // The system's menu has closed by now (macOS): what was chosen is done at once, not
            // at the next redraw, which an idle window can leave for up to a second.
            self.native_menu_tick();
            true
        } else {
            false
        }
    }

    /// Each frame: does what was chosen in a native menu; while one is still waiting for its
    /// answer, the window looks again shortly.
    pub(crate) fn native_menu_wait(&mut self, ctx: &egui::Context) {
        self.native_menu_tick();
        if !self.native_menu.is_empty() && crate::nativemenu::waiting() {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }

    pub(crate) fn native_menu_tick(&mut self) {
        if let Some(id) = crate::nativemenu::take("fontcraft-menu") {
            if let Some(a) = self.native_menu.remove(&id) {
                self.native_menu.clear();
                self.run_menu_action(a);
            }
        }
    }

    pub(crate) fn font_menu(&self, ui: &mut egui::Ui, row: Row) -> Option<MenuAction> {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let pal = self.pal;
        let FontFacts { family, fam_name, label, all, files, all_on, .. } = self.font_facts(row);
        let mut out = None;
        ui.set_min_width(240.0);
        if all_on {
            if item(ui, Icon::Off, t(K::DeactivateBtn), true).clicked() {
                out = Some(MenuAction::Deactivate(files.clone()));
                ui.close();
            }
        } else if item(ui, Icon::On, t(K::ActivateBtn), !files.is_empty()).clicked() {
            out = Some(MenuAction::Activate(files.clone()));
            ui.close();
        }
        if item(ui, Icon::Clock, t(K::ActivateTemp), !files.is_empty() && !all_on).clicked() {
            out = Some(MenuAction::ActivateTemp(files.clone()));
            ui.close();
        }
        ui.separator();
        let fav = self.favorites.contains(&fam_name);
        if item(ui, Icon::Star(fav), if fav { t(K::RemoveFavorite) } else { t(K::AddFavorite) }, true).clicked() {
            out = Some(MenuAction::Favorite(fam_name.clone(), !fav));
            ui.close();
        }
        let smart = sub_item(ui, Icon::Smart, t(K::AddToSmart), |ui| {
            let mut pick = None;
            for (view, name) in self.smart_lists() {
                let Some(key) = smart_key(view) else { continue };
                let has = self.in_smart_list(family, view);
                if ui.add_enabled(!has, egui::Button::new(name).frame(false)).clicked() {
                    pick = Some(MenuAction::AddToSmart(key, fam_name.clone(), name.to_string()));
                    ui.close();
                }
            }
            pick
        });
        if let Some(a) = smart.flatten() {
            out = Some(a);
        }
        let lib = sub_item(ui, Icon::Library, t(K::AddToLibrary), |ui| {
            let order = crate::treemenu::order(&self.libraries);
            let mut pick = None;
            for kind in [LibKind::Fcl, LibKind::Folder] {
                if !order.iter().any(|&i| self.libraries[i].kind == kind) {
                    continue;
                }
                if let Some(place) = crate::treemenu::tree(ui, &pal, &self.libraries, &order, kind, None) {
                    pick = Some(MenuAction::AddToLibrary(place, all.clone()));
                }
                ui.separator();
            }
            pick
        });
        if let Some(a) = lib.flatten() {
            out = Some(a);
        }
        ui.separator();
        if item(ui, Icon::Export, t(K::ExportBtn), !all.is_empty()).clicked() {
            out = Some(MenuAction::Export(label.clone(), all.clone()));
            ui.close();
        }
        if item(ui, Icon::Reveal, t(K::ShowLocation), !all.is_empty()).clicked() {
            out = Some(MenuAction::Reveal(all[0].clone()));
            ui.close();
        }
        ui.separator();
        // "Delete from Library" or "Delete from Folder", after where the font is.
        let del = self.font_facts(row).delete_label;
        if item(ui, Icon::Trash, del, !files.is_empty()).clicked() {
            out = Some(MenuAction::Confirm(Confirm::Trash { label: format!("“{label}”"), items: files }));
            ui.close();
        }
        out
    }

    /// Does what a menu asked for.
    /// The window that picks one of a big library's lists (from the right-click menu).
    pub(crate) fn list_pick_window(&mut self, ctx: &egui::Context) {
        let Some(mut pk) = self.list_pick.take() else { return };
        let Some(l) = self.libraries.get(pk.lib) else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let fcl = l.kind == LibKind::Fcl;
        let title = format!("{} — {}", t(K::AddToLibrary), l.name);
        let q = pk.search.to_lowercase();
        // Each list with its nesting depth: RightFont's list chooser indents every level
        // (-[RFLibrarySelectDialog buildFontlistMenu:withNavigationItem:andIndentationLevel:],
        // 0x10015ee30: indentation level + 1 per level, 16 x 16 icons).
        let sets: Vec<(usize, String, Option<PathBuf>)> = l
            .sets
            .iter()
            .enumerate()
            .filter(|(_, s)| q.is_empty() || s.name.to_lowercase().contains(&q))
            .map(|(_, s)| (s.depth.min(6), s.name.clone(), if fcl { None } else { Some(s.rel.clone()) }))
            .collect();
        let mut chosen: Option<Option<PathBuf>> = None;
        let (_, closed) = crate::oslook::dialog(ctx, "list-pick", &title, 460.0, pal.panel, Some(520.0), |ui| {
            crate::oslook::field(ui, egui::TextEdit::singleline(&mut pk.search).hint_text(t(K::SearchLists)).desired_width(f32::INFINITY).margin(egui::Margin::symmetric(8, 5)));
            ui.add_space(6.0);
            if ui.selectable_label(false, format!("   {}", t(K::RootLevel))).clicked() {
                chosen = Some(None);
            }
            ui.separator();
            let row_h = 24.0;
            egui::ScrollArea::vertical().id_salt("list-pick-rows").auto_shrink([false, false]).max_height(400.0).show_rows(ui, row_h, sets.len(), |ui, range| {
                for (depth, name, place) in &sets[range] {
                    let indent = (*depth as f32) * PICK_INDENT;
                    let (r, resp) = ui.allocate_exact_size(egui::vec2(ui.available_width(), row_h), egui::Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(r, 5.0, pal.hover);
                    }
                    crate::uiicons::paint(ui, if fcl { "family" } else { "set" }, r.left_center() + egui::vec2(14.0 + indent, 0.0), PICK_ICON, pal.muted);
                    ui.painter().text(r.left_center() + egui::vec2(30.0 + indent, 0.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(13.5), pal.ink);
                    if resp.clicked() {
                        chosen = Some(place.clone());
                    }
                }
            });
        });
        if let Some(place) = chosen {
            let files = std::mem::take(&mut pk.files);
            self.run_menu_action(MenuAction::AddToLibrary((pk.lib, place), files));
            return;
        }
        if !closed {
            self.list_pick = Some(pk);
        }
    }

    pub(crate) fn run_menu_action(&mut self, action: MenuAction) {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        match action {
            MenuAction::Activate(files) => {
                let mut n = 0;
                for p in files {
                    if self.active.contains(&p) || activate::set(&p, true) {
                        self.active.insert(p);
                        n += 1;
                    }
                }
                self.save_active();
                self.notice = Some(t(K::ActivatedN).replace("{n}", &n.to_string()));
            }
            MenuAction::Deactivate(files) => {
                let mut n = 0;
                for p in files {
                    if activate::set(&p, false) {
                        n += 1;
                    }
                    self.active.remove(&p);
                    self.temp_active.remove(&p);
                    self.forget_auto(&p);
                }
                self.save_active();
                self.notice = Some(t(K::DeactivatedN).replace("{n}", &n.to_string()));
            }
            MenuAction::ActivateTemp(files) => {
                let mut n = 0;
                for p in files {
                    if self.active.contains(&p) || activate::set(&p, true) {
                        self.temp_active.insert(p);
                        n += 1;
                    }
                }
                self.notice = Some(t(K::TempActivatedN).replace("{n}", &n.to_string()));
            }
            MenuAction::Favorite(name, on) => {
                if on {
                    self.favorites.insert(name);
                } else {
                    self.favorites.remove(&name);
                }
                self.save_favorites();
            }
            MenuAction::AddToSmart(key, family, list) => {
                self.smart_extra.entry(key.to_string()).or_default().insert(family);
                self.save_smart_extra();
                self.notice = Some(t(K::AddedToList).replace("{list}", &list));
            }
            MenuAction::PickList(lib, files) => {
                self.list_pick = Some(ListPick { lib, files, search: String::new() });
            }
            MenuAction::AddToLibrary((lib, sub), files) => {
                let name = self.libraries.get(lib).map(|l| l.name.clone()).unwrap_or_default();
                let n = self.add_to_library(lib, sub.as_deref(), &files);
                self.notice = Some(t(K::AddedToLibraryN).replace("{n}", &n.to_string()).replace("{lib}", &name));
            }
            MenuAction::Export(name, files) => {
                // RightFont: an Open panel for a folder, "Select the destination for the exported
                // font files." (0x100021294); the files are copied into a folder named after the font,
                // made with its parents (-[RFFontService exportFonts:toLocation:], 0x10007e218).
                #[cfg(not(target_os = "android"))]
                if let Some(dir) = rfd::FileDialog::new().set_title(t(K::ExportPrompt)).pick_folder() {
                    self.notice = Some(match crate::fcl::export_fonts(&dir, &name, &files) {
                        Ok((n, to)) => t(K::ExportedN).replace("{n}", &n.to_string()).replace("{dir}", &to.display().to_string()),
                        Err(e) => e,
                    });
                }
                #[cfg(target_os = "android")]
                let _ = (name, files);
            }
            MenuAction::Reveal(path) => crate::paths::reveal(&path),
            MenuAction::Add(lib, sub) => {
                self.import_dlg = Some(ImportDlg::into_target(lib, sub));
            }
            MenuAction::Confirm(c) => self.confirm = Some(c),
        }
    }

    /// Takes a library or folder out of FontCraft: its fonts are switched off first; its files stay.
    pub(crate) fn remove_library(&mut self, i: usize) {
        let root = self.libraries[i].path.clone();
        let gone: Vec<PathBuf> = self.active.iter().filter(|p| p.starts_with(&root)).cloned().collect();
        for p in gone {
            activate::set(&p, false);
            self.active.remove(&p);
        }
        self.libraries.remove(i);
        self.lib_collapsed.clear();
        self.save_libraries();
        self.save_active();
        self.view = View::All;
        self.rescan();
    }

    /// The window asking to confirm a deletion.
    pub(crate) fn confirm_window(&mut self, ctx: &egui::Context) {
        let Some(c) = self.confirm.take() else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let text = match &c {
            Confirm::RemoveLibrary(i) => t(K::ConfirmRemove).replace("{name}", &self.libraries.get(*i).map(|l| l.name.clone()).unwrap_or_default()),
            Confirm::Trash { label, .. } => t(K::ConfirmTrash).replace("{name}", label),
        };
        let mut go = false;
        let mut cancel = false;
        let (_, closed) = oslook::dialog(ctx, "confirm-delete", t(K::DeleteBtn), 440.0, pal.panel, None, |ui| {
            ui.add(egui::Label::new(egui::RichText::new(&text).color(pal.ink)).wrap());
            ui.add_space(12.0);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), oslook::metrics().control_h + 4.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if oslook::default_button(ui, true, t(K::DeleteBtn)).clicked() {
                    go = true;
                }
                if ui.button(t(K::Cancel)).clicked() {
                    cancel = true;
                }
            });
        });
        if go {
            match c {
                Confirm::RemoveLibrary(i) => {
                    if i < self.libraries.len() {
                        self.remove_library(i);
                    }
                }
                Confirm::Trash { items, .. } => {
                    // Switched-on fonts are switched off before their files go.
                    for p in self.active.clone() {
                        if items.iter().any(|i| p.starts_with(i)) {
                            activate::set(&p, false);
                            self.active.remove(&p);
                        }
                    }
                    self.save_active();
                    self.notice = Some(match crate::paths::move_to_trash(&items) {
                        Ok(n) => t(K::TrashedN).replace("{n}", &n.to_string()),
                        Err(e) => e,
                    });
                    for l in self.libraries.iter_mut() {
                        l.sets = crate::libs::sets_for(&l.path, l.kind);
                    }
                    self.save_libraries();
                    self.rescan();
                }
            }
            return;
        }
        if !closed && !cancel {
            self.confirm = Some(c);
        }
    }
}
