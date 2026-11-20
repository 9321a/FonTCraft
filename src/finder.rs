//! The sidebar drawn the way Finder draws its own on macOS 26: a floating card with rounded corners
//! inset from the window, section titles in small grey bold type (their fold chevron shows at the
//! right only under the pointer), rows with icons tinted in the accent colour, the name in the
//! label colour and a soft grey rounded selection. Every row of the sidebar (library, sources,
//! smart lists, libraries, folders and their lists) goes through here, so they all look alike.
//!
//! Sizes follow the sidebar size chosen in System Settings › Appearance, as Finder's do
//! (NSTableViewDefaultSizeMode: small, medium, large): the text is written by the system's own text
//! engine at that size, and the icons are sized to the text.

use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Shadow, Stroke, Ui, Vec2};

use crate::settings::Pal;

/// Corner radius of the floating card.
pub const CARD_RADIUS: u8 = 14;
/// Space between the card and the window's edges.
pub const CARD_INSET: i8 = 8;
/// Room inside the card on either side of the rows.
pub const CARD_PAD: f32 = 8.0;
/// NSFontWeightBold: the weight of an outline view's group rows (Finder's section titles).
pub const TITLE_WEIGHT: f32 = 0.4;

/// The sidebar's sizes for one system sidebar size.
#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    /// Names (Finder: 11, 13 or 15 points for small, medium, large).
    pub label: f32,
    /// Section titles.
    pub title: f32,
    /// Counts at the right.
    pub count: f32,
    /// Row height.
    pub row_h: f32,
    /// Width kept for the icon, which is sized to the text.
    pub icon_box: f32,
}

impl Metrics {
    fn icon_x(&self) -> f32 {
        4.0 + self.icon_box / 2.0
    }
    fn text_x(&self) -> f32 {
        4.0 + self.icon_box + 7.0
    }
}

/// The sizes for the system's sidebar size now (estimated from Finder's look at each size).
pub fn metrics() -> Metrics {
    let (label, title, row_h) = match crate::oslook::sidebar_size_mode() {
        1 => (11.0, 10.0, 24.0),
        3 => (15.0, 12.0, 32.0),
        _ => (13.0, 11.0, 28.0),
    };
    Metrics { label, title, count: title, row_h, icon_box: (label * 1.5_f32).round() }
}

/// The card's fill: a shade apart from the content, as Finder's glass sidebar sits over it.
pub fn card_fill(pal: &Pal) -> Color32 {
    if pal.dark {
        mix(pal.panel, Color32::WHITE, 0.055)
    } else {
        mix(pal.panel, Color32::from_rgb(0x76, 0x76, 0x80), 0.10)
    }
}

/// The selected row's fill: neutral grey, as Finder's sidebar shows its selection.
pub fn selection_fill(pal: &Pal) -> Color32 {
    if pal.dark {
        Color32::from_white_alpha(30)
    } else {
        Color32::from_black_alpha(22)
    }
}

fn hover_fill(pal: &Pal) -> Color32 {
    if pal.dark {
        Color32::from_white_alpha(14)
    } else {
        Color32::from_black_alpha(10)
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Section titles' grey (Finder's tertiary label).
pub fn title_ink(pal: &Pal) -> Color32 {
    mix(pal.muted, card_fill(pal), 0.18)
}

/// Draws the floating card in `rect` with its soft shadow and hairline edge.
pub fn paint_card(ui: &Ui, rect: Rect, pal: &Pal) {
    let p = ui.painter();
    let shadow = Shadow { offset: [0, 2], blur: 14, spread: 0, color: Color32::from_black_alpha(if pal.dark { 90 } else { 28 }) };
    p.add(shadow.as_shape(rect, CornerRadius::same(CARD_RADIUS)));
    p.rect_filled(rect, CornerRadius::same(CARD_RADIUS), card_fill(pal));
    let edge = if pal.dark { Color32::from_white_alpha(22) } else { Color32::from_black_alpha(16) };
    p.rect_stroke(rect, CornerRadius::same(CARD_RADIUS), Stroke::new(1.0, edge), egui::StrokeKind::Inside);
}

/// A section title ("Biblioteca", "Orígenes"…). Clicking it folds the section; its chevron shows
/// at the right while the pointer is over the title, as in Finder. `extra` adds controls at the
/// right (such as "+"), before the chevron. Returns whether the title was clicked.
pub fn section_header(ui: &mut Ui, pal: &Pal, title: &str, open: bool, extra: impl FnOnce(&mut Ui)) -> bool {
    let m = metrics();
    let w = ui.available_width();
    ui.add_space(m.title * 0.4);
    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, (m.title * 2.2).round()), Sense::click());
    let ink = title_ink(pal);
    let font = FontId::proportional(m.title);
    let title = &sentence_case(title);
    let at = Pos2::new(r.left() + 8.0, r.center().y + 1.0);
    text(ui, at, Align2::LEFT_CENTER, title, m.title, TITLE_WEIGHT, ink, &font);
    let hot = ui.rect_contains_pointer(r);
    let chevron = Pos2::new(r.right() - 12.0, r.center().y + 1.0);
    if hot || !open {
        let name = if open { "chevron.down" } else { "chevron.right" };
        let col = if hot { pal.muted } else { ink.gamma_multiply(0.7) };
        if crate::oslook::symbol_for_text(ui, name, chevron, m.title * 0.85, col).is_none() {
            crate::oslook::disclosure(ui, chevron, open, col);
        }
    }
    let extra_rect = Rect::from_min_max(r.min, Pos2::new(r.right() - 24.0, r.max.y));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(extra_rect).layout(egui::Layout::right_to_left(egui::Align::Center)));
    extra(&mut child);
    resp.clicked()
}

/// A row's picture.
#[derive(Clone, Copy)]
pub enum Icon<'a> {
    /// A system symbol (SF Symbols on macOS).
    Symbol(&'a str),
    /// One of FontCraft's own pictures (`uiicons`): "library" and "folder" are the ones the Import
    /// Fonts window shows for libraries and folders, "set" a subfolder, "family" a font family.
    Own(&'static str),
}

/// What a row shows.
pub struct Row<'a> {
    pub icon: Icon<'a>,
    pub label: &'a str,
    pub count: Option<usize>,
    pub selected: bool,
    /// How far in the row sits (nested lists), in points.
    pub indent: f32,
    /// Leaves room at the left for a disclosure arrow (libraries with lists).
    pub disclosure: Option<bool>,
}

/// One of FontCraft's own pictures, `pt` points square, tinted `tint`, centred at `c`.
pub fn own_icon(ui: &Ui, name: &'static str, c: Pos2, pt: f32, tint: Color32) -> Option<Rect> {
    let ctx = ui.ctx();
    let px = (pt * ctx.pixels_per_point()).round().max(8.0) as u32;
    let id = egui::Id::unique(("own-icon", name, px));
    let tex = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = crate::uiicons::rgba(name, [255, 255, 255], px).map(|rgba| ctx.load_texture(format!("own-{name}"), egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rgba), egui::TextureOptions::LINEAR));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    let t = tex?;
    let rect = Rect::from_center_size(c, Vec2::splat(pt));
    ui.painter().image(t.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    Some(rect)
}

/// One sidebar row; `fallback` draws the picture where the system has no symbol.
/// Returns the row's response, its rect, and the disclosure arrow's rect when it has one.
pub fn row(ui: &mut Ui, pal: &Pal, row: Row, fallback: impl FnOnce(&egui::Painter, Pos2, Color32)) -> (Response, Rect, Option<Rect>) {
    let m = metrics();
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(Vec2::new(w, m.row_h), Sense::click());
    let r = r.shrink2(Vec2::new(0.0, 1.0));
    if row.selected {
        ui.painter().rect_filled(r, CornerRadius::same(8), selection_fill(pal));
    } else if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(8), hover_fill(pal));
    }
    let mut x0 = r.left() + row.indent;
    let mut tri = None;
    if let Some(open) = row.disclosure {
        let c = Pos2::new(x0 + 8.0, r.center().y);
        let t = Rect::from_center_size(c, Vec2::splat(16.0));
        let col = pal.muted;
        if crate::oslook::symbol_for_text(ui, if open { "chevron.down" } else { "chevron.right" }, c, m.label * 0.75, col).is_none() {
            crate::oslook::disclosure(ui, c, open, col);
        }
        tri = Some(t);
        x0 += 14.0;
    }
    let icon_c = Pos2::new(x0 + m.icon_x(), r.center().y);
    let tint = pal.accent;
    let drawn = match row.icon {
        Icon::Symbol(name) => crate::oslook::symbol_for_text(ui, name, icon_c, m.label, tint),
        // FontCraft's pictures fill their square, so they are drawn a little larger than the text.
        Icon::Own(name) => own_icon(ui, name, icon_c, (m.label * 1.35).round(), tint),
    };
    if drawn.is_none() {
        fallback(ui.painter(), icon_c, tint);
    }
    let font = FontId::proportional(m.label);
    let mut right = r.right() - 8.0;
    if let Some(n) = row.count {
        let cr = text(ui, Pos2::new(right, r.center().y), Align2::RIGHT_CENTER, &n.to_string(), m.count, 0.0, title_ink(pal), &FontId::proportional(m.count));
        right = cr.left() - 8.0;
    }
    let tx = x0 + m.text_x();
    let room = (right - tx).max(20.0);
    let label = fit(ui, row.label, &font, room);
    text(ui, Pos2::new(tx, r.center().y), Align2::LEFT_CENTER, &label, m.label, 0.0, pal.ink, &font);
    (resp, r, tri)
}

/// Writes `s` with the system's text engine, or with egui's copy of the font where it can't.
#[allow(clippy::too_many_arguments)]
pub fn text(ui: &Ui, pos: Pos2, align: Align2, s: &str, pt: f32, weight: f32, color: Color32, fallback: &FontId) -> Rect {
    match crate::oslook::sys_text(ui, pos, align, s, pt, weight, color) {
        Some(r) => r,
        None => ui.painter().text(pos, align, s, fallback.clone(), color),
    }
}

/// How wide `s` is written in the sidebar.
pub fn text_width(ui: &Ui, s: &str, pt: f32, weight: f32) -> f32 {
    crate::oslook::sys_text_width(s, pt, weight).unwrap_or_else(|| ui.painter().layout_no_wrap(s.to_string(), FontId::proportional(pt), Color32::WHITE).size().x)
}

/// Finder writes its section titles in sentence case ("Favoritos", "Ubicaciones"): a title written
/// all in capitals becomes "Listas inteligentes"; one already mixed is kept as it is.
pub fn sentence_case(title: &str) -> String {
    let has_lower = title.chars().any(|c| c.is_lowercase());
    if has_lower {
        return title.to_string();
    }
    let lower = title.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// `text` shortened with "…" to fit `room` points.
pub fn fit(ui: &Ui, text: &str, font: &FontId, room: f32) -> String {
    let width = |s: &str| text_width(ui, s, font.size, 0.0);
    if width(text) <= room {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut n = chars.len();
    while n > 1 {
        n -= 1;
        let s: String = chars[..n].iter().collect::<String>() + "…";
        if width(&s) <= room {
            return s;
        }
    }
    "…".into()
}
