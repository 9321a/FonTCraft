//! The libraries and folders as a menu, the way the sidebar shows them: each library (books icon)
//! or folder (folder icon) with its whole tree of lists below it, indented and with their icons
//! (families in a library, subfolders in a folder). The menu grows to most of the screen's height
//! and scrolls only when the tree is taller than that, so nothing is cut short.

use crate::libs::{paint_family, paint_folder, paint_library, LibKind, UserLibrary as Library};
use crate::settings::Pal;
use eframe::egui::{self, Align2, FontId, Rect, Sense, Stroke, Vec2};
use std::path::PathBuf;

/// Where something goes: a library or folder (index) and, for a folder, one of its subfolders.
pub type Place = (usize, Option<PathBuf>);

/// One row: an icon, the name (shortened with "…" to fit) and the chosen mark.
fn row(ui: &mut egui::Ui, pal: &Pal, indent: f32, icon: &dyn Fn(&egui::Painter, egui::Pos2, egui::Color32), name: &str, bold: bool, chosen: bool) -> egui::Response {
    let w = ui.available_width().max(260.0);
    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 24.0), Sense::click());
    if let Some(c) = crate::settings::row_tint(pal.dark, chosen, resp.hovered()) {
        ui.painter().rect_filled(r, 5.0, c);
    }
    let col = if chosen { pal.accent } else if bold { pal.ink } else { pal.muted };
    icon(ui.painter(), r.left_center() + Vec2::new(14.0 + indent, 0.0), col);
    let font = FontId::proportional(if bold { 14.0 } else { 13.0 });
    let room = (r.width() - 40.0 - indent).max(40.0);
    let text = crate::libui::fit_text(ui, name, font.clone(), room);
    ui.painter().text(r.left_center() + Vec2::new(28.0 + indent, 0.0), Align2::LEFT_CENTER, text, font, if chosen { pal.accent } else { pal.ink });
    if chosen {
        let c = r.right_center() - Vec2::new(12.0, 0.0);
        ui.painter().line_segment([c + Vec2::new(-4.0, 0.0), c + Vec2::new(-1.0, 3.0)], Stroke::new(1.6_f32, pal.accent));
        ui.painter().line_segment([c + Vec2::new(-1.0, 3.0), c + Vec2::new(4.0, -3.5)], Stroke::new(1.6_f32, pal.accent));
    }
    resp.on_hover_text(name)
}

/// The tree of every library (`kind` = Fcl) or folder (`kind` = Folder) in `order`. Returns the
/// place chosen. In a FontCraft library the lists are its families (the library files fonts A–Z
/// itself), so choosing one of them points at the library; a folder's subfolders are real places.
pub fn tree(ui: &mut egui::Ui, pal: &Pal, libraries: &[Library], order: &[usize], kind: LibKind, chosen: Option<&Place>) -> Option<Place> {
    let mut out = None;
    let max_h = (ui.ctx().content_rect().height() * 0.75).max(240.0);
    ui.set_min_width(320.0);
    let any = order.iter().any(|&i| libraries[i].kind == kind);
    if !any {
        ui.label(egui::RichText::new("—").color(pal.muted));
        return None;
    }
    // Every row (a library, or one of its lists) in order; only the rows in sight are drawn, so a
    // library with thousands of lists opens as fast as a small one.
    let rows: Vec<(usize, Option<usize>)> = order
        .iter()
        .filter(|&&i| libraries[i].kind == kind)
        .flat_map(|&i| std::iter::once((i, None)).chain((0..libraries[i].sets.len()).map(move |s| (i, Some(s)))))
        .collect();
    egui::ScrollArea::vertical().id_salt(("tree-menu", kind == LibKind::Fcl)).max_height(max_h).show_rows(ui, 24.0, rows.len(), |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for &(i, set) in &rows[range] {
            let l = &libraries[i];
            let Some(si) = set else {
                let top_chosen = chosen.is_some_and(|c| c.0 == i && c.1.is_none());
                let icon: &dyn Fn(&egui::Painter, egui::Pos2, egui::Color32) = if kind == LibKind::Fcl { &paint_library } else { &paint_folder };
                if row(ui, pal, 0.0, icon, &l.name, true, top_chosen).clicked() {
                    out = Some((i, None));
                }
                continue;
            };
            {
                let s = &l.sets[si];
                let indent = 14.0 + (s.depth.min(8) as f32) * 12.0;
                let sub = if kind == LibKind::Folder { Some(s.rel.clone()) } else { None };
                let on = sub.is_some() && chosen.is_some_and(|c| c.0 == i && c.1 == sub);
                let sub_icon: &dyn Fn(&egui::Painter, egui::Pos2, egui::Color32) = if kind == LibKind::Fcl {
                    &paint_family
                } else {
                    &|p: &egui::Painter, c: egui::Pos2, col: egui::Color32| {
                        p.rect_stroke(Rect::from_center_size(c, Vec2::new(10.0, 8.0)), 1.5, Stroke::new(1.2_f32, col), egui::StrokeKind::Inside);
                    }
                };
                if row(ui, pal, indent, sub_icon, &s.name, false, on).clicked() {
                    out = Some((i, sub));
                }
            }
        }
    });
    if out.is_some() {
        ui.close();
    }
    out
}

/// Libraries first, then folders, each in alphabetical order (indexes into `libraries`).
pub fn order(libraries: &[Library]) -> Vec<usize> {
    let mut v: Vec<usize> = (0..libraries.len()).collect();
    v.sort_by_key(|&i| (libraries[i].kind != LibKind::Fcl, libraries[i].name.to_lowercase()));
    v
}
