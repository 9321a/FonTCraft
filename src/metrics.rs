//! A glyph drawn large, white on black, with the font's lines: baseline, x-height, cap height and
//! the glyph's width (fine solid lines), ascender and descender (dotted). Used by the Glyphs page
//! (hover preview, 144 pt) and the Character Info window (with the lines' names).

use crate::i18n::{self, K};
use crate::App;
use eframe::egui::{self, Align2, Color32, ColorImage, FontId, Pos2, Rect, Stroke, TextureHandle, TextureOptions, Vec2};

/// The glyph's picture and the lines, in pixels from the picture's top-left.
pub struct GlyphMetrics {
    pub tex: TextureHandle,
    /// Picture size in pixels.
    pub size: Vec2,
    /// Where the glyph's origin (start of the baseline) is.
    pub origin: Pos2,
    pub advance: f32,
    pub ascender: f32,
    pub descender: f32,
    pub x_height: f32,
    pub cap_height: f32,
}

/// The preview size the person asked for.
pub const PREVIEW_PT: f32 = 144.0;
/// The glyph preview's drawing (300 points square, as the tooltip it replaces) and the frame's
/// padding around it; the preview's right edge touches the details panel's left edge.
pub const PREVIEW_SIDE: f32 = 300.0;
pub const PREVIEW_PAD: f32 = 6.0;
/// The keys that type a character, in the glyph preview and in Character Info (the person's
/// latest sizes: 18 pt in both).
pub const KEYS_PT: f32 = 18.0;
/// Glyph cells, from RightFont's -[RFFontGlyphView updateLayer] (0x1000670f4): corner radius
/// (`fmov d0, #8.0`), the chosen glyph's tint (0.08 at 0x100282548) and the hover tint
/// (0.025 at 0x1002825d0).
pub const CELL_RADIUS: f32 = 8.0;
pub const SELECTED_TINT: f32 = 0.08;
pub const HOVER_TINT: f32 = 0.025;

fn render(ctx: &egui::Context, data: &[u8], index: u32, gid: u16, px: f32, key: &str) -> Option<GlyphMetrics> {
    use ab_glyph::{point, Font, FontRef, GlyphId, PxScale, ScaleFont};
    let font = FontRef::try_from_slice_and_index(data, index).ok()?;
    let tf = ttf_parser::Face::parse(data, index).ok()?;
    let scale = PxScale::from(px);
    let sf = font.as_scaled(scale);
    let (asc, desc) = (sf.ascent(), -sf.descent());
    let k = (asc + desc) / ((tf.ascender() as f32 - tf.descender() as f32).max(1.0));
    let xh = tf.x_height().map(|v| v as f32 * k).unwrap_or(asc * 0.5);
    let cap = tf.capital_height().map(|v| v as f32 * k).unwrap_or(asc * 0.7);
    let adv = sf.h_advance(GlyphId(gid));
    let outline = font.outline_glyph(GlyphId(gid).with_scale_and_position(scale, point(0.0, 0.0)));
    let (bx0, by0, bx1, by1) = outline.as_ref().map(|o| {
        let b = o.px_bounds();
        (b.min.x, b.min.y, b.max.x, b.max.y)
    }).unwrap_or((0.0, 0.0, 0.0, 0.0));
    let pad = px * 0.12;
    let left = bx0.min(0.0) - pad;
    let right = bx1.max(adv) + pad;
    let top = by0.min(-asc).min(-cap) - pad;
    let bottom = by1.max(desc) + pad;
    let (w, h) = ((right - left).ceil().max(1.0) as usize, (bottom - top).ceil().max(1.0) as usize);
    let (ox, oy) = (-left, -top);
    let mut img = ColorImage::filled([w, h], Color32::TRANSPARENT);
    if let Some(o) = outline {
        let b = o.px_bounds();
        o.draw(|gx, gy, c| {
            let x = (b.min.x + ox) as i32 + gx as i32;
            let y = (b.min.y + oy) as i32 + gy as i32;
            if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                img.pixels[y as usize * w + x as usize] = Color32::from_white_alpha((c.clamp(0.0, 1.0) * 255.0) as u8);
            }
        });
    }
    let tex = ctx.load_texture(key, img, TextureOptions::LINEAR);
    Some(GlyphMetrics {
        tex,
        size: Vec2::new(w as f32, h as f32),
        origin: Pos2::new(ox, oy),
        advance: adv,
        ascender: asc,
        descender: desc,
        x_height: xh,
        cap_height: cap,
    })
}

fn dashed(p: &egui::Painter, a: Pos2, b: Pos2, st: Stroke) {
    let len = (b - a).length();
    let dir = (b - a) / len.max(1.0);
    let mut d = 0.0;
    while d < len {
        let e = (d + 4.0).min(len);
        p.line_segment([a + dir * d, a + dir * e], st);
        d += 8.0;
    }
}

impl App {
    /// The metrics picture of a glyph (made once per face, glyph and size).
    fn glyph_metrics(&mut self, ctx: &egui::Context, face: usize, gid: u16, pt: f32) -> Option<std::rc::Rc<GlyphMetrics>> {
        let ppp = ctx.pixels_per_point();
        let key = (face, gid, (pt * ppp) as u32);
        if let Some(m) = self.metrics_cache.get(&key) {
            return Some(m.clone());
        }
        if self.metrics_cache.len() > 64 {
            self.metrics_cache.clear();
        }
        let f = &self.lib.faces[face];
        let m = std::rc::Rc::new(render(ctx, &self.lib.files[f.file].data, f.index, gid, pt * ppp, &format!("metrics-{face}-{gid}-{pt}"))?);
        self.metrics_cache.insert(key, m.clone());
        Some(m)
    }

    /// Draws the glyph with its lines in `area` (black background, the system's accent around it),
    /// as large as fits up to `pt` points. With `labels`, each line is named in capitals.
    pub(crate) fn paint_glyph_metrics(&mut self, ui: &egui::Ui, area: Rect, face: usize, gid: u16, pt: f32, labels: bool) {
        let ppp = ui.ctx().pixels_per_point();
        let p = ui.painter();
        let accent = ui.visuals().selection.bg_fill;
        p.rect_filled(area, 8.0, Color32::BLACK);
        p.rect_stroke(area, 8.0, Stroke::new(1.5_f32, accent), eframe::egui::StrokeKind::Inside);
        let Some(m) = self.glyph_metrics(ui.ctx(), face, gid, pt) else { return };
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let room = area.shrink(if labels { 12.0 } else { 8.0 });
        let label_w = if labels { 118.0 } else { 0.0 };
        let fit = ((room.width() - label_w) / (m.size.x / ppp)).min(room.height() / (m.size.y / ppp)).min(1.0);
        let s = fit / ppp; // pixels of the picture → points on screen
        let size = m.size * s;
        // Placed as RightFont's -[RFGlyphDrawingView drawRect:] (0x10008d54c) does: the advance
        // width centred across, and the ascent+descent band centred up and down, so the baseline
        // sits at descent + (height − (ascent + descent)) × 0.5 from the bottom.
        let cx = room.left() + label_w + (room.width() - label_w) * 0.5;
        let o = Pos2::new(cx - m.advance * s * 0.5, room.center().y + (m.ascender - m.descender) * s * 0.5);
        let img = Rect::from_min_size(o - m.origin.to_vec2() * s, size);
        let (x0, x1) = (img.left() - 4.0, img.right() + 4.0);
        let line = Color32::from_white_alpha(70);
        let solid = Stroke::new(1.0_f32, line);
        let dots = Stroke::new(1.0_f32, Color32::from_white_alpha(110));
        let rows: [(f32, bool, K); 5] = [
            (o.y, false, K::MBaseline),
            (o.y - m.x_height * s, false, K::MXHeight),
            (o.y - m.cap_height * s, false, K::MCapHeight),
            (o.y - m.ascender * s, true, K::MAscender),
            (o.y + m.descender * s, true, K::MDescender),
        ];
        for (y, dotted, name) in rows {
            if dotted {
                dashed(p, Pos2::new(x0, y), Pos2::new(x1, y), dots);
            } else {
                p.line_segment([Pos2::new(x0, y), Pos2::new(x1, y)], solid);
            }
            if labels {
                p.text(Pos2::new(room.left(), y), Align2::LEFT_CENTER, t(name), FontId::proportional(10.0), Color32::from_white_alpha(170));
            }
        }
        // The glyph's width (advance): from its origin to where the next glyph starts.
        let (wl, wr) = (o.x, o.x + m.advance * s);
        p.line_segment([Pos2::new(wl, img.top()), Pos2::new(wl, img.bottom())], solid);
        p.line_segment([Pos2::new(wr, img.top()), Pos2::new(wr, img.bottom())], solid);
        if labels {
            p.text(Pos2::new((wl + wr) / 2.0, img.bottom() + 2.0), Align2::CENTER_TOP, t(K::MWidth), FontId::proportional(10.0), Color32::from_white_alpha(170));
        }
        p.image(m.tex.id(), img, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        // The keys that type this character with the keyboard in use, at the top right in the
        // activation asterisk's green, easy to see on the black: the person asked for 5x the first
        // size (85 / 75 pt), then for half of that (42.5 / 37.5 pt), then for 60 % less (17 / 15
        // pt), then for the preview's 20 % more (18 pt), and Character Info the same: 18 pt in both.
        if let Some(keys) = self.glyph_char(gid).and_then(crate::keys::shortcut_for) {
            let at = Pos2::new(area.right() - 12.0, area.top() + 12.0);
            let font = FontId::proportional(crate::metrics::KEYS_PT);
            let r = crate::finder::text(ui, at, Align2::RIGHT_TOP, &keys, font.size, 0.3, crate::libs::LIME, &font);
            let _ = r;
        }
    }
}
