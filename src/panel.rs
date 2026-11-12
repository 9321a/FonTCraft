//! The font panel on the right: Preview, Sizes, Glyphs, Features and Info pages.

use crate::i18n::{self, K};
use crate::settings::Pal;
use crate::{ot, App, Row};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum DetailTab {
    Preview,
    Sizes,
    Glyphs,
    Features,
    Info,
}

/// Facts read once from the selected font.
pub struct FaceData {
    pub face: usize,
    pub info: ot::FontInfo,
    pub cmap: HashMap<u16, u32>,
    pub feats: Vec<([u8; 4], ot::FeatureDetail)>,
    pub sets: Vec<(u8, ot::FeatureDetail)>,
}

const SIZES: [f32; 19] = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 48.0, 60.0, 72.0, 96.0, 120.0, 144.0];
const PUNCTUATION: &str = ".,:;…!?¡¿'\"‘’“”«»()[]{}/\\|-–—_@#&%*+=<>$€£¥¢§©®™";

fn uv() -> Rect {
    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0))
}

fn paint_tex(ui: &egui::Ui, tex: &Option<TextureHandle>, rect: Rect, tint: Color32) {
    if let Some(t) = tex {
        ui.painter().with_clip_rect(ui.clip_rect()).image(t.id(), rect, uv(), tint);
    }
}

/// An on/off switch. Returns true when clicked.
fn switch(ui: &mut egui::Ui, on: bool, enabled: bool, pal: &Pal) -> bool {
    let (r, resp) = ui.allocate_exact_size(Vec2::new(34.0, 20.0), if enabled { Sense::click() } else { Sense::hover() });
    let t = ui.ctx().animate_bool(resp.id, on);
    let off_bg = if pal.dark { Color32::from_rgb(0x48, 0x48, 0x50) } else { Color32::from_rgb(0xd4, 0xd4, 0xdc) };
    let mut bg = if on { pal.accent } else { off_bg };
    if !enabled {
        bg = bg.gamma_multiply(0.4);
    }
    ui.painter().rect_filled(r, 10.0, bg);
    let x = egui::lerp((r.left() + 10.0)..=(r.right() - 10.0), t);
    ui.painter().circle_filled(Pos2::new(x, r.center().y), 8.0, if enabled { Color32::WHITE } else { Color32::from_white_alpha(140) });
    enabled && resp.clicked()
}

fn chip(ui: &mut egui::Ui, text: &str, fill: Color32, ink: Color32) {
    let galley = ui.painter().layout_no_wrap(text.to_string(), FontId::monospace(12.0), ink);
    let (r, _) = ui.allocate_exact_size(galley.size() + Vec2::new(12.0, 6.0), Sense::hover());
    ui.painter().rect_filled(r, 5.0, fill);
    ui.painter().galley(r.min + Vec2::new(6.0, 3.0), galley, ink);
}

impl App {
    fn ensure_face_data(&mut self, face: usize) {
        if self.face_data.as_ref().map(|d| d.face) == Some(face) {
            return;
        }
        let f = &self.lib.faces[face];
        let file = &self.lib.files[f.file];
        let in_collection = ttf_parser::fonts_in_collection(&file.data).is_some();
        let Ok(tf) = ttf_parser::Face::parse(&file.data, f.index) else {
            self.face_data = None;
            return;
        };
        let feats = ot::MAIN_FEATURES.iter().map(|t| (**t, ot::feature_detail(&tf, t))).collect();
        let sets = ot::stylistic_sets(&tf).into_iter().map(|n| (n, ot::feature_detail(&tf, &ot::ss_tag(n)))).collect();
        self.face_data = Some(FaceData {
            face,
            info: ot::info(&tf, &file.path, in_collection),
            cmap: ot::reverse_cmap(&tf),
            feats,
            sets,
        });
        self.glyph_sel = tf.glyph_index('A').or_else(|| tf.glyph_index('a')).map(|g| g.0).or(Some(1));
    }

    fn feature_on(&self, tag: &[u8; 4]) -> bool {
        *self.features.get(tag).unwrap_or(&ot::on_by_default(tag))
    }

    pub(crate) fn details(&mut self, ui: &mut egui::Ui) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let Some(row) = self.selected else {
            ui.label(egui::RichText::new(t(K::SelectFont)).color(pal.muted));
            return;
        };
        let (family, face) = match row {
            Row::Family(f) => (f, self.lib.families[f].default_face),
            Row::Face(f, face) => (f, face),
        };
        self.ensure_face_data(face);

        ui.add_space(12.0);
        // The five buttons.
        let tabs = [
            (DetailTab::Preview, t(K::TabPreview)),
            (DetailTab::Sizes, t(K::TabSizes)),
            (DetailTab::Glyphs, t(K::TabGlyphs)),
            (DetailTab::Features, t(K::TabFeatures)),
            (DetailTab::Info, t(K::TabInfo)),
        ];
        let w = ui.available_width();
        let (bar, _) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::hover());
        ui.painter().rect_filled(bar, bar.height() * 0.5, pal.card);
        let seg = w / tabs.len() as f32;
        for (i, (tab, label)) in tabs.iter().enumerate() {
            let r = Rect::from_min_size(bar.min + Vec2::new(seg * i as f32, 0.0), Vec2::new(seg, 30.0)).shrink(3.0);
            let resp = ui.interact(r, ui.scope_id().with(("tab", i)), Sense::click());
            let on = self.detail_tab == *tab;
            // Pill-shaped, as RightFont's -[RFTabButton drawRect:] (0x1000be314) draws them: corner
            // radius = height × 0.5, the system accent behind the chosen one with white text.
            let pill = r.height() * 0.5;
            if on {
                ui.painter().rect_filled(r, pill, pal.accent);
            } else if resp.hovered() {
                ui.painter().rect_filled(r, pill, pal.hover);
            }
            let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::proportional(13.0), if on { Color32::WHITE } else { pal.ink });
            let scale = (r.width() - 6.0) / galley.size().x;
            let font = if scale < 1.0 { 13.0 * scale.max(0.7) } else { 13.0 };
            ui.painter().text(r.center(), Align2::CENTER_CENTER, *label, FontId::proportional(font), if on { Color32::WHITE } else { pal.ink });
            if resp.clicked() {
                self.detail_tab = *tab;
            }
        }
        ui.add_space(10.0);
        let fam = &self.lib.families[family];
        let fc = &self.lib.faces[face];
        ui.label(egui::RichText::new(&fam.name).size(18.0).strong().color(pal.ink));
        ui.label(egui::RichText::new(&fc.style).size(13.5).color(pal.muted));
        ui.add_space(4.0);
        ui.separator();

        match self.detail_tab {
            DetailTab::Preview => self.page_preview(ui, face, family),
            DetailTab::Sizes => self.page_sizes(ui, face, family),
            DetailTab::Glyphs => self.page_glyphs(ui, face),
            DetailTab::Features => self.page_features(ui, face),
            DetailTab::Info => self.page_info(ui, face, family),
        }
    }

    fn page_preview(&mut self, ui: &mut egui::Ui, face: usize, family: usize) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ctx = ui.ctx().clone();
        // Fonts without Latin letters show the characters they do have instead.
        let latin = self.face_data.as_ref().map(|d| d.cmap.values().any(|&c| c == 'A' as u32 || c == 'a' as u32)).unwrap_or(true);
        let mut blocks: Vec<(String, String)> = Vec::new();
        if latin {
            blocks.push((t(K::Uppercase).into(), "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into()));
            blocks.push((t(K::Lowercase).into(), "abcdefghijklmnopqrstuvwxyz".into()));
            blocks.push((t(K::Numbers).into(), "0123456789".into()));
            blocks.push((t(K::Punctuation).into(), PUNCTUATION.into()));
        } else if let Some(d) = &self.face_data {
            let mut cps: Vec<u32> = d.cmap.values().copied().filter(|&c| c > 0x20 && !(0x7f..0xa0).contains(&c)).collect();
            cps.sort_unstable();
            let s: String = cps.iter().take(160).filter_map(|&c| char::from_u32(c)).collect();
            blocks.push((t(K::TabGlyphs).into(), s));
        }
        blocks.push((t(K::Sentence).into(), self.preview_text(family)));
        egui::ScrollArea::vertical().id_salt("preview-page").auto_shrink(false).show(ui, |ui| {
            let w = ui.available_width() - 4.0;
            for (title, text) in blocks {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(title.to_uppercase()).size(11.5).color(pal.muted).strong());
                // Always 60 points here, whatever the list's size; lines never overlap (see fonts::draw_lines).
                let px = crate::mechanics::PANEL_PREVIEW_SIZE;
                let (tex, size, _, _) = self.texture_wrapped(&ctx, face, &text, px, w);
                let (r, _) = ui.allocate_exact_size(Vec2::new(w, size.y.max(px * 0.8)), Sense::hover());
                paint_tex(ui, &tex, Rect::from_min_size(r.min, size), pal.ink);
            }
            ui.add_space(12.0);
        });
    }

    fn page_sizes(&mut self, ui: &mut egui::Ui, face: usize, family: usize) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ctx = ui.ctx().clone();
        ui.label(egui::RichText::new(t(K::ClickSize)).size(12.0).color(pal.muted));
        let text = self.preview_text(family);
        egui::ScrollArea::vertical().id_salt("sizes-page").auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            let w = ui.available_width() - 4.0;
            for px in SIZES {
                let (tex, size, _, _) = self.texture(&ctx, face, &text, px, w - 12.0);
                let h = size.y.max(px) + 22.0;
                let (r, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
                let on = (self.size - px).abs() < 0.5;
                if on {
                    ui.painter().rect_filled(r, 6.0, pal.accent_soft);
                } else if resp.hovered() {
                    ui.painter().rect_filled(r, 6.0, pal.hover);
                }
                ui.painter().text(r.min + Vec2::new(6.0, 4.0), Align2::LEFT_TOP, format!("{px} px"), FontId::monospace(11.5), if on { pal.accent } else { pal.muted });
                if let Some(tex) = &tex {
                    let img = Rect::from_min_size(r.min + Vec2::new(6.0, 18.0), size);
                    ui.painter().with_clip_rect(r.intersect(ui.clip_rect())).image(tex.id(), img, uv(), pal.ink);
                }
                if resp.clicked() {
                    self.size = px;
                }
            }
        });
    }

    fn page_glyphs(&mut self, ui: &mut egui::Ui, face: usize) {
        // Lets the automatic checks open the glyph tools without a right-click:
        // FONTCRAFT_TEST_GLYPH=<glyph>[,info|mark|similar].
        if self.frame == 8 {
            if let Ok(v) = std::env::var("FONTCRAFT_TEST_GLYPH") {
                let mut parts = v.split(',');
                if let Some(gid) = parts.next().and_then(|g| g.parse::<u16>().ok()) {
                    self.glyph_sel = Some(gid);
                    match parts.next() {
                        Some("info") => self.char_info = Some((face, gid)),
                        Some("mark") => self.mark_char = self.glyph_char(gid),
                        Some("similar") => {
                            if let Some(c) = self.glyph_char(gid) {
                                self.similar = crate::glyphtools::start_similar(self, face, gid, c);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ctx = ui.ctx().clone();
        let n = self.lib.faces[face].glyphs as usize;
        let w = ui.available_width();

        // The chosen glyph, large, with its details.
        let sel = self.glyph_sel.unwrap_or(0);
        let (card, _) = ui.allocate_exact_size(Vec2::new(w, 130.0), Sense::hover());
        ui.painter().rect_filled(card, 10.0, pal.card);
        let (tex, size, _, _) = self.texture_glyph(&ctx, face, sel, 96.0);
        let big = Rect::from_min_size(card.min, Vec2::new(150.0, 130.0));
        paint_tex(ui, &tex, Rect::from_center_size(big.center(), size), pal.ink);
        let (name, cp) = {
            let f = &self.lib.faces[face];
            let data = &self.lib.files[f.file].data;
            let name = ttf_parser::Face::parse(data, f.index)
                .ok()
                .and_then(|tf| tf.glyph_name(ttf_parser::GlyphId(sel)).map(String::from))
                .unwrap_or_else(|| "—".into());
            let cp = self.face_data.as_ref().and_then(|d| d.cmap.get(&sel).copied());
            (name, cp)
        };
        let x = card.left() + 158.0;
        let p = ui.painter();
        p.text(Pos2::new(x, card.top() + 22.0), Align2::LEFT_CENTER, format!("#{sel}"), FontId::monospace(17.0), pal.ink);
        p.text(Pos2::new(x, card.top() + 50.0), Align2::LEFT_CENTER, t(K::GlyphName), FontId::proportional(11.5), pal.muted);
        p.text(Pos2::new(x, card.top() + 66.0), Align2::LEFT_CENTER, &name, FontId::monospace(13.0), pal.ink);
        p.text(Pos2::new(x, card.top() + 90.0), Align2::LEFT_CENTER, t(K::UnicodeLabel), FontId::proportional(11.5), pal.muted);
        let uni = match cp {
            Some(c) => format!("U+{c:04X}  {}", char::from_u32(c).unwrap_or(' ')),
            None => t(K::NotMapped).to_string(),
        };
        p.text(Pos2::new(x, card.top() + 106.0), Align2::LEFT_CENTER, uni, FontId::monospace(13.0), pal.ink);
        ui.add_space(6.0);
        ui.label(egui::RichText::new(t(K::GlyphsCount).replace("{n}", &n.to_string())).size(12.0).color(pal.muted));
        // How to put a glyph into the app FontCraft works with.
        let target = self.target_app();
        let hint = match target {
            Some(app) => t(K::DragGlyphHint).replace("{app}", self.autoact.apps[app].def_name()),
            None => t(K::NoTargetApp).to_string(),
        };
        ui.add(egui::Label::new(egui::RichText::new(hint).size(11.5).color(pal.muted)).wrap());
        // RightFont's glyph size buttons: ±2 pt between 56 and 128 (0x1000b7388 / 0x1000b7300).
        ui.horizontal(|ui| {
            use crate::mechanics::{GLYPH_MAX, GLYPH_MIN, GLYPH_STEP};
            if ui.add_enabled(self.glyph_px >= GLYPH_MIN + GLYPH_STEP, egui::Button::new("−")).clicked() {
                self.glyph_px -= GLYPH_STEP;
            }
            ui.label(egui::RichText::new(format!("{} pt", self.glyph_px)).monospace().size(12.0).color(pal.muted));
            if ui.add_enabled(self.glyph_px <= GLYPH_MAX - GLYPH_STEP, egui::Button::new("+")).clicked() {
                self.glyph_px += GLYPH_STEP;
            }
        });
        ui.add_space(4.0);

        // RightFont's glyph filter (0x1000b7050): the character typed ("SELF.character == %@") and/or
        // a kind of character ("SELF.unicodeCategory == %@"). Also accepts U+0041 or 0041.
        ui.horizontal(|ui| {
            crate::oslook::field(ui, egui::TextEdit::singleline(&mut self.glyph_find).hint_text("A · U+0041").desired_width(110.0).margin(egui::Margin::symmetric(8, 4)));
            let kinds = [t(K::CatAll), t(K::Uppercase), t(K::Lowercase), t(K::Numbers), t(K::Punctuation), t(K::SymbolList), t(K::NotMapped)];
            crate::oslook::combo(("glyph-kind", face), kinds[self.glyph_kind.min(kinds.len() - 1)]).width(150.0).show_ui(ui, |ui| {
                for (i, k) in kinds.iter().enumerate() {
                    ui.selectable_value(&mut self.glyph_kind, i, *k);
                }
            });
        });
        let wanted: Option<u32> = {
            let q = self.glyph_find.trim();
            let hex = q.strip_prefix("U+").or_else(|| q.strip_prefix("u+")).unwrap_or(q);
            if q.is_empty() {
                None
            } else if q.chars().count() == 1 {
                q.chars().next().map(|c| c as u32)
            } else if hex.len() >= 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                u32::from_str_radix(hex, 16).ok()
            } else {
                q.chars().next().map(|c| c as u32)
            }
        };
        let kind = self.glyph_kind;
        let shown: Vec<u16> = {
            let cmap = self.face_data.as_ref().map(|d| &d.cmap);
            (0..n as u16)
                .filter(|g| {
                    let cp = cmap.and_then(|m| m.get(g).copied());
                    if let Some(w) = wanted {
                        if cp != Some(w) {
                            return false;
                        }
                    }
                    let ch = cp.and_then(char::from_u32);
                    match kind {
                        1 => ch.is_some_and(|c| c.is_uppercase()),
                        2 => ch.is_some_and(|c| c.is_lowercase()),
                        3 => ch.is_some_and(|c| c.is_numeric()),
                        4 => ch.is_some_and(|c| c.is_ascii_punctuation() || matches!(c as u32, 0xA1..=0xBF | 0x2010..=0x205E | 0x3000..=0x303F) && !c.is_alphanumeric()),
                        5 => ch.is_some_and(|c| !c.is_alphanumeric() && !c.is_whitespace() && !c.is_ascii_punctuation() && !matches!(c as u32, 0xA1..=0xBF | 0x2010..=0x205E | 0x300..=0x36F)),
                        6 => ch.is_none(),
                        _ => true,
                    }
                })
                .collect()
        };
        ui.add_space(4.0);

        // Glyphs at 72 points to start, with no box drawn around each one; each fits inside its own cell.
        let glyph_px = self.glyph_px;
        let cell = glyph_px * 1.6;
        let cols = ((w - 4.0) / cell).floor().max(1.0) as usize;
        let rows = shown.len().div_ceil(cols);
        egui::ScrollArea::vertical().id_salt(("glyph-grid", face)).auto_shrink(false).show_rows(ui, cell, rows, |ui, range| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for r in range {
                ui.horizontal(|ui| {
                    for c in 0..cols {
                        let Some(&gid) = shown.get(r * cols + c) else { break };
                        // On Android a finger drag scrolls the grid (glyphs aren't dragged out to other apps there).
                        let sense = if cfg!(target_os = "android") { Sense::click() } else { Sense::click_and_drag() };
                        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(cell), sense);
                        let on = gid == sel;
                        // As RightFont's -[RFFontGlyphView updateLayer] (0x1000670f4): corner radius 8,
                        // the accent at 8% for the chosen glyph, the text colour at 2.5% under the pointer.
                        if on {
                            ui.painter().rect_filled(rect.shrink(2.0), crate::metrics::CELL_RADIUS, pal.accent.gamma_multiply(crate::metrics::SELECTED_TINT));
                        } else if resp.hovered() {
                            ui.painter().rect_filled(rect.shrink(2.0), crate::metrics::CELL_RADIUS, pal.ink.gamma_multiply(crate::metrics::HOVER_TINT));
                        }
                        let (tex, size, _, _) = self.texture_glyph(&ctx, face, gid, glyph_px);
                        // Wide or tall glyphs are scaled down to stay inside their cell (never on top of a neighbour).
                        let room = Vec2::new(cell - 8.0, cell - 20.0);
                        let k = (room.x / size.x.max(1.0)).min(room.y / size.y.max(1.0)).min(1.0);
                        let shown = Rect::from_center_size(rect.center() - Vec2::new(0.0, 6.0), size * k);
                        paint_tex(ui, &tex, shown, if on { pal.accent } else { pal.ink });
                        ui.painter().text(rect.center_bottom() - Vec2::new(0.0, 7.0), Align2::CENTER_CENTER, gid.to_string(), FontId::proportional(10.0), pal.muted);
                        if resp.clicked() {
                            self.glyph_sel = Some(gid);
                        }
                        // Double-click: insert into the target app. Drag: drop it on the app's icon.
                        // Double-click: into the app FontCraft works with; with none, copied, as RightFont
                        // does (its double-click runs copy:, 0x1000b6504 → 0x1000b7208).
                        if resp.double_clicked() {
                            if let Some(app) = target {
                                self.insert_glyph(app, face, gid);
                            } else if let Some(c) = self.glyph_char(gid) {
                                self.notice = Some(t(K::Copied).replace("{v}", &c.to_string()));
                                self.clip = Some(c.to_string());
                            }
                        }
                        if resp.drag_started() {
                            self.glyph_sel = Some(gid);
                            self.autoact.dragging = Some((gid, face));
                        }
                        // Pointer resting on a glyph: a 144-point preview with the font's lines.
                        let resp = resp.on_hover_ui(|ui| {
                            let (r, _) = ui.allocate_exact_size(Vec2::new(300.0, 300.0), Sense::hover());
                            self.paint_glyph_metrics(ui, r, face, gid, crate::metrics::PREVIEW_PT, false);
                        });
                        // Right-click: character info, copy, mark fonts, similar shapes.
                        resp.context_menu(|ui| {
                            self.glyph_sel = Some(gid);
                            self.glyph_menu(ui, face, gid);
                        });
                    }
                });
            }
        });
    }

    /// The examples of one feature: "these glyphs → those glyphs".
    fn feature_examples(&mut self, ui: &mut egui::Ui, face: usize, d: &ot::FeatureDetail) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ctx = ui.ctx().clone();
        if !d.present {
            ui.label(egui::RichText::new(t(K::NotInFont)).size(12.5).color(pal.warn));
            return;
        }
        if d.swaps.is_empty() && d.contextual > 0 {
            ui.label(egui::RichText::new(t(K::ContextualOnly)).size(12.0).color(pal.muted));
            return;
        }
        let mut shown = 0;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
            for s in &d.swaps {
                if shown >= 60 {
                    break;
                }
                let px = crate::mechanics::SAMPLE_PREVIEW_SIZE;
                let (a, asz, _, _) = self.texture_ids(&ctx, face, &s.from, px);
                let (b, bsz, _, _) = self.texture_ids(&ctx, face, &s.to, px);
                // Skip swaps between invisible glyphs (spaces and the like): nothing to see.
                if a.is_none() || b.is_none() {
                    continue;
                }
                shown += 1;
                let h = asz.y.max(bsz.y).max(24.0) + 8.0;
                let wbox = asz.x + bsz.x + 34.0;
                let (r, _) = ui.allocate_exact_size(Vec2::new(wbox.min(ui.available_width().max(60.0)), h), Sense::hover());
                ui.painter().rect(r, 5.0, pal.card, Stroke::new(1.0_f32, pal.line), eframe::egui::StrokeKind::Inside);
                let ra = Rect::from_min_size(Pos2::new(r.left() + 6.0, r.center().y - asz.y / 2.0), asz);
                paint_tex(ui, &a, ra, pal.muted);
                ui.painter().text(Pos2::new(ra.right() + 11.0, r.center().y), Align2::CENTER_CENTER, "→", FontId::proportional(13.0), pal.accent);
                let rb = Rect::from_min_size(Pos2::new(ra.right() + 22.0, r.center().y - bsz.y / 2.0), bsz);
                paint_tex(ui, &b, rb, pal.ink);
            }
        });
        if d.total > shown {
            ui.label(egui::RichText::new(t(K::MoreN).replace("{n}", &(d.total - shown).to_string())).size(12.0).color(pal.muted));
        }
    }

    /// One drop-down row: arrow, name, tag, count and switch. Returns (open, switch clicked).
    #[allow(clippy::too_many_arguments)]
    fn feature_row(&mut self, ui: &mut egui::Ui, key: &str, name: &str, tag: &str, sub: &str, on: bool, enabled: bool, indent: f32) -> (bool, bool) {
        let pal = self.pal;
        let open = self.feature_open.contains(key);
        let w = ui.available_width();
        let (r, resp) = ui.allocate_exact_size(Vec2::new(w, 42.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(r, 6.0, pal.hover);
        }
        let c = Pos2::new(r.left() + indent + 10.0, r.top() + 15.0);
        crate::oslook::disclosure(ui, c, open, pal.muted);
        let tx = r.left() + indent + 22.0;
        let name_r = ui.painter().text(Pos2::new(tx, r.top() + 15.0), Align2::LEFT_CENTER, name, FontId::proportional(14.0), if enabled { pal.ink } else { pal.muted });
        let tag_r = ui.painter().text(Pos2::new(name_r.right() + 8.0, r.top() + 15.0), Align2::LEFT_CENTER, tag, FontId::monospace(11.5), pal.accent);
        let _ = tag_r;
        ui.painter().text(Pos2::new(tx, r.top() + 31.0), Align2::LEFT_CENTER, sub, FontId::proportional(11.5), if enabled { pal.muted } else { pal.warn });
        // The switch sits on top of the row, so its click wins.
        let sw_rect = Rect::from_min_size(Pos2::new(r.right() - 42.0, r.center().y - 10.0), Vec2::new(34.0, 20.0));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(sw_rect));
        let flipped = switch(&mut child, on, enabled, &pal);
        if resp.clicked() && !flipped {
            if open {
                self.feature_open.remove(key);
            } else {
                self.feature_open.insert(key.to_string());
            }
        }
        (self.feature_open.contains(key), flipped)
    }

    fn page_features(&mut self, ui: &mut egui::Ui, face: usize) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let Some(data) = self.face_data.as_ref() else { return };
        let feats = data.feats.clone();
        let sets = data.sets.clone();
        ui.label(egui::RichText::new(t(K::FeaturesNote)).size(12.0).color(pal.muted));
        egui::ScrollArea::vertical().id_salt("features-page").auto_shrink(false).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            for (tag, d) in &feats {
                let (name, desc) = match tag {
                    b"aalt" => (K::FeatAalt, K::DescAalt),
                    b"case" => (K::FeatCase, K::DescCase),
                    b"ccmp" => (K::FeatCcmp, K::DescCcmp),
                    b"liga" => (K::FeatLiga, K::DescLiga),
                    b"locl" => (K::FeatLocl, K::DescLocl),
                    _ => (K::FeatMedi, K::DescMedi),
                };
                let tag_s = String::from_utf8_lossy(tag).to_string();
                let sub = if !d.present {
                    t(K::NotInFont).to_string()
                } else if d.total == 0 {
                    t(K::RulesN).replace("{n}", &d.contextual.to_string())
                } else {
                    t(K::RulesN).replace("{n}", &d.total.to_string())
                };
                let on = self.feature_on(tag) && d.present;
                let (open, flipped) = self.feature_row(ui, &tag_s, t(name), &tag_s, &sub, on, d.present, 0.0);
                if flipped {
                    let cur = self.feature_on(tag);
                    self.features.insert(*tag, !cur);
                }
                if open {
                    ui.indent(("feat", &tag_s), |ui| {
                        ui.label(egui::RichText::new(t(desc)).size(12.5).color(pal.ink));
                        ui.add_space(4.0);
                        self.feature_examples(ui, face, d);
                        ui.add_space(8.0);
                    });
                }
                ui.separator();
            }

            // Stylistic Sets: one switch for all, then each set with its own switch.
            let any_on = sets.iter().any(|(n, _)| self.feature_on(&ot::ss_tag(*n)));
            let range = match (sets.first(), sets.last()) {
                (Some((a, _)), Some((b, _))) if a != b => format!("ss{a:02}–ss{b:02}"),
                (Some((a, _)), _) => format!("ss{a:02}"),
                _ => "ss01–ss20".into(),
            };
            let sub = if sets.is_empty() { t(K::NoSets).to_string() } else { t(K::SetsN).replace("{n}", &sets.len().to_string()) };
            let (open, flipped) = self.feature_row(ui, "ss", t(K::FeatSs), &range, &sub, any_on, !sets.is_empty(), 0.0);
            if flipped {
                for (n, _) in &sets {
                    self.features.insert(ot::ss_tag(*n), !any_on);
                }
            }
            if open {
                ui.indent("sets", |ui| {
                    ui.label(egui::RichText::new(t(K::DescSs)).size(12.5).color(pal.ink));
                    ui.add_space(4.0);
                    if sets.is_empty() {
                        ui.label(egui::RichText::new(t(K::NoSets)).size(12.5).color(pal.warn));
                    }
                    for (n, d) in &sets {
                        let tag = ot::ss_tag(*n);
                        let tag_s = String::from_utf8_lossy(&tag).to_string();
                        let label = t(K::SetN).replace("{n}", &format!("{n:02}"));
                        let sub = t(K::RulesN).replace("{n}", &d.total.max(d.contextual).to_string());
                        let on = self.feature_on(&tag);
                        let (open, flipped) = self.feature_row(ui, &tag_s, &label, &tag_s, &sub, on, true, 0.0);
                        if flipped {
                            self.features.insert(tag, !on);
                        }
                        if open {
                            ui.indent(("set", *n), |ui| {
                                self.feature_examples(ui, face, d);
                                ui.add_space(6.0);
                            });
                        }
                    }
                });
            }
            ui.add_space(12.0);
        });
    }

    fn page_info(&mut self, ui: &mut egui::Ui, face: usize, family: usize) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let Some(d) = self.face_data.as_ref() else { return };
        let info = &d.info;
        let fam = &self.lib.families[family];
        let file = &self.lib.files[self.lib.faces[face].file];
        let or_none = |s: &str| if s.trim().is_empty() { t(K::NoneText).to_string() } else { s.to_string() };

        let mut sets = vec![t(K::AllFonts).to_string()];
        match file.source {
            crate::fonts::Source::System => sets.push(t(K::System).to_string()),
            crate::fonts::Source::Computer => sets.push(t(K::Computer).to_string()),
            crate::fonts::Source::User => sets.push(t(K::User).to_string()),
            crate::fonts::Source::Google => sets.push(t(K::GoogleFonts).to_string()),
            crate::fonts::Source::Adobe => sets.push(t(K::AdobeFonts).to_string()),
            crate::fonts::Source::Library => {}
        }
        for &l in &fam.libs {
            if let Some(lib) = self.libraries.get(l) {
                sets.push(lib.name.clone());
            }
        }
        sets.push(
            t(match fam.class {
                crate::fonts::Class::Serif => K::SerifList,
                crate::fonts::Class::Sans => K::SansList,
                crate::fonts::Class::Slab => K::SlabList,
                crate::fonts::Class::Script => K::ScriptList,
                crate::fonts::Class::Blackletter => K::BlackletterList,
                crate::fonts::Class::Display => K::CatDisplay,
                crate::fonts::Class::Symbol => K::SymbolList,
                crate::fonts::Class::Other => K::NoneText,
            })
            .to_string(),
        );
        if fam.class == crate::fonts::Class::Other {
            sets.pop();
        }
        if self.favorites.contains(&fam.name) {
            sets.push(t(K::Favorites).to_string());
        }
        if fam.monospaced {
            sets.push(t(K::Monospaced).to_string());
        }
        let axes = if info.axes.is_empty() {
            t(K::NoneText).to_string()
        } else {
            info.axes
                .iter()
                .map(|(tag, lo, def, hi)| format!("{tag}  {lo} – {hi}  ({})", t(K::DefaultN).replace("{n}", &def.to_string())))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let rows: Vec<(&str, String)> = vec![
            (t(K::Name), or_none(&info.full_name)),
            (t(K::FamilyLabel), or_none(&info.family)),
            (t(K::StyleLabel), or_none(&info.style)),
            (t(K::Format), info.format.clone()),
            (t(K::PsName), or_none(&info.ps_name)),
            (t(K::UniqueName), or_none(&info.unique)),
            (t(K::Location), file.path.display().to_string()),
            (t(K::FontSets), sets.join(", ")),
            (t(K::FontVariables), axes),
            (t(K::Version), or_none(&info.version)),
            (t(K::GlyphCount), info.glyphs.to_string()),
        ];
        let mut descr: Vec<&str> = Vec::new();
        if !info.axes.is_empty() {
            descr.push(t(K::Variable));
        }
        if info.bold {
            descr.push(t(K::Bold));
        }
        if info.italic {
            descr.push(t(K::Italic));
        }
        if info.monospaced {
            descr.push(t(K::Monospaced));
        }
        let scripts = info.scripts.clone();
        let feats = info.features.clone();
        let languages = info.languages.clone();
        // Licence block: what the font's own name table and OS/2 embedding bits say.
        let approved = match info.fs_type & 0x000E {
            0x0002 => t(K::UseRestricted),
            0x0004 => t(K::UsePreview),
            0x0008 => t(K::UseEditable),
            _ if info.fs_type & 0x0002 != 0 => t(K::UseRestricted),
            _ => t(K::UseInstallable),
        };
        let mut license = or_none(&info.license);
        if !info.license_url.trim().is_empty() {
            license = if info.license.trim().is_empty() { info.license_url.clone() } else { format!("{}\n{}", info.license, info.license_url) };
        }
        let license_rows: Vec<(&str, String)> = vec![
            (t(K::License), license),
            (t(K::ApprovedUse), approved.to_string()),
            (t(K::Rights), or_none(&info.copyright)),
            (t(K::Trademark), or_none(&info.trademark)),
            (t(K::Designer), or_none(&info.designer)),
            (t(K::Manufacturer), or_none(&info.manufacturer)),
        ];

        egui::ScrollArea::vertical().id_salt("info-page").auto_shrink(false).show(ui, |ui| {
            ui.add_space(4.0);
            let label_w = 118.0;
            for (k, v) in rows {
                ui.horizontal_top(|ui| {
                    ui.add_sized([label_w, 16.0], egui::Label::new(egui::RichText::new(k).size(12.5).color(pal.muted)).wrap());
                    ui.add(egui::Label::new(egui::RichText::new(v).size(12.5).color(pal.ink)).wrap());
                });
                ui.add_space(3.0);
            }
            ui.add_space(4.0);
            ui.label(egui::RichText::new(t(K::Tags)).size(12.5).color(pal.muted));
            {
                {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                        for d in &descr {
                            chip(ui, d, pal.accent_soft, pal.accent);
                        }
                        for s in &scripts {
                            chip(ui, s, pal.card, pal.ink);
                        }
                        for f in &feats {
                            chip(ui, f, pal.card, pal.muted);
                        }
                        if descr.is_empty() && scripts.is_empty() && feats.is_empty() {
                            ui.label(egui::RichText::new(t(K::NoneText)).size(12.5).color(pal.ink));
                        }
                    });
                }
            }

            // Supported languages (each in its own name), after a separating line.
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("{}  ·  {}", t(K::SupportedLanguages), languages.len())).size(12.5).color(pal.muted));
            ui.add_space(2.0);
            if languages.is_empty() {
                ui.label(egui::RichText::new(t(K::NoneText)).size(12.5).color(pal.ink));
            } else {
                ui.add(egui::Label::new(egui::RichText::new(languages.join(" · ")).size(12.5).color(pal.ink)).wrap());
            }

            // Licence, approved use, rights, trademark, designer and manufacturer, after another line.
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            for (k, v) in license_rows {
                ui.horizontal_top(|ui| {
                    ui.add_sized([label_w, 16.0], egui::Label::new(egui::RichText::new(k).size(12.5).color(pal.muted)).wrap());
                    ui.add(egui::Label::new(egui::RichText::new(v).size(12.5).color(pal.ink)).wrap());
                });
                ui.add_space(3.0);
            }
            ui.add_space(12.0);
        });
    }
}
