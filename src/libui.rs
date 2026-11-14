//! The sidebar's "Libraries" and "Folders", and the New Library / Add Folder window.

use crate::i18n::{self, K};
use crate::libs::{library_from, paint_family, paint_folder, paint_library, LibKind, NewLib};
use crate::{fcl, libscan, oslook, App, View};
use eframe::egui::{self, Align2, FontId, Margin, Rect, Sense, Stroke, Vec2};
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
        // The section's header, with the system's disclosure arrow and "+".
        let clicked = oslook::section_header(ui, title, open, pal.muted, |ui| {
            {
                ui.add_space(6.0);
                // + : a new FontCraft library, or a folder from the computer.
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, 5.0, pal.hover);
                }
                let c = r.center();
                let st = Stroke::new(1.6_f32, if resp.hovered() { pal.accent } else { pal.muted });
                ui.painter().line_segment([c - Vec2::new(5.0, 0.0), c + Vec2::new(5.0, 0.0)], st);
                ui.painter().line_segment([c - Vec2::new(0.0, 5.0), c + Vec2::new(0.0, 5.0)], st);
                if resp.on_hover_text(plus_tip).clicked() {
                    add = true;
                }
            }
        });
        if add {
            self.new_lib = Some(if kind == LibKind::Fcl { NewLib::library() } else { NewLib::new(String::new()) });
        } else if clicked {
            self.toggle_folded(key);
        }
        if !open {
            ui.add_space(4.0);
            return;
        }
        let mut menu_action: Option<crate::menus::MenuAction> = None;
        let mut native_menu: Option<(usize, Option<usize>)> = None;
        let indices: Vec<usize> = (0..self.libraries.len()).filter(|&i| self.libraries[i].kind == kind).collect();
        for i in indices {
            let view = View::Lib(i);
            let count = self.count(view);
            let w = ui.available_width();
            let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 26.0), Sense::click());
            let r = r.shrink2(Vec2::new(4.0, 0.0));
            let sel = self.view == view;
            if let Some(c) = crate::settings::row_tint(pal.dark, sel, resp.hovered()) {
                ui.painter().rect_filled(r, 6.0, c);
            }
            let has_sets = !self.libraries[i].sets.is_empty();
            let open = !self.lib_collapsed.contains(&i);
            // The system's disclosure arrow on the left (the same one as the section headers), then
            // the library's or folder's icon and the name, each with room of its own.
            let tri = Rect::from_center_size(r.left_center() + Vec2::new(9.0, 0.0), Vec2::splat(16.0));
            if has_sets {
                oslook::disclosure(ui, tri.center(), open, pal.muted);
            }
            // A library has its own icon (books); a folder keeps the folder icon.
            if kind == LibKind::Fcl {
                paint_library(ui.painter(), r.left_center() + Vec2::new(27.0, 0.0), if sel { pal.accent } else { pal.muted });
            } else {
                paint_folder(ui.painter(), r.left_center() + Vec2::new(27.0, 0.0), if sel { pal.accent } else { pal.muted });
            }
            // The count first, then the name shortened with "…" so it never runs into it.
            let count_rect = ui.painter().text(r.right_center() - Vec2::new(10.0, 0.0), Align2::RIGHT_CENTER, count.to_string(), FontId::proportional(12.5), pal.muted);
            let room = (count_rect.left() - 8.0 - (r.left() + 42.0)).max(20.0);
            let name = fit_text(ui, &self.libraries[i].name, FontId::proportional(14.0), room);
            ui.painter().text(r.left_center() + Vec2::new(42.0, 0.0), Align2::LEFT_CENTER, name, FontId::proportional(14.0), if sel { pal.accent } else { pal.ink });
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
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click());
                    let r = r.shrink2(Vec2::new(4.0, 0.0));
                    let sel = self.view == view;
                    if let Some(c) = crate::settings::row_tint(pal.dark, sel, resp.hovered()) {
                        ui.painter().rect_filled(r, 6.0, c);
                    }
                    let col = if sel { pal.accent } else { pal.muted };
                    // Nested lists sit further in, like their folders.
                    let indent = (self.libraries[i].sets[s].depth.min(6) as f32) * 12.0;
                    let c = r.left_center() + Vec2::new(42.0 + indent, 0.0);
                    if kind == LibKind::Fcl {
                        paint_family(ui.painter(), c, col);
                    } else {
                        ui.painter().rect_stroke(Rect::from_center_size(c, Vec2::new(10.0, 8.0)), 1.5, Stroke::new(1.2_f32, col), eframe::egui::StrokeKind::Inside);
                    }
                    let set_name = self.libraries[i].sets[s].name.clone();
                    // Long names are shortened with "…" to fit; the full name shows on hover.
                    let max_chars = ((r.width() - 80.0 - indent) / 6.5).max(6.0) as usize;
                    let label = if set_name.chars().count() > max_chars {
                        let mut short: String = set_name.chars().take(max_chars - 1).collect();
                        short.push('…');
                        short
                    } else {
                        set_name.clone()
                    };
                    ui.painter().text(r.left_center() + Vec2::new(54.0 + indent, 0.0), Align2::LEFT_CENTER, label, FontId::proportional(13.0), if sel { pal.accent } else { pal.ink });
                    ui.painter().text(r.right_center() - Vec2::new(10.0, 0.0), Align2::RIGHT_CENTER, count.to_string(), FontId::proportional(12.0), pal.muted);
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
                            let button_w = ui.spacing().button_padding.x * 2.0 + 14.0 + ui.fonts_mut(|f| f.layout_no_wrap(t(K::ChooseBtn).to_string(), egui::FontId::proportional(oslook::metrics().text), egui::Color32::WHITE).size().x);
                            let field_w = (ui.available_width() - button_w - ui.spacing().item_spacing.x).max(80.0);
                            oslook::field(ui, egui::TextEdit::singleline(&mut nl.path).desired_width(field_w).hint_text("/Users/…/Fonts").margin(Margin::symmetric(8, 5)));
                            // The system's picker: a folder, or a FontCraft library (.fcl, a package on macOS).
                            #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
                            if ui.button(t(K::ChooseBtn)).clicked() {
                                if let Some(p) = crate::fcl::pick_folder_or_library(exists.then_some(folder)) {
                                    nl.path = p.display().to_string();
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
