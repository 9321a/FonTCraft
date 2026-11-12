//! Glyphs page tools: the right-click menu (character info, copying the character in several
//! forms, marking the fonts that have it, finding characters with a similar shape), the character
//! info window, and the similar-shapes search.

use crate::i18n::{self, K};
use crate::App;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Vec2};
use std::sync::{Arc, Mutex};

/// The Unicode block a character is in (the main blocks; others show as their code range).
pub fn unicode_block(c: u32) -> String {
    const BLOCKS: &[(u32, u32, &str)] = &[
        (0x0000, 0x007F, "Basic Latin"),
        (0x0080, 0x00FF, "Latin-1 Supplement"),
        (0x0100, 0x017F, "Latin Extended-A"),
        (0x0180, 0x024F, "Latin Extended-B"),
        (0x0250, 0x02AF, "IPA Extensions"),
        (0x02B0, 0x02FF, "Spacing Modifier Letters"),
        (0x0300, 0x036F, "Combining Diacritical Marks"),
        (0x0370, 0x03FF, "Greek and Coptic"),
        (0x0400, 0x04FF, "Cyrillic"),
        (0x0500, 0x052F, "Cyrillic Supplement"),
        (0x0530, 0x058F, "Armenian"),
        (0x0590, 0x05FF, "Hebrew"),
        (0x0600, 0x06FF, "Arabic"),
        (0x0900, 0x097F, "Devanagari"),
        (0x0980, 0x09FF, "Bengali"),
        (0x0E00, 0x0E7F, "Thai"),
        (0x10A0, 0x10FF, "Georgian"),
        (0x1100, 0x11FF, "Hangul Jamo"),
        (0x1E00, 0x1EFF, "Latin Extended Additional"),
        (0x1F00, 0x1FFF, "Greek Extended"),
        (0x2000, 0x206F, "General Punctuation"),
        (0x2070, 0x209F, "Superscripts and Subscripts"),
        (0x20A0, 0x20CF, "Currency Symbols"),
        (0x20D0, 0x20FF, "Combining Marks for Symbols"),
        (0x2100, 0x214F, "Letterlike Symbols"),
        (0x2150, 0x218F, "Number Forms"),
        (0x2190, 0x21FF, "Arrows"),
        (0x2200, 0x22FF, "Mathematical Operators"),
        (0x2300, 0x23FF, "Miscellaneous Technical"),
        (0x2400, 0x243F, "Control Pictures"),
        (0x2460, 0x24FF, "Enclosed Alphanumerics"),
        (0x2500, 0x257F, "Box Drawing"),
        (0x2580, 0x259F, "Block Elements"),
        (0x25A0, 0x25FF, "Geometric Shapes"),
        (0x2600, 0x26FF, "Miscellaneous Symbols"),
        (0x2700, 0x27BF, "Dingbats"),
        (0x27F0, 0x27FF, "Supplemental Arrows-A"),
        (0x2800, 0x28FF, "Braille Patterns"),
        (0x2900, 0x297F, "Supplemental Arrows-B"),
        (0x2B00, 0x2BFF, "Miscellaneous Symbols and Arrows"),
        (0x2C60, 0x2C7F, "Latin Extended-C"),
        (0x2E00, 0x2E7F, "Supplemental Punctuation"),
        (0x3000, 0x303F, "CJK Symbols and Punctuation"),
        (0x3040, 0x309F, "Hiragana"),
        (0x30A0, 0x30FF, "Katakana"),
        (0x3100, 0x312F, "Bopomofo"),
        (0x3130, 0x318F, "Hangul Compatibility Jamo"),
        (0x4E00, 0x9FFF, "CJK Unified Ideographs"),
        (0xA720, 0xA7FF, "Latin Extended-D"),
        (0xAC00, 0xD7AF, "Hangul Syllables"),
        (0xE000, 0xF8FF, "Private Use Area"),
        (0xFB00, 0xFB4F, "Alphabetic Presentation Forms"),
        (0xFE70, 0xFEFF, "Arabic Presentation Forms-B"),
        (0xFF00, 0xFFEF, "Halfwidth and Fullwidth Forms"),
        (0x1D400, 0x1D7FF, "Mathematical Alphanumeric Symbols"),
        (0x1F300, 0x1F5FF, "Miscellaneous Symbols and Pictographs"),
        (0x1F600, 0x1F64F, "Emoticons"),
        (0x1F680, 0x1F6FF, "Transport and Map Symbols"),
        (0x1F900, 0x1F9FF, "Supplemental Symbols and Pictographs"),
        (0xF0000, 0xFFFFD, "Supplementary Private Use Area-A"),
    ];
    BLOCKS.iter().find(|(a, b, _)| (*a..=*b).contains(&c)).map(|b| b.2.to_string()).unwrap_or_else(|| format!("U+{:04X}…", c & !0x7F))
}

/// The forms "Copy as…" puts on the clipboard.
pub fn html_of(c: char) -> String {
    match c {
        '&' => "&amp;".into(),
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '"' => "&quot;".into(),
        '\u{a0}' => "&nbsp;".into(),
        _ => format!("&#{};", c as u32),
    }
}
pub fn hex_of(c: char) -> String {
    format!("U+{:04X}", c as u32)
}

/// A glyph's shape reduced to a small grid of ink (its bounding box scaled to fill the grid),
/// plus its proportions; two glyphs look alike when these are close.
#[derive(Clone)]
struct Shape {
    grid: [u8; 256],
    aspect: f32,
}

const GRID: usize = 16;

fn shape_of(font: &ab_glyph::FontRef, gid: u16) -> Option<Shape> {
    use ab_glyph::{point, Font, GlyphId, PxScale};
    let o = font.outline_glyph(GlyphId(gid).with_scale_and_position(PxScale::from(48.0), point(0.0, 48.0)))?;
    let b = o.px_bounds();
    let (w, h) = (b.width().max(1.0), b.height().max(1.0));
    let side = w.max(h);
    let mut acc = [0f32; GRID * GRID];
    let mut cnt = [0f32; GRID * GRID];
    // Square box around the glyph so thin and wide shapes keep their proportions.
    let ox = (side - w) / 2.0;
    let oy = (side - h) / 2.0;
    o.draw(|x, y, c| {
        let gx = (((x as f32 + ox) / side) * GRID as f32).clamp(0.0, GRID as f32 - 1.0) as usize;
        let gy = (((y as f32 + oy) / side) * GRID as f32).clamp(0.0, GRID as f32 - 1.0) as usize;
        acc[gy * GRID + gx] += c;
        cnt[gy * GRID + gx] += 1.0;
    });
    let cell = (side / GRID as f32).powi(2).max(1.0);
    let mut grid = [0u8; 256];
    for i in 0..GRID * GRID {
        let _ = cnt[i];
        grid[i] = ((acc[i] / cell).clamp(0.0, 1.0) * 255.0) as u8;
    }
    Some(Shape { grid, aspect: w / h })
}

fn distance(a: &Shape, b: &Shape) -> f32 {
    let d: u32 = a.grid.iter().zip(b.grid.iter()).map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs()).sum();
    d as f32 / (255.0 * 256.0) + (a.aspect.ln() - b.aspect.ln()).abs() * 0.25
}

/// One found character: the face (in the font list), its glyph, the character and how far off it is.
#[derive(Clone)]
pub struct Found {
    pub face: usize,
    pub gid: u16,
    pub ch: char,
    pub score: f32,
}

#[derive(Default)]
pub struct SimilarState {
    pub done: usize,
    pub total: usize,
    pub finished: bool,
    pub found: Vec<Found>,
}

pub struct Similar {
    pub ch: char,
    pub state: Arc<Mutex<SimilarState>>,
}

/// Starts looking through every font (one face per family, switched on or not) for characters
/// shaped like glyph `gid` of `face`. Runs on several threads; results arrive as they're found.
pub fn start_similar(app: &App, face: usize, gid: u16, ch: char) -> Option<Similar> {
    let f = &app.lib.faces[face];
    let data = app.lib.files[f.file].data.clone();
    let target = {
        let font = ab_glyph::FontRef::try_from_slice_and_index(&data, f.index).ok()?;
        shape_of(&font, gid)?
    };
    let jobs: Vec<(usize, std::sync::Arc<memmap2::Mmap>, u32)> = app
        .lib
        .families
        .iter()
        .map(|fam| {
            let fc = &app.lib.faces[fam.default_face];
            (fam.default_face, app.lib.files[fc.file].data.clone(), fc.index)
        })
        .collect();
    let state = Arc::new(Mutex::new(SimilarState { total: jobs.len(), ..Default::default() }));
    let jobs = Arc::new(Mutex::new(jobs));
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(2, 8);
    let alive = Arc::new(std::sync::atomic::AtomicUsize::new(threads));
    for _ in 0..threads {
        let (jobs, state, target, alive) = (jobs.clone(), state.clone(), target.clone(), alive.clone());
        let from = (face, gid);
        std::thread::spawn(move || {
            loop {
                let Some((face, data, index)) = jobs.lock().unwrap().pop() else { break };
                let mut best: Vec<Found> = Vec::new();
                if let (Ok(font), Ok(tf)) = (ab_glyph::FontRef::try_from_slice_and_index(&data, index), ttf_parser::Face::parse(&data, index)) {
                    let mut seen = 0;
                    if let Some(sub) = tf.tables().cmap.and_then(|c| c.subtables.into_iter().find(|s| s.is_unicode())) {
                        sub.codepoints(|cp| {
                            if seen > 3000 {
                                return;
                            }
                            let Some(c) = char::from_u32(cp) else { return };
                            if c.is_control() || c.is_whitespace() {
                                return;
                            }
                            seen += 1;
                            if let Some(g) = sub.glyph_index(cp) {
                                if face == from.0 && g.0 == from.1 {
                                    return; // the chosen glyph itself
                                }
                                if let Some(s) = shape_of(&font, g.0) {
                                    let score = distance(&target, &s);
                                    if score < 0.16 {
                                        best.push(Found { face, gid: g.0, ch: c, score });
                                    }
                                }
                            }
                        });
                    }
                }
                best.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                best.truncate(6);
                let mut st = state.lock().unwrap();
                st.done += 1;
                st.found.extend(best);
                st.found.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                st.found.truncate(60);
            }
            if alive.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                state.lock().unwrap().finished = true;
            }
        });
    }
    Some(Similar { ch, state })
}

impl App {
    /// The character a glyph of the open face stands for, if any.
    pub(crate) fn glyph_char(&self, gid: u16) -> Option<char> {
        self.face_data.as_ref().and_then(|d| d.cmap.get(&gid).copied()).and_then(char::from_u32)
    }

    /// The right-click menu of a glyph on the Glyphs page.
    pub(crate) fn glyph_menu(&mut self, ui: &mut egui::Ui, face: usize, gid: u16) {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ch = self.glyph_char(gid);
        let c = ch.map(|c| c.to_string()).unwrap_or_default();
        if ui.button(t(K::CharInfo)).clicked() {
            self.char_info = Some((face, gid));
            ui.close();
        }
        ui.separator();
        let copies: [(String, Option<String>); 4] = [
            (t(K::CopyChar).replace("{c}", &c), ch.map(|c| c.to_string())),
            (t(K::CopyHtml).replace("{v}", &ch.map(html_of).unwrap_or_default()), ch.map(html_of)),
            (t(K::CopyDec).replace("{v}", &ch.map(|c| (c as u32).to_string()).unwrap_or_default()), ch.map(|c| (c as u32).to_string())),
            (t(K::CopyHex).replace("{v}", &ch.map(hex_of).unwrap_or_default()), ch.map(hex_of)),
        ];
        for (label, value) in copies {
            if ui.add_enabled(value.is_some(), egui::Button::new(label)).clicked() {
                if let Some(v) = value {
                    self.notice = Some(t(K::Copied).replace("{v}", &v));
                    self.clip = Some(v);
                }
                ui.close();
            }
        }
        ui.separator();
        let marked = ch.is_some() && self.mark_char == ch;
        let label = if marked { t(K::UnmarkFonts) } else { t(K::MarkFonts) }.replace("{c}", &c);
        if ui.add_enabled(ch.is_some(), egui::Button::new(label)).clicked() {
            self.mark_char = if marked { None } else { ch };
            self.mark_cache.clear();
            ui.close();
        }
        ui.separator();
        if ui.add_enabled(ch.is_some(), egui::Button::new(t(K::FindSimilar))).clicked() {
            if let Some(c) = ch {
                self.similar = start_similar(self, face, gid, c);
            }
            ui.close();
        }
    }

    /// Whether a family has the marked character (cached per family).
    pub(crate) fn family_has_mark(&mut self, family: usize) -> bool {
        let Some(c) = self.mark_char else { return false };
        if let Some(v) = self.mark_cache.get(&family) {
            return *v;
        }
        let has = self.lib.families[family].faces.iter().any(|&f| {
            let fc = &self.lib.faces[f];
            ttf_parser::Face::parse(&self.lib.files[fc.file].data, fc.index).ok().and_then(|tf| tf.glyph_index(c)).is_some()
        });
        self.mark_cache.insert(family, has);
        has
    }

    /// The character info window (like a character palette's info panel).
    pub(crate) fn char_info_window(&mut self, ctx: &egui::Context) {
        let Some((face, gid)) = self.char_info else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ch = self.glyph_char(gid);
        let (glyph_name, advance) = {
            let f = &self.lib.faces[face];
            match ttf_parser::Face::parse(&self.lib.files[f.file].data, f.index) {
                Ok(tf) => (
                    tf.glyph_name(ttf_parser::GlyphId(gid)).map(String::from).unwrap_or_else(|| "—".into()),
                    tf.glyph_hor_advance(ttf_parser::GlyphId(gid)).map(|a| format!("{a} / {}", tf.units_per_em())).unwrap_or_else(|| "—".into()),
                ),
                Err(_) => ("—".into(), "—".into()),
            }
        };
        let font = format!("{} {}", self.lib.faces[face].family, self.lib.faces[face].style);
        let mut rows: Vec<(String, String)> = Vec::new();
        if let Some(c) = ch {
            let mut b = [0u8; 4];
            let utf8: String = c.encode_utf8(&mut b).bytes().map(|x| format!("{x:02X} ")).collect();
            let mut u = [0u16; 2];
            let utf16: String = c.encode_utf16(&mut u).iter().map(|x| format!("{x:04X} ")).collect();
            rows.push((t(K::UnicodeLabel).into(), hex_of(c)));
            rows.push((t(K::DecimalLabel).into(), (c as u32).to_string()));
            rows.push((t(K::HtmlLabel).into(), html_of(c)));
            rows.push((t(K::Utf8Label).into(), utf8.trim().to_string()));
            rows.push((t(K::Utf16Label).into(), utf16.trim().to_string()));
            rows.push((t(K::BlockLabel).into(), unicode_block(c as u32)));
        } else {
            rows.push((t(K::UnicodeLabel).into(), t(K::NotMapped).into()));
        }
        rows.push((t(K::GlyphName).into(), glyph_name));
        rows.push((t(K::GlyphIdLabel).into(), gid.to_string()));
        rows.push((t(K::AdvanceLabel).into(), advance));
        rows.push((t(K::FontLabel).into(), font));
        // A system window of its own, centred on the screen.
        let (_, closed) = crate::oslook::dialog_centered(ctx, "char-info", t(K::CharInfo), 680.0, pal.panel, None, |ui| {
                ui.horizontal_top(|ui| {
                    // The glyph at 144 points (or as large as fits) with the font's lines, named.
                    let (r, _) = ui.allocate_exact_size(Vec2::new(330.0, 280.0), Sense::hover());
                    self.paint_glyph_metrics(ui, r, face, gid, crate::metrics::PREVIEW_PT, true);
                    ui.vertical(|ui| {
                        egui::Grid::new("char-info-grid").num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                            for (k, v) in &rows {
                                ui.label(egui::RichText::new(k).size(12.0).color(pal.muted));
                                ui.label(egui::RichText::new(v).monospace().color(pal.ink));
                                ui.end_row();
                            }
                        });
                    });
                });
            });
        if closed {
            self.char_info = None;
        }
    }

    /// The floating window with characters shaped like the chosen one, at 96 points. Clicking one
    /// closes the window and puts it into the app FontCraft works with (or copies it).
    pub(crate) fn similar_window(&mut self, ctx: &egui::Context) {
        let Some(sim) = &self.similar else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let ch = sim.ch;
        let (found, done, total, finished) = {
            let s = sim.state.lock().unwrap();
            (s.found.clone(), s.done, s.total, s.finished)
        };
        if !finished {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
        let target = self.target_app();
        let mut pick: Option<Found> = None;
        let title = format!("{} — “{ch}”", t(K::SimilarTitle));
        // A system window of its own, centred on the screen; its grid scrolls inside a set height.
        let (_, closed) = crate::oslook::dialog_centered(ctx, "similar-shapes", &title, 660.0, pal.panel, Some(540.0), |ui| {
                ui.horizontal(|ui| {
                    if !finished {
                        ui.spinner();
                        ui.label(egui::RichText::new(t(K::SearchingFonts).replace("{n}", &total.to_string())).size(12.5).color(pal.muted));
                        ui.label(egui::RichText::new(format!("{done}/{total}")).monospace().size(12.0).color(pal.muted));
                    } else if found.is_empty() {
                        ui.label(egui::RichText::new(t(K::NoSimilar)).color(pal.muted));
                    } else {
                        let hint = match target {
                            Some(a) => t(K::ClickToInsert).replace("{app}", self.autoact.apps[a].def_name()),
                            None => t(K::ClickToCopy).to_string(),
                        };
                        ui.label(egui::RichText::new(hint).size(12.5).color(pal.muted));
                    }
                });
                ui.add_space(6.0);
                let cell = Vec2::new(140.0, 150.0);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    let cols = ((ui.available_width()) / cell.x).floor().max(1.0) as usize;
                    for row in found.chunks(cols) {
                        ui.horizontal(|ui| {
                            for f in row {
                                let (r, resp) = ui.allocate_exact_size(cell, Sense::click());
                                if resp.hovered() {
                                    ui.painter().rect_filled(r, 8.0, pal.hover);
                                }
                                let (tex, size, _, _) = self.texture_glyph(ctx, f.face, f.gid, 96.0);
                                if let Some(tex) = tex {
                                    let room = Vec2::new(r.width() - 8.0, r.height() - 34.0);
                                    let k = (room.x / size.x.max(1.0)).min(room.y / size.y.max(1.0)).min(1.0);
                                    let img = Rect::from_center_size(r.center_top() + Vec2::new(0.0, 4.0 + room.y / 2.0), size * k);
                                    ui.painter().image(tex.id(), img, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), pal.ink);
                                }
                                let fam = &self.lib.faces[f.face].family;
                                let short: String = fam.chars().take(20).collect();
                                ui.painter().text(r.center_bottom() - Vec2::new(0.0, 22.0), Align2::CENTER_CENTER, hex_of(f.ch), FontId::monospace(11.0), pal.muted);
                                ui.painter().text(r.center_bottom() - Vec2::new(0.0, 9.0), Align2::CENTER_CENTER, short, FontId::proportional(11.0), pal.muted);
                                if resp.on_hover_text(format!("{} · {}", fam, hex_of(f.ch))).clicked() {
                                    pick = Some(f.clone());
                                }
                            }
                        });
                    }
                });
            });
        if let Some(f) = pick {
            self.similar = None;
            match target {
                Some(app) => self.insert_char(app, f.face, f.ch),
                None => {
                    self.notice = Some(t(K::Copied).replace("{v}", &f.ch.to_string()));
                    self.clip = Some(f.ch.to_string());
                }
            }
            return;
        }
        if closed {
            self.similar = None;
        }
    }

    /// A red mark (the asterisk's size) for fonts that have the marked character.
    pub(crate) fn paint_mark_dot(p: &egui::Painter, c: Pos2) {
        p.circle_filled(c, 6.5, Color32::from_rgb(0xe5, 0x3b, 0x3b));
    }
}
