//! The grid view, as the system's Font Book (Catálogo tipográfico) shows its fonts: one tile per
//! family with "Aa" set large in it, the family's name and how many styles it has underneath. The
//! size slider makes the tiles larger or smaller. A click selects the family (its details show at
//! the right) and uses it in the app FontCraft works with; a double-click does too; the asterisk
//! turns it on or off, the star marks it as a favourite, and right-click opens the font menu.

use crate::i18n::{self, K};
use crate::{libs, menus, nativemenu, App, Row};
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};

/// Tile width at the default sample size (60 px), and its limits (estimated from Font Book).
const TILE_PER_PX: f32 = 2.6;
const TILE_MIN: f32 = 120.0;
const TILE_MAX: f32 = 320.0;
const GAP: f32 = 12.0;

impl App {
    pub(crate) fn grid_view(&mut self, ui: &mut egui::Ui) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let families: Vec<usize> = self.rows().into_iter().filter_map(|r| if let Row::Family(f) = r { Some(f) } else { None }).collect();
        if families.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new(t(K::NoMatch)).size(17.0).color(pal.muted));
            });
            return;
        }
        let ctx = ui.ctx().clone();
        let mut list_action: Option<menus::MenuAction> = None;
        let mut native_row: Option<Row> = None;
        let avail = ui.available_width() - 2.0 * GAP;
        let want = (self.size * TILE_PER_PX).clamp(TILE_MIN, TILE_MAX);
        let cols = ((avail + GAP) / (want + GAP)).floor().max(1.0) as usize;
        let tile_w = ((avail - GAP * (cols as f32 - 1.0)) / cols as f32).floor();
        let sample_h = (tile_w * 0.62).round();
        let tile_h = sample_h + 44.0;
        let rows_n = families.len().div_ceil(cols);
        let total = GAP + rows_n as f32 * (tile_h + GAP);
        egui::ScrollArea::vertical().id_salt("font-grid").auto_shrink(false).show_viewport(ui, |ui, viewport| {
            ui.set_min_height(total);
            let origin = ui.max_rect().min + Vec2::new(GAP, GAP);
            for (k, &family) in families.iter().enumerate() {
                let (r, c) = (k / cols, k % cols);
                let top = r as f32 * (tile_h + GAP);
                if top + tile_h < viewport.min.y - GAP || top > viewport.max.y {
                    continue;
                }
                let rect = Rect::from_min_size(origin + Vec2::new(c as f32 * (tile_w + GAP), top), Vec2::new(tile_w, tile_h));
                let row = Row::Family(family);
                let resp = ui.interact(rect, ui.scope_id().with(("tile", k)), Sense::click());
                let sel = self.selected == Some(row);
                let face = self.lib.families[family].default_face;
                let p = ui.painter().clone();
                // The sample's square: the card colour, the accent's tint when chosen.
                let sample = Rect::from_min_size(rect.min, Vec2::new(tile_w, sample_h));
                let fill = if sel { pal.accent_soft } else if resp.hovered() { pal.hover } else { pal.card };
                p.rect_filled(sample, CornerRadius::same(10), fill);
                if sel {
                    p.rect_stroke(sample, CornerRadius::same(10), Stroke::new(2.0, pal.accent), egui::StrokeKind::Inside);
                }
                // "Aa" in the font (its own sample text when the font has no Latin letters).
                let px = (sample_h * 0.56).round();
                let (mut tex, mut size, _, missing) = self.texture(&ctx, face, "Aa", px, tile_w - 16.0);
                if missing || tex.is_none() {
                    let own: String = self.preview_text(family).chars().filter(|c| !c.is_whitespace()).take(2).collect();
                    let alt = self.texture(&ctx, face, &own, px, tile_w - 16.0);
                    if alt.0.is_some() {
                        tex = alt.0;
                        size = alt.1;
                    }
                }
                if let Some(tex) = tex {
                    let k = ((tile_w - 16.0) / size.x.max(1.0)).min((sample_h - 16.0) / size.y.max(1.0)).min(1.0);
                    let shown = Rect::from_center_size(sample.center(), size * k);
                    p.image(tex.id(), shown, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), pal.ink);
                }
                // Asterisk (on/off) at the top left, the star at the top right when it's a favourite
                // or under the pointer.
                self.activation_control(ui, row, sample.left_top() + Vec2::new(14.0, 14.0));
                let name = self.lib.families[family].name.clone();
                let is_fav = self.favorites.contains(&name);
                let star = Rect::from_center_size(sample.right_top() + Vec2::new(-14.0, 14.0), Vec2::splat(libs::ICON + 4.0));
                if is_fav || resp.hovered() {
                    let hot = ui.rect_contains_pointer(star);
                    let col = if is_fav { libs::STAR_ON } else if hot { libs::STAR_ON.gamma_multiply(0.6) } else { libs::ASTERISK_GRAY };
                    libs::paint_star(&p, star.center(), is_fav, col);
                }
                // Name and number of styles below, centred.
                let name_font = FontId::proportional(12.5);
                let shown_name = crate::finder::fit(ui, &name, &name_font, tile_w - 6.0);
                p.text(Pos2::new(rect.center().x, sample.bottom() + 14.0), Align2::CENTER_CENTER, shown_name, name_font, pal.ink);
                let n = self.lib.families[family].faces.len();
                let styles = if n == 1 { t(K::Style1).to_string() } else { t(K::StylesN).replace("{n}", &n.to_string()) };
                p.text(Pos2::new(rect.center().x, sample.bottom() + 31.0), Align2::CENTER_CENTER, styles, FontId::proportional(11.0), pal.muted);

                let resp = resp.on_hover_text(&name);
                if resp.clicked() {
                    let pos = resp.interact_pointer_pos().unwrap_or_default();
                    if star.expand(2.0).contains(pos) {
                        if !self.favorites.remove(&name) {
                            self.favorites.insert(name.clone());
                        }
                        self.save_favorites();
                    } else {
                        self.selected = Some(row);
                        if let Some(app) = self.target_app() {
                            self.apply_to_app(app, face);
                        }
                    }
                }

                if nativemenu::available() {
                    if resp.secondary_clicked() {
                        self.selected = Some(row);
                        native_row = Some(row);
                    }
                } else {
                    resp.context_menu(|ui| {
                        self.selected = Some(row);
                        if let Some(a) = self.font_menu(ui, row) {
                            list_action = Some(a);
                        }
                    });
                }
            }
        });
        if let Some(row) = native_row {
            let (items, acts) = self.font_menu_native(row);
            self.open_native_menu(items, acts);
        }
        if let Some(a) = list_action {
            self.run_menu_action(a);
        }
        let _ = Color32::TRANSPARENT;
    }
}
