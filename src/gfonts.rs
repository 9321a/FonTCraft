//! Google Fonts: the list of families (from Google's public font list), downloading a family's
//! files in the background, and the window to search and download.
//! Downloaded families live in one folder; that folder is the "Google Fonts" library and each
//! family is one of its lists.

use crate::i18n::{self, K};
use crate::libs;
use crate::{App, View};
use eframe::egui::{self, Align2, Color32, FontId, Margin, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Hover preview size in the Google Fonts window (the person asked for 60 pt).
const PREVIEW_PT: f32 = 60.0;
const LIST_URL: &str = "https://fonts.google.com/metadata/fonts";
const CSS_URL: &str = "https://fonts.googleapis.com/css2";
/// The saved copy of the family list is refreshed after a week.
const LIST_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

#[derive(Clone)]
pub struct GFamily {
    pub name: String,
    pub category: String,
    /// Styles as Google names them: "400", "700i", …
    pub styles: Vec<String>,
    pub popularity: u64,
}

#[derive(Clone, PartialEq)]
pub enum Dl {
    Busy,
    Done,
    Failed,
}

#[derive(Default)]
pub struct Shared {
    pub list: Option<Result<Vec<GFamily>, String>>,
    /// "Download All": families done and families in the batch, while it runs.
    pub bulk: Option<(usize, usize)>,
    /// Hover previews: a family's regular style (None while it's being fetched).
    pub previews: HashMap<String, Option<Vec<u8>>>,
    pub loading: bool,
    pub downloads: HashMap<String, Dl>,
    /// A download finished: the library needs reading again.
    pub changed: bool,
}

#[derive(Default)]
pub struct GState {
    pub shared: Arc<Mutex<Shared>>,
    /// Pictures of the hover previews (48 pt), by family.
    pub preview_tex: HashMap<String, egui::TextureHandle>,
    pub open: bool,
    pub search: String,
    pub category: Option<&'static str>,
}

/// Where downloaded families are kept (one subfolder per family).
pub fn dir() -> PathBuf {
    libs::data_dir().join("Google Fonts")
}

fn family_dir(name: &str) -> PathBuf {
    dir().join(name.replace(['/', ':'], "-"))
}

pub fn is_downloaded(name: &str) -> bool {
    std::fs::read_dir(family_dir(name))
        .map(|rd| rd.flatten().any(|e| e.path().extension().is_some_and(|x| x == "ttf")))
        .unwrap_or(false)
}

fn parse_list(text: &str) -> Result<Vec<GFamily>, String> {
    // The reply may start with a few guard characters before the JSON.
    let start = text.find('{').ok_or("empty list")?;
    let v: serde_json::Value = serde_json::from_str(&text[start..]).map_err(|e| e.to_string())?;
    let list = v.get("familyMetadataList").and_then(|x| x.as_array()).ok_or("no families")?;
    let mut out: Vec<GFamily> = list
        .iter()
        .filter_map(|f| {
            let name = f.get("family")?.as_str()?.to_string();
            let mut styles: Vec<String> = f.get("fonts")?.as_object()?.keys().cloned().collect();
            styles.sort_by_key(|s| style_key(s));
            Some(GFamily {
                name,
                category: f.get("category").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                styles,
                popularity: f.get("popularity").and_then(|p| p.as_u64()).unwrap_or(u64::MAX),
            })
        })
        .collect();
    out.sort_by_key(|f| f.popularity);
    Ok(out)
}

/// "700i" → (italic, weight).
fn style_key(s: &str) -> (u8, u16) {
    let italic = s.ends_with('i');
    let w = s.trim_end_matches('i').parse().unwrap_or(400);
    (italic as u8, w)
}

fn style_name(weight: u16, italic: bool) -> String {
    let w = match weight {
        100 => "Thin",
        200 => "ExtraLight",
        300 => "Light",
        400 => "Regular",
        500 => "Medium",
        600 => "SemiBold",
        700 => "Bold",
        800 => "ExtraBold",
        900 => "Black",
        _ => "",
    };
    match (w, italic) {
        ("Regular", true) => "Italic".into(),
        ("", i) => format!("{weight}{}", if i { "Italic" } else { "" }),
        (w, true) => format!("{w}Italic"),
        (w, false) => w.into(),
    }
}

/// Downloads a page as text / a file as bytes (Google Fonts). The `net` feature (on by default) can be
/// left out to check the rest of FontCraft for a system whose C libraries aren't on this computer.
#[cfg(feature = "net")]
fn get_text(url: &str) -> Result<String, String> {
    let mut resp = agent().get(url).call().map_err(|e| e.to_string())?;
    resp.body_mut().with_config().limit(64 << 20).read_to_string().map_err(|e| e.to_string())
}
#[cfg(feature = "net")]
fn get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut resp = agent().get(url).call().map_err(|e| e.to_string())?;
    resp.body_mut().with_config().limit(256 << 20).read_to_vec().map_err(|e| e.to_string())
}
#[cfg(not(feature = "net"))]
fn get_text(_url: &str) -> Result<String, String> {
    Err("built without network".into())
}
#[cfg(not(feature = "net"))]
fn get_bytes(_url: &str) -> Result<Vec<u8>, String> {
    Err("built without network".into())
}

#[cfg(feature = "net")]
fn agent() -> ureq::Agent {
    // macOS and Windows: the system's own secure connections; Linux and Android: rustls.
    let provider = if cfg!(any(target_os = "linux", target_os = "android")) { ureq::tls::TlsProvider::Rustls } else { ureq::tls::TlsProvider::NativeTls };
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .tls_config(ureq::tls::TlsConfig::builder().provider(provider).build())
        .build()
        .into()
}

/// Loads the family list in the background (from the saved copy when it's recent).
pub fn start_list(shared: &Arc<Mutex<Shared>>) {
    {
        let mut s = shared.lock().unwrap();
        if s.loading || matches!(s.list, Some(Ok(_))) {
            return;
        }
        s.loading = true;
        s.list = None;
    }
    let shared = shared.clone();
    std::thread::spawn(move || {
        let cache = libs::data_dir().join("google-fonts-list.json");
        let fresh = std::fs::metadata(&cache)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age < LIST_MAX_AGE);
        let mut result = if fresh { std::fs::read_to_string(&cache).map_err(|e| e.to_string()).and_then(|t| parse_list(&t)) } else { Err(String::new()) };
        if result.is_err() {
            result = get_text(LIST_URL).and_then(|text| {
                    let parsed = parse_list(&text);
                    if parsed.is_ok() {
                        let _ = std::fs::create_dir_all(libs::data_dir());
                        let _ = std::fs::write(&cache, &text);
                    }
                    parsed
                });
        }
        let mut s = shared.lock().unwrap();
        s.list = Some(result);
        s.loading = false;
    });
}

/// The font files for a family: Google's stylesheet lists one .ttf address per style.
fn download_family(f: &GFamily) -> Result<usize, String> {
    let mut pairs: Vec<(u8, u16)> = f.styles.iter().map(|s| style_key(s)).collect();
    pairs.sort();
    pairs.dedup();
    let spec = pairs.iter().map(|(i, w)| format!("{i},{w}")).collect::<Vec<_>>().join(";");
    let url = format!("{CSS_URL}?family={}:ital,wght@{spec}", f.name.replace(' ', "+"));
    let css = get_text(&url)?;
    let out_dir = family_dir(&f.name);
    let tmp_dir = out_dir.with_extension("partial");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let file_base = f.name.replace(' ', "");
    let mut saved = 0;
    for block in css.split("@font-face").skip(1) {
        let italic = block.contains("font-style: italic");
        let weight: u16 = block
            .split("font-weight:")
            .nth(1)
            .and_then(|r| r.trim().split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(400);
        let Some(src) = block.split("url(").nth(1).and_then(|r| r.split(')').next()) else { continue };
        let name = format!("{file_base}-{}.ttf", style_name(weight, italic));
        let path = tmp_dir.join(&name);
        if path.exists() {
            continue; // same style listed twice
        }
        let bytes = get_bytes(src)?;
        if ttf_parser::Face::parse(&bytes, 0).is_err() {
            return Err(format!("{name} isn't a font"));
        }
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        saved += 1;
    }
    if saved == 0 {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err("no styles".into());
    }
    // Only a complete family replaces what was there.
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::rename(&tmp_dir, &out_dir).map_err(|e| e.to_string())?;
    Ok(saved)
}

/// The file of a family's regular style (or its first one), for the hover preview.
fn fetch_preview(f: &GFamily) -> Result<Vec<u8>, String> {
    let mut pairs: Vec<(u8, u16)> = f.styles.iter().map(|s| style_key(s)).collect();
    pairs.sort_by_key(|(i, w)| (*i, (*w as i32 - 400).abs()));
    let (i, w) = pairs.first().copied().unwrap_or((0, 400));
    let css = get_text(&format!("{CSS_URL}?family={}:ital,wght@{i},{w}", f.name.replace(' ', "+")))?;
    let src = css.split("url(").nth(1).and_then(|r| r.split(')').next()).ok_or("no file")?;
    let bytes = get_bytes(src)?;
    ttf_parser::Face::parse(&bytes, 0).map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// Starts fetching a family's hover preview (once).
pub fn start_preview(shared: &Arc<Mutex<Shared>>, f: &GFamily) {
    {
        let mut s = shared.lock().unwrap();
        if s.previews.contains_key(&f.name) {
            return;
        }
        s.previews.insert(f.name.clone(), None);
    }
    let (shared, f) = (shared.clone(), f.clone());
    std::thread::spawn(move || {
        let bytes = fetch_preview(&f).ok();
        let mut s = shared.lock().unwrap();
        match bytes {
            Some(b) => {
                s.previews.insert(f.name.clone(), Some(b));
            }
            None => {
                s.previews.remove(&f.name);
            }
        }
    });
}

/// "Download All": every family given that isn't downloaded yet, four at a time.
pub fn start_download_all(shared: &Arc<Mutex<Shared>>, families: Vec<GFamily>) {
    let todo: Vec<GFamily> = families.into_iter().filter(|f| !is_downloaded(&f.name)).collect();
    if todo.is_empty() {
        return;
    }
    let total = todo.len();
    {
        let mut s = shared.lock().unwrap();
        s.bulk = Some((0, total));
        for f in &todo {
            s.downloads.insert(f.name.clone(), Dl::Busy);
        }
    }
    let queue = Arc::new(Mutex::new(todo));
    let alive = Arc::new(std::sync::atomic::AtomicUsize::new(4));
    for _ in 0..4 {
        let (queue, shared, alive) = (queue.clone(), shared.clone(), alive.clone());
        std::thread::spawn(move || {
            loop {
                let Some(f) = queue.lock().unwrap().pop() else { break };
                let ok = download_family(&f).is_ok();
                let mut s = shared.lock().unwrap();
                s.downloads.insert(f.name.clone(), if ok { Dl::Done } else { Dl::Failed });
                if let Some((done, _)) = s.bulk.as_mut() {
                    *done += 1;
                    // Read the library again now and then, not after every family.
                    if *done % 10 == 0 {
                        s.changed = true;
                    }
                }
            }
            if alive.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 1 {
                let mut s = shared.lock().unwrap();
                s.bulk = None;
                s.changed = true;
            }
        });
    }
}

pub fn start_download(shared: &Arc<Mutex<Shared>>, f: GFamily) {
    shared.lock().unwrap().downloads.insert(f.name.clone(), Dl::Busy);
    let shared = shared.clone();
    std::thread::spawn(move || {
        let r = download_family(&f);
        if let Err(e) = &r {
            eprintln!("Google Fonts download of {} failed: {e}", f.name);
        }
        let mut s = shared.lock().unwrap();
        s.downloads.insert(f.name.clone(), if r.is_ok() { Dl::Done } else { Dl::Failed });
        s.changed = true;
    });
}

/// Small "G" mark for Google Fonts in the sidebar.
pub fn paint_mark(p: &egui::Painter, c: egui::Pos2, col: Color32) {
    p.circle_stroke(c, 6.5, Stroke::new(1.4_f32, col));
    p.text(c + Vec2::new(0.2, 0.5), Align2::CENTER_CENTER, "G", FontId::proportional(10.5), col);
}

impl App {
    /// Earlier versions kept downloads as a "Google Fonts" library; now Google Fonts is a source,
    /// so that library entry is dropped (the files stay where they are).
    pub(crate) fn google_migrate(&mut self) {
        let path = dir();
        let before = self.libraries.len();
        self.libraries.retain(|l| l.path != path);
        if self.libraries.len() != before {
            self.save_libraries();
            self.rescan();
        }
    }

    /// After a download: read the fonts again so the new family appears under Sources › Google Fonts.
    pub(crate) fn google_sync(&mut self, ctx: &egui::Context) {
        let (changed, busy) = {
            let mut s = self.gf.shared.lock().unwrap();
            let c = s.changed;
            s.changed = false;
            (c, s.loading || s.downloads.values().any(|d| *d == Dl::Busy))
        };
        if busy {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
        if !changed {
            return;
        }
        // Read in the background: the window keeps answering while the new family is found.
        self.rescan_soon();
    }

    /// The Google Fonts window: search, categories, and a Download button per family.
    pub(crate) fn google_window(&mut self, ctx: &egui::Context) {
        if !self.gf.open {
            return;
        }
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let mut open = true;
        let mut start: Option<GFamily> = None;
        let mut show: Option<String> = None;
        let mut retry_list = false;
        let mut hovered: Option<GFamily> = None;
        // The hovered family's preview (when ready), the dialog's left edge and the row's height on screen.
        let mut float: Option<(Option<egui::TextureHandle>, f32, f32)> = None;
        let mut download_all: Option<Vec<GFamily>> = None;
        let cats: [(Option<&'static str>, &str); 6] = [
            (None, t(K::CatAll)),
            (Some("Serif"), t(K::SerifList)),
            (Some("Sans Serif"), t(K::SansList)),
            (Some("Display"), t(K::CatDisplay)),
            (Some("Handwriting"), t(K::CatHandwriting)),
            (Some("Monospace"), t(K::Monospaced)),
        ];
        // Its own system window; the family list scrolls inside it.
        let (_, closed) = crate::oslook::dialog(ctx, "google-fonts", t(K::GoogleFonts), 640.0, pal.panel, Some(590.0), |ui| {
                ui.label(egui::RichText::new(t(K::GoogleHint)).size(12.5).color(pal.muted));
                ui.add_space(8.0);
                crate::oslook::field(ui, egui::TextEdit::singleline(&mut self.gf.search).hint_text(format!("🔍 {}", t(K::GoogleSearch))).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 5)));
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for (c, label) in cats {
                        if ui.selectable_label(self.gf.category == c, label).clicked() {
                            self.gf.category = c;
                        }
                    }
                });
                ui.add_space(6.0);
                let s = self.gf.shared.lock().unwrap();
                let downloads = s.downloads.clone();
                match &s.list {
                    None => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new(t(K::LoadingList)).color(pal.muted));
                        });
                    }
                    Some(Err(_)) => {
                        ui.label(egui::RichText::new(t(K::ListFailed)).color(pal.warn));
                        if ui.button(t(K::Retry)).clicked() {
                            retry_list = true;
                        }
                    }
                    Some(Ok(list)) => {
                        let q = self.gf.search.to_lowercase();
                        let items: Vec<&GFamily> = list
                            .iter()
                            .filter(|f| self.gf.category.is_none_or(|c| f.category == c))
                            .filter(|f| q.is_empty() || f.name.to_lowercase().contains(&q))
                            .collect();
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t(K::FamiliesN).replace("{n}", &items.len().to_string())).size(12.0).color(pal.muted));
                            // Every family shown (all of them, or the ones the search and category leave).
                            if let Some((done, total)) = s.bulk {
                                ui.spinner();
                                ui.label(egui::RichText::new(t(K::DownloadingAll).replace("{n}", &done.to_string()).replace("{m}", &total.to_string())).size(12.0).color(pal.muted));
                            } else if ui.button(t(K::DownloadAll)).clicked() {
                                download_all = Some(items.iter().map(|f| (*f).clone()).collect::<Vec<_>>());
                            }
                        });
                        let row_h = 44.0;
                        egui::ScrollArea::vertical().id_salt("gf-list").max_height(380.0).auto_shrink([false, false]).show_rows(ui, row_h, items.len(), |ui, range| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for f in &items[range] {
                                let w = ui.available_width();
                                let (r, row_resp) = ui.allocate_exact_size(Vec2::new(w, row_h), Sense::hover());
                                // Pointer resting on a family: its letters at 48 points (fetched once, even
                                // before the family is downloaded).
                                // It shows in a small window outside this one, on its left, so the list
                                // stays in sight.
                                if row_resp.hovered() {
                                    hovered = Some((*f).clone());
                                    let (left, top) = crate::oslook::dialog_origin(ui);
                                    float = Some((self.gf.preview_tex.get(&f.name).cloned(), left, top + r.center().y));
                                }
                                ui.painter().line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0_f32, pal.line));
                                ui.painter().text(r.left_top() + Vec2::new(4.0, 13.0), Align2::LEFT_CENTER, &f.name, FontId::proportional(15.0), pal.ink);
                                let cat = match f.category.as_str() {
                                    "Serif" => t(K::SerifList),
                                    "Sans Serif" => t(K::SansList),
                                    "Display" => t(K::CatDisplay),
                                    "Handwriting" => t(K::CatHandwriting),
                                    "Monospace" => t(K::Monospaced),
                                    other => other,
                                };
                                let styles = if f.styles.len() == 1 { t(K::Style1).to_string() } else { t(K::StylesN).replace("{n}", &f.styles.len().to_string()) };
                                ui.painter().text(r.left_top() + Vec2::new(4.0, 31.0), Align2::LEFT_CENTER, format!("{cat} · {styles}"), FontId::proportional(12.0), pal.muted);
                                let state = downloads.get(&f.name).cloned().or_else(|| is_downloaded(&f.name).then_some(Dl::Done));
                                let btn_rect = egui::Rect::from_min_size(egui::pos2(r.right() - 150.0, r.center().y - 13.0), Vec2::new(144.0, 26.0));
                                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(btn_rect).layout(egui::Layout::right_to_left(egui::Align::Center)));
                                match state {
                                    Some(Dl::Busy) => {
                                        child.label(egui::RichText::new(t(K::Downloading)).size(12.5).color(pal.muted));
                                        child.spinner();
                                    }
                                    Some(Dl::Done) => {
                                        if child.button(t(K::ShowInList)).clicked() {
                                            show = Some(f.name.clone());
                                        }
                                        child.label(egui::RichText::new(format!("✔ {}", t(K::Downloaded))).size(12.5).color(crate::libs::LIME));
                                    }
                                    Some(Dl::Failed) => {
                                        if child.button(egui::RichText::new(t(K::DownloadFailed)).size(12.0).color(pal.warn)).clicked() {
                                            start = Some((*f).clone());
                                        }
                                    }
                                    None => {
                                        let b = egui::Button::new(egui::RichText::new(t(K::Download)).color(Color32::WHITE)).fill(pal.accent);
                                        if child.add(b).clicked() {
                                            start = Some((*f).clone());
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            });
        if closed {
            open = false;
        }
        if let Some((tex, left, y)) = float.filter(|_| !closed) {
            let ppp = ctx.pixels_per_point();
            let size = tex.as_ref().map(|t| t.size_vec2() / ppp).unwrap_or(Vec2::new(220.0, 24.0)) + Vec2::splat(16.0);
            let ink = pal.ink;
            let loading = t(K::LoadingPreview);
            crate::oslook::float_beside(ctx, "gf-preview", left, y, size, pal.panel, |ui| match &tex {
                Some(tex) => {
                    ui.add(egui::Image::new((tex.id(), tex.size_vec2() / ppp)).tint(ink));
                }
                None => {
                    ui.label(egui::RichText::new(loading).color(ink));
                }
            });
        }
        if let Some(list) = download_all {
            start_download_all(&self.gf.shared, list);
        }
        // The hover preview's picture: from the downloaded family, or fetched for the preview.
        if let Some(f) = hovered {
            if !self.gf.preview_tex.contains_key(&f.name) {
                let local = is_downloaded(&f.name)
                    .then(|| std::fs::read_dir(family_dir(&f.name)).ok())
                    .flatten()
                    .and_then(|rd| rd.flatten().map(|e| e.path()).find(|p| p.extension().is_some_and(|x| x == "ttf")))
                    .and_then(|p| std::fs::read(p).ok());
                let bytes = local.or_else(|| {
                    start_preview(&self.gf.shared, &f);
                    self.gf.shared.lock().unwrap().previews.get(&f.name).cloned().flatten()
                });
                match bytes {
                    Some(b) => {
                        let ppp = ctx.pixels_per_point();
                        // Every letter, figure and the usual accents and marks, in lines short enough for 60 pt.
                        let text = "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ\nabcdefghijklm\nnopqrstuvwxyz\n0123456789\nÁÉÍÓÚÑ áéíóúñü\n.,;:¿?¡!&@#%()";
                        if let Some(r) = crate::fonts::render(&b, 0, text, PREVIEW_PT * ppp, (720.0 * ppp) as usize, &[], true) {
                            let pixels = r.alpha.iter().map(|&a| Color32::from_white_alpha(a)).collect();
                            let img = egui::ColorImage::new([r.w, r.h], pixels);
                            if self.gf.preview_tex.len() > 200 {
                                self.gf.preview_tex.clear();
                            }
                            let tex = ctx.load_texture(format!("gf-preview-{}", f.name), img, egui::TextureOptions::LINEAR);
                            self.gf.preview_tex.insert(f.name.clone(), tex);
                        }
                    }
                    None => ctx.request_repaint_after(Duration::from_millis(200)),
                }
            }
        }
        if self.gf.shared.lock().unwrap().bulk.is_some() {
            ctx.request_repaint_after(Duration::from_millis(300));
        }
        if retry_list {
            self.gf.shared.lock().unwrap().list = None;
            start_list(&self.gf.shared);
        }
        if let Some(f) = start {
            start_download(&self.gf.shared, f);
        }
        if let Some(name) = show {
            // Show the family under Sources › Google Fonts, and select it.
            self.view = View::Source(crate::fonts::Source::Google);
            self.selected = self.lib.families.iter().position(|f| f.name == name).map(crate::Row::Family).or_else(|| self.rows().first().copied());
            open = false;
        }
        self.gf.open = open;
    }
}

/// Lets the automatic checks download one family without the window (`--google-download <name>`).
pub fn download_now(name: &str) -> Result<usize, String> {
    let text = get_text(LIST_URL)?;
    let list = parse_list(&text)?;
    let f = list.into_iter().find(|f| f.name == name).ok_or("no such family")?;
    download_family(&f)
}
