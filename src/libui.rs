//! The sidebar's "Libraries" and "Folders", and the New Library / Add Folder window.

use crate::i18n::{self, K};
use crate::libs::{library_from, paint_family, paint_folder, paint_library, LibKind, NewLib};
use crate::{fcl, libscan, oslook, App, View};
use eframe::egui::{self, FontId, Margin, Rect, Sense, Stroke, Vec2};
use std::path::{Path, PathBuf};

impl App {
    /// The sidebar's "MY LIBRARIES" (FontCraft .fcl libraries) and "MY FOLDERS" (folders added from
    /// the computer): both show each entry with its lists (subfolders) underneath, the same way.
    pub(crate) fn sidebar_libraries(&mut self, ui: &mut egui::Ui) {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        self.library_section(ui, LibKind::Fcl, t(K::LibrariesHead), t(K::NewLibrary));
        self.library_section(ui, LibKind::Folder, t(K::FoldersHead), t(K::AddFolder));
    }

    fn library_section(&mut self, ui: &mut egui::Ui, kind: LibKind, title: &str, plus_tip: &str) {
        let pal = self.pal;
        let key = if kind == LibKind::Fcl { "libraries" } else { "folders" };
        let open = !self.folded.contains(key);
        let mut add = false;
        // Finder's section title, with "+" at the right: a new FontCraft library, or a folder.
        let clicked = crate::finder::section_header(ui, &pal, title, open, |ui| {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
            let hot = resp.hovered();
            if hot {
                ui.painter().rect_filled(r, 5.0, pal.hover);
            }
            let col = if hot { pal.accent } else { crate::finder::title_ink(&pal) };
            if !oslook::symbol(ui, "plus", r.center(), 11.0, col) {
                let c = r.center();
                let st = Stroke::new(1.6_f32, col);
                ui.painter().line_segment([c - Vec2::new(5.0, 0.0), c + Vec2::new(5.0, 0.0)], st);
                ui.painter().line_segment([c - Vec2::new(0.0, 5.0), c + Vec2::new(0.0, 5.0)], st);
            }
            if resp.on_hover_text(plus_tip).clicked() {
                add = true;
            }
        });
        if add {
            self.new_lib = Some(if kind == LibKind::Fcl { NewLib::library() } else { NewLib::new(String::new()) });
        } else if clicked {
            self.toggle_folded(key);
        }
        if !open {
            ui.add_space(2.0);
            return;
        }
        let mut menu_action: Option<crate::menus::MenuAction> = None;
        let mut native_menu: Option<(usize, Option<usize>)> = None;
        let indices: Vec<usize> = (0..self.libraries.len()).filter(|&i| self.libraries[i].kind == kind).collect();
        for i in indices {
            let view = View::Lib(i);
            let count = self.count(view);
            let sel = self.view == view;
            let has_sets = !self.libraries[i].sets.is_empty();
            let open = !self.lib_collapsed.contains(&i);
            let name = self.libraries[i].name.clone();
            // Finder's row: the fold arrow (when it has lists), the library's books or the folder,
            // the name shortened with "…" so it never runs into the count.
            let (resp, _, tri) = crate::finder::row(
                ui,
                &pal,
                crate::finder::Row {
                    icon: crate::finder::Icon::Own(if kind == LibKind::Fcl { "library" } else { "folder" }),
                    label: &name,
                    count: Some(count),
                    selected: sel,
                    indent: if has_sets { 0.0 } else { 14.0 },
                    disclosure: if has_sets { Some(open) } else { None },
                },
                |p, c, col| if kind == LibKind::Fcl { paint_library(p, c, col) } else { paint_folder(p, c, col) },
            );
            let tri = tri.unwrap_or(Rect::NOTHING);
            let path = self.libraries[i].path.display().to_string();
            let resp = resp.on_hover_text(path);
            if resp.clicked() {
                let pos = resp.interact_pointer_pos().unwrap_or_default();
                if has_sets && tri.expand(3.0).contains(pos) {
                    self.toggle_collapsed(i);
                } else {
                    self.view = view;
                }
            }
            // Right-click: the system's own menu where it offers one, else FontCraft's.
            if crate::nativemenu::available() {
                if resp.secondary_clicked() {
                    native_menu = Some((i, None));
                }
            } else {
                resp.context_menu(|ui| {
                    if let Some(a) = self.library_menu(ui, i, None) {
                        menu_action = Some(a);
                    }
                });
            }

            // The lists (subfolders; the letter folders of a FontCraft library), in name order.
            if has_sets && open {
                for s in 0..self.libraries[i].sets.len() {
                    let view = View::LibSet(i, s);
                    let count = self.count(view);
                    let sel = self.view == view;
                    // Nested lists sit further in, like their folders; long names end in "…" and
                    // the full name shows on hover.
                    let indent = 12.0 + (self.libraries[i].sets[s].depth.min(6) as f32) * 12.0;
                    let set_name = self.libraries[i].sets[s].name.clone();
                    let (resp, _, _) = crate::finder::row(
                        ui,
                        &pal,
                        crate::finder::Row {
                            icon: crate::finder::Icon::Own(if kind == LibKind::Fcl { "family" } else { "set" }),
                            label: &set_name,
                            count: Some(count),
                            selected: sel,
                            indent,
                            disclosure: None,
                        },
                        |p, c, col| {
                            if kind == LibKind::Fcl {
                                paint_family(p, c, col)
                            } else {
                                p.rect_stroke(Rect::from_center_size(c, Vec2::new(10.0, 8.0)), 1.5, Stroke::new(1.2_f32, col), eframe::egui::StrokeKind::Inside);
                            }
                        },
                    );
                    let resp = resp.on_hover_text(set_name);
                    if resp.clicked() {
                        self.view = view;
                    }
                    if crate::nativemenu::available() {
                        if resp.secondary_clicked() {
                            native_menu = Some((i, Some(s)));
                        }
                    } else {
                        resp.context_menu(|ui| {
                            if let Some(a) = self.library_menu(ui, i, Some(s)) {
                                menu_action = Some(a);
                            }
                        });
                    }
                }
            }
        }
        if let Some((lib, set)) = native_menu {
            let (items, acts) = self.library_menu_native(lib, set);
            self.open_native_menu(items, acts);
        }
        if let Some(a) = menu_action {
            self.run_menu_action(a);
        }
        ui.add_space(12.0);
    }

    /// The New Library window, in two forms: a new FontCraft library (a name, and fonts to put in
    /// it if wanted; saved as ~/FontCraft/<name>.fcl with its fonts in A–Z folders), or a folder
    /// from the computer (its subfolders become lists).
    pub(crate) fn new_library_window(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.folder_picker_rx {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            if let Ok(p_opt) = rx.try_recv() {
                if let Some(p) = p_opt {
                    if let Some(nl) = &mut self.new_lib {
                        nl.path = p.display().to_string();
                    }
                }
                self.folder_picker_rx = None;
            }
        }
        let Some(mut nl) = self.new_lib.take() else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        if let Some(p) = dropped.first() {
            // A folder dropped onto the window fills in the path.
            nl.path = p.display().to_string();
        }
        if nl.previewed_for != nl.path {
            nl.previewed_for = nl.path.clone();
            let p = Path::new(nl.path.trim());
            nl.preview = if !nl.path.trim().is_empty() && p.is_dir() { Some(libscan::preview(p)) } else { None };
        }
        let folder_buf = PathBuf::from(nl.path.trim());
        let folder = folder_buf.as_path();
        let exists = nl.preview.is_some();
        let fname = if exists { libscan::library_name(folder) } else { String::new() };
        // A library made earlier (a .fcl) can be chosen too: it is added as it is.
        let existing_fcl = nl.fcl && exists && folder.extension().is_some_and(|e| e.eq_ignore_ascii_case(fcl::EXT));
        let title = if nl.fcl { t(K::NewLibrary) } else { t(K::NewFolder) };
        let mut create = false;
        let mut cancel = false;
        let key = if nl.fcl { "new-fcl" } else { "add-folder" };
        let (_, closed) = oslook::dialog(ctx, key, title, 560.0, pal.panel, None, |ui| {
            {
                ui.label(egui::RichText::new(t(K::NewLibraryHint)).size(12.5).color(pal.muted));
                ui.add_space(10.0);
                egui::Grid::new("newlib").num_columns(2).spacing([14.0, 10.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new(if nl.fcl { t(K::FolderOrLibrary) } else { t(K::Folder) }).strong().color(pal.ink));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // The field takes the room the button leaves, so neither leaves the window.
                            let button_w = ui.spacing().button_padding.x * 2.0 + 14.0 + ui.painter().layout_no_wrap(t(K::ChooseBtn).to_string(), egui::FontId::proportional(oslook::metrics().text), egui::Color32::WHITE).size().x;
                            let field_w = (ui.available_width() - button_w - ui.spacing().item_spacing.x).max(80.0);
                            oslook::field(ui, egui::TextEdit::singleline(&mut nl.path).desired_width(field_w).hint_text("/Users/…/Fonts").margin(Margin::symmetric(8, 5)));
                            // The system's picker: a folder, or a FontCraft library (.fcl, a package on macOS).
                            #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
                            {
                                let is_picking = self.folder_picker_rx.is_some();
                                let btn_text = if is_picking { "…" } else { t(K::ChooseBtn) };
                                if ui.add_enabled(!is_picking, egui::Button::new(btn_text)).clicked() {
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    let start = exists.then_some(folder).map(|p| p.to_path_buf());
                                    crate::fcl::pick_folder_or_library_async(start, move |p| {
                                        let _ = tx.send(p);
                                    });
                                    self.folder_picker_rx = Some(rx);
                                }
                            }
                        });
                        if let Some(pv) = &nl.preview {
                            ui.label(egui::RichText::new(t(K::FontsFound).replace("{n}", &pv.total.to_string())).size(12.0).color(pal.accent));
                        } else if !nl.path.trim().is_empty() {
                            ui.label(egui::RichText::new(t(K::NoFolder)).size(12.0).color(pal.warn));
                        }
                    });
                    ui.end_row();
                    ui.label(egui::RichText::new(if nl.fcl { t(K::LibName) } else { t(K::FolderName) }).strong().color(pal.ink));
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            oslook::radio(ui, &mut nl.use_folder_name, true, t(K::UseChosenName));
                            if !fname.is_empty() {
                                ui.label(egui::RichText::new(format!("“{fname}”")).color(pal.muted));
                            }
                        });
                        ui.horizontal(|ui| {
                            oslook::radio(ui, &mut nl.use_folder_name, false, t(K::CustomName));
                            let edit = ui.add_enabled(
                                !nl.use_folder_name,
                                egui::TextEdit::singleline(&mut nl.custom).desired_width((ui.available_width() - 4.0).max(80.0)).margin(Margin::symmetric(8, 4)),
                            );
                            if edit.clicked() {
                                nl.use_folder_name = false;
                            }
                        });
                    });
                    ui.end_row();
                });

                // The subfolders (each becomes a list) with the fonts inside them.
                if let Some(pv) = &nl.preview {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(t(K::SubfoldersHead)).strong().color(pal.ink));
                    ui.add_space(4.0);
                    egui::Frame::NONE.fill(pal.card).corner_radius(8).inner_margin(Margin::same(8)).show(ui, |ui| {
                        egui::ScrollArea::vertical().id_salt("newlib-subs").max_height(230.0).auto_shrink([false, true]).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let list_fonts = |ui: &mut egui::Ui, fonts: &[String]| {
                                for f in fonts.iter().take(300) {
                                    ui.label(egui::RichText::new(f).size(12.5).color(pal.ink));
                                }
                                if fonts.len() > 300 {
                                    ui.label(egui::RichText::new(t(K::MoreN).replace("{n}", &(fonts.len() - 300).to_string())).size(12.0).color(pal.muted));
                                }
                            };
                            if pv.subs.is_empty() {
                                ui.label(egui::RichText::new(t(K::NoSubfolders)).size(12.5).color(pal.muted));
                            }
                            for (k, sub) in pv.subs.iter().enumerate() {
                                // Nested lists are indented under their parent list.
                                ui.horizontal(|ui| {
                                    ui.add_space(sub.depth.min(6) as f32 * 16.0);
                                    ui.vertical(|ui| {
                                        egui::CollapsingHeader::new(egui::RichText::new(format!("{}  ·  {}", sub.name, sub.fonts.len())).color(pal.ink))
                                            .id_salt(("sub", k))
                                            .default_open(pv.subs.len() <= 3)
                                            .show(ui, |ui| list_fonts(ui, &sub.fonts));
                                    });
                                });
                            }
                            if !pv.root_fonts.is_empty() {
                                egui::CollapsingHeader::new(egui::RichText::new(format!("{}  ·  {}", t(K::RootFonts), pv.root_fonts.len())).color(pal.muted))
                                    .id_salt("root-fonts")
                                    .default_open(pv.subs.is_empty())
                                    .show(ui, |ui| list_fonts(ui, &pv.root_fonts));
                            }
                        });
                    });
                }
            }

            ui.add_space(12.0);
            // One row high, buttons on the right (the window keeps the height of its content).
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), oslook::metrics().control_h + 4.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let name_ok = nl.use_folder_name || !nl.custom.trim().is_empty();
                let (label, ok) = if existing_fcl {
                    (t(K::AddBtn), !self.libraries.iter().any(|l| l.path == folder))
                } else if nl.fcl {
                    let name = if nl.use_folder_name { fname.clone() } else { nl.custom.trim().to_string() };
                    (t(K::CreateLibrary), exists && name_ok && !fcl::home_dir().join(format!("{name}.{}", fcl::EXT)).exists())
                } else {
                    (t(K::AddBtn), exists && (nl.use_folder_name || !nl.custom.trim().is_empty()))
                };
                if oslook::default_button(ui, ok, label).clicked() {
                    create = true;
                }
                if ui.button(t(K::Cancel)).clicked() {
                    cancel = true;
                }
            });
        });
        if create {
            if existing_fcl {
                // A FontCraft library made earlier: listed under LIBRARIES as it is.
                let name = if nl.use_folder_name { fname } else { nl.custom.trim().to_string() };
                let path = std::fs::canonicalize(folder).unwrap_or(folder.to_path_buf());
                self.libraries.push(library_from(name, path, LibKind::Fcl, true, &libscan::Preview::default()));
            } else if nl.fcl {
                // The library is a new ~/FontCraft/<name>.fcl; the folder's fonts are copied into it,
                // filed in A–Z folders.
                let name = if nl.use_folder_name { fname } else { nl.custom.trim().to_string() };
                match fcl::create(&name) {
                    Ok(path) => {
                        let n = fcl::add_fonts(&path, LibKind::Fcl, &[folder.to_path_buf()]);
                        let pv = libscan::preview(&path);
                        self.libraries.push(library_from(name, path, LibKind::Fcl, true, &pv));
                        if n > 0 {
                            self.notice = Some(t(K::ImportedN).replace("{n}", &n.to_string()));
                        }
                    }
                    Err(e) => {
                        self.notice = Some(e);
                        return;
                    }
                }
            } else {
                let name = if nl.use_folder_name { fname } else { nl.custom.trim().to_string() };
                let path = std::fs::canonicalize(folder).unwrap_or(folder.to_path_buf());
                let pv = nl.preview.take().unwrap_or_default();
                self.libraries.push(library_from(name, path, LibKind::Folder, true, &pv));
            }
            self.save_libraries();
            self.rescan();
            self.view = View::Lib(self.libraries.len() - 1);
            return;
        }
        if !closed && !cancel {
            self.new_lib = Some(nl);
        }
    }
}

/// `text` shortened with "…" to fit `width` points in `font`.
pub(crate) fn fit_text(ui: &egui::Ui, text: &str, font: FontId, width: f32) -> String {
    let w = |s: &str| ui.painter().layout_no_wrap(s.to_string(), font.clone(), egui::Color32::WHITE).size().x;
    if w(text) <= width {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.chars().collect();
    while chars.len() > 1 {
        chars.pop();
        let s: String = chars.iter().collect::<String>() + "…";
        if w(&s) <= width {
            return s;
        }
    }
    "…".into()
}
