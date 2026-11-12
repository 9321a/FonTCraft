//! The app's settings: what's stored, the colours for light and dark mode, and small painted icons.

use crate::fonts::Source;
use eframe::egui::{self, Color32, Pos2, CornerRadius, Stroke, Vec2};
use crate::mechanics;
use std::path::PathBuf;

/// Ubuntu's ten accent colours (Yaru, Settings › Appearance), in its order, offered on every system:
/// (name, colour on light panels, colour on dark panels — the same colour lifted 22 % towards white
/// so it reads on the dark background).
pub const ACCENTS: [(crate::i18n::K, Color32, Color32); 10] = {
    use crate::i18n::K;
    const fn hex(v: u32) -> Color32 {
        Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }
    [
        (K::AccOrange, hex(0xe95420), hex(0xee7a51)),
        (K::AccBark, hex(0x787859), hex(0x96967e)),
        (K::AccSage, hex(0x657b69), hex(0x87988a)),
        (K::AccOlive, hex(0x4b8501), hex(0x73a039)),
        (K::AccViridian, hex(0x03875b), hex(0x3aa17f)),
        (K::AccPrussian, hex(0x308280), hex(0x5e9e9c)),
        (K::AccUbBlue, hex(0x0073e5), hex(0x3892eb)),
        (K::AccUbPurple, hex(0x7764d8), hex(0x9586e1)),
        (K::AccMagenta, hex(0xb34cb3), hex(0xc473c4)),
        (K::AccUbRed, hex(0xda3450), hex(0xe26176)),
    ]
};

/// `accent` value meaning "the system's accent colour" (the default), after Ubuntu's ten.
pub const SYSTEM_ACCENT: usize = 10;

/// The accent for light (`dark` false) or dark panels: a swatch, or the system's own colour (on
/// dark panels lifted a little, as the systems do, so it reads on the dark background).
pub fn accent_color(accent: usize, dark: bool) -> Color32 {
    if accent >= ACCENTS.len() {
        let sys = crate::oslook::system_accent().unwrap_or(ACCENTS[6].1);
        return if dark && !cfg!(target_os = "macos") { mix(sys, Color32::WHITE, 0.2) } else { sys };
    }
    let a = ACCENTS[accent];
    if dark { a.2 } else { a.1 }
}

/// The background of a sidebar row, as RightFont's -[RFNavigationItemRowView drawRect:]
/// (0x1000c61c4) paints it: the chosen row black at 6 % (0x1002822a8) on light panels, white at 9 %
/// (0x100282750) on dark ones; under the pointer black at 2.8 % (0x100282748) / white at 4 %
/// (0x1002822a0). The accent colour is kept for the text and icon of the chosen row.
pub fn row_tint(dark: bool, selected: bool, hovered: bool) -> Option<Color32> {
    let a = match (selected, hovered, dark) {
        (true, _, false) => 0.06,
        (true, _, true) => 0.09,
        (false, true, false) => 0.028,
        (false, true, true) => 0.04,
        _ => return None,
    };
    Some(if dark { Color32::WHITE } else { Color32::BLACK }.gamma_multiply(a))
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    Appearance,
    Library,
    AutoActivation,
}

#[derive(Clone, PartialEq)]
pub struct Settings {
    /// Index into the list of languages the app's words come in.
    pub lang: usize,
    pub theme: ThemeChoice,
    pub accent: usize,
    /// 0 = compact, 1 = regular, 2 = roomy
    pub density: usize,
    pub default_preview: String,
    pub default_size: f32,
    pub show_style_count: bool,
    pub show_details: bool,
    pub show_waterfall: bool,
    pub warn_missing: bool,
    pub sources: [bool; 3],
    /// Start FontCraft when the person logs in (a login item that starts it in the background).
    pub launch_at_login: bool,
    /// Keep running in the menu bar when the window is closed.
    pub background: bool,
    /// Keyboard shortcut that brings FontCraft to the front, e.g. "Cmd+Shift+F" (empty = none).
    pub hotkey: String,
    pub show_google: bool,
    pub show_adobe: bool,
    /// Sidebar's Favorites item and Smart Lists section (RightFont `ShowStarredFonts` /
    /// `ShowSmartFilters`, both on by default: 0x1000a9a14, 0x1000a9f54).
    pub show_favorites: bool,
    pub show_smart: bool,
    /// Families open (styles showing) in the list, by name, and folded libraries in the sidebar, by
    /// path — kept between runs like RightFont's ExpandedFontFamilies (restoreExpandedState, 0x1001082a0).
    pub expanded: Vec<String>,
    /// FontCraft already asked the system for the permissions it needs (first start).
    pub asked_access: bool,
    pub collapsed: Vec<String>,
    /// Who ~/FontCraft was last shared with ("all", "user:<name>", or empty = not shared).
    pub share: String,
    /// Apps (bundle ids) the person switched off in Auto Activation.
    pub apps_off: Vec<String>,
    pub act_family: bool,
    pub silent: bool,
    pub deact_auto_on_quit: bool,
    pub deact_manual_on_quit: bool,
    /// Widths of the side panels, in points.
    pub left_w: f32,
    pub right_w: f32,
}

impl Default for Settings {
    fn default() -> Self {
        let lang = system_language();
        Settings {
            lang,
            theme: ThemeChoice::System,
            accent: SYSTEM_ACCENT,
            density: 1,
            default_preview: crate::i18n::tr(lang, crate::i18n::K::Pangram).into(),
            // RightFont: list preview size 16 when never set (-[RFPreferencesHandler fontPreviewListViewFontSize], 0x1000aa540).
            default_size: mechanics::LIST_PREVIEW_SIZE,
            show_style_count: true,
            show_details: true,
            show_waterfall: true,
            warn_missing: true,
            sources: [true; 3],
            launch_at_login: false,
            background: false,
            hotkey: String::new(),
            show_google: true,
            show_adobe: true,
            show_favorites: true,
            show_smart: true,
            share: String::new(),
            expanded: Vec::new(),
            asked_access: false,
            collapsed: Vec::new(),
            apps_off: Vec::new(),
            act_family: false,
            silent: false,
            // RightFont's defaults: AutoDeactivateAutoActivatedFonts off (0x1000a9518),
            // AutoActivateFontFamily off (0x1000a9470), AutoDeactivateManuallyActivatedFonts off (0x1000a95c0).
            deact_auto_on_quit: false,
            deact_manual_on_quit: false,
            left_w: 210.0,
            right_w: 400.0,
        }
    }
}

fn settings_path() -> PathBuf {
    crate::paths::data_dir().join("settings.txt")
}

/// The computer's language, when it is one the app offers; English otherwise.
fn system_language() -> usize {
    #[allow(unused_mut)]
    let mut wanted = std::env::var("LANG").unwrap_or_default();
    // Windows: the user's language from the system (there is no LANG there).
    #[cfg(target_os = "windows")]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultLocaleName(buf: *mut u16, len: i32) -> i32;
        }
        let mut buf = [0u16; 85];
        let n = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
        if n > 1 {
            wanted = String::from_utf16_lossy(&buf[..(n - 1) as usize]);
        }
    }
    #[cfg(target_os = "macos")]
    if let Ok(out) = crate::paths::command("defaults").args(["read", "-g", "AppleLanguages"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(first) = text.split('"').nth(1) {
            wanted = first.to_string();
        }
    }
    language_from_code(&wanted)
}

fn language_from_code(code: &str) -> usize {
    use crate::i18n::{index_of, LANGS};
    let code = code.split('.').next().unwrap_or("").replace('_', "-");
    if code.starts_with("zh") {
        let trad = code.contains("Hant") || code.ends_with("TW") || code.ends_with("HK") || code.ends_with("MO");
        return index_of(if trad { "zh-Hant" } else { "zh-Hans" });
    }
    let base = code.split('-').next().unwrap_or("");
    let base = if base == "no" || base == "nn" { "nb" } else { base };
    LANGS.iter().position(|l| l.code == base).unwrap_or(0)
}

fn yes(v: &str) -> bool {
    v == "1" || v == "true"
}

impl Settings {
    pub fn load() -> Settings {
        let mut s = Settings::default();
        if let Ok(text) = std::fs::read_to_string(settings_path()) {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                match k {
                    "lang" => s.lang = crate::i18n::index_of(v),
                    "theme" => {
                        s.theme = match v {
                            "light" => ThemeChoice::Light,
                            "dark" => ThemeChoice::Dark,
                            _ => ThemeChoice::System,
                        }
                    }
                    // "accent3": "system" or one of Ubuntu's ten. Older keys ("accent", "accent2") named
                    // VectorCraft's six swatches, which are gone, so they fall back to the system's colour.
                    "accent3" => s.accent = if v == "system" { SYSTEM_ACCENT } else { v.parse().unwrap_or(SYSTEM_ACCENT).min(SYSTEM_ACCENT) },
                    "density" => s.density = v.parse().unwrap_or(1).min(2),
                    "preview" => s.default_preview = v.to_string(),
                    "size" => s.default_size = v.parse().unwrap_or(mechanics::LIST_PREVIEW_SIZE).clamp(10.0, 160.0),
                    "style_count" => s.show_style_count = yes(v),
                    "details" => s.show_details = yes(v),
                    "waterfall" => s.show_waterfall = yes(v),
                    "warn_missing" => s.warn_missing = yes(v),
                    "system" => s.sources[0] = yes(v),
                    "computer" => s.sources[1] = yes(v),
                    "user" => s.sources[2] = yes(v),
                    "launch_at_login" => s.launch_at_login = yes(v),
                    "background" => s.background = yes(v),
                    "hotkey" => s.hotkey = v.to_string(),
                    "show_google" => s.show_google = yes(v),
                    "show_adobe" => s.show_adobe = yes(v),
                    "show_favorites" => s.show_favorites = yes(v),
                    "show_smart" => s.show_smart = yes(v),
                    "share" => s.share = v.to_string(),
                    "asked_access" => s.asked_access = yes(v),
                    "expanded" => s.expanded = v.split('|').filter(|x| !x.is_empty()).map(String::from).collect(),
                    "collapsed" => s.collapsed = v.split('|').filter(|x| !x.is_empty()).map(String::from).collect(),
                    "apps_off" => s.apps_off = v.split(',').filter(|x| !x.is_empty()).map(String::from).collect(),
                    "act_family" => s.act_family = yes(v),
                    "silent" => s.silent = yes(v),
                    "deact_auto" => s.deact_auto_on_quit = yes(v),
                    "deact_manual" => s.deact_manual_on_quit = yes(v),
                    "left_w" => s.left_w = v.parse().unwrap_or(210.0f32).clamp(150.0, 480.0),
                    "right_w" => s.right_w = v.parse().unwrap_or(400.0f32).clamp(260.0, 1000.0),
                    _ => {}
                }
            }
        }
        if crate::i18n::is_sample(&s.default_preview) {
            s.default_preview = crate::i18n::tr(s.lang, crate::i18n::K::Pangram).into();
        }
        // Lets the automatic checks look at other languages and dark mode without touching saved settings.
        if let Ok(code) = std::env::var("FONTCRAFT_LANG") {
            s.lang = language_from_code(&code);
            if crate::i18n::is_sample(&s.default_preview) {
                s.default_preview = crate::i18n::tr(s.lang, crate::i18n::K::Pangram).into();
            }
        }
        match std::env::var("FONT_SHELF_THEME").as_deref() {
            Ok("dark") => s.theme = ThemeChoice::Dark,
            Ok("light") => s.theme = ThemeChoice::Light,
            _ => {}
        }
        s
    }

    pub fn save(&self) {
        let theme = match self.theme {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        };
        let b = |x: bool| if x { "1" } else { "0" };
        let text = format!(
            "lang={}\ntheme={theme}\naccent3={}\ndensity={}\npreview={}\nsize={}\nstyle_count={}\ndetails={}\nwaterfall={}\nwarn_missing={}\nsystem={}\ncomputer={}\nuser={}\n",
            crate::i18n::LANGS[self.lang].code,
            if self.accent >= SYSTEM_ACCENT { "system".to_string() } else { self.accent.to_string() },
            self.density,
            self.default_preview.replace('\n', " "),
            self.default_size.round(),
            b(self.show_style_count),
            b(self.show_details),
            b(self.show_waterfall),
            b(self.warn_missing),
            b(self.sources[0]),
            b(self.sources[1]),
            b(self.sources[2]),
        ) + &format!(
            "launch_at_login={}\nbackground={}\nhotkey={}\nshow_google={}\nshow_adobe={}\nshow_favorites={}\nshow_smart={}\nshare={}\nasked_access={}\nexpanded={}\ncollapsed={}\napps_off={}\nact_family={}\nsilent={}\ndeact_auto={}\ndeact_manual={}\nleft_w={}\nright_w={}\n",
            b(self.launch_at_login),
            b(self.background),
            self.hotkey,
            b(self.show_google),
            b(self.show_adobe),
            b(self.show_favorites),
            b(self.show_smart),
            self.share,
            b(self.asked_access),
            self.expanded.join("|").replace('\n', " "),
            self.collapsed.join("|").replace('\n', " "),
            self.apps_off.join(","),
            b(self.act_family),
            b(self.silent),
            b(self.deact_auto_on_quit),
            b(self.deact_manual_on_quit),
            self.left_w.round(),
            self.right_w.round(),
        );
        let p = settings_path();
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, text);
    }

    pub fn source_enabled(&self, s: Source) -> bool {
        match s {
            Source::System => self.sources[0],
            Source::Computer => self.sources[1],
            Source::User => self.sources[2],
            Source::Library => true,
            Source::Google => self.show_google,
            Source::Adobe => self.show_adobe,
        }
    }

    pub fn row_padding(&self) -> f32 {
        [28.0, 40.0, 56.0][self.density]
    }

    pub fn theme_preference(&self) -> egui::ThemePreference {
        match self.theme {
            ThemeChoice::System => egui::ThemePreference::System,
            ThemeChoice::Light => egui::ThemePreference::Light,
            ThemeChoice::Dark => egui::ThemePreference::Dark,
        }
    }
}

/// Every colour the window uses, for one theme.
#[derive(Clone, Copy, PartialEq)]
pub struct Pal {
    pub dark: bool,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub line: Color32,
    pub sidebar: Color32,
    pub panel: Color32,
    pub status: Color32,
    pub hover: Color32,
    pub face_bg: Color32,
    pub card: Color32,
    pub warn: Color32,
}

/// A colour with transparency laid over `bg` (an opaque result).
fn over(c: Color32, bg: Color32) -> Color32 {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    let t = a as f32 / 255.0;
    let m = |x: u8, y: u8| (y as f32 + (x as f32 - y as f32) * t).round() as u8;
    Color32::from_rgb(m(r, bg.r()), m(g, bg.g()), m(b, bg.b()))
}

/// An opaque colour (a translucent one laid over `bg`).
fn opaque(c: Color32, bg: Color32) -> Color32 {
    over(c, bg)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

impl Pal {
    pub fn light(accent: usize) -> Pal {
        Pal::from_system(accent, false)
    }

    pub fn dark(accent: usize) -> Pal {
        Pal::from_system(accent, true)
    }

    /// The host system's own colours for its light or dark look (oslook::system_colors: AppKit's
    /// named colours on macOS, Windows 11 Fluent, GNOME libadwaita), with the accent.
    fn from_system(accent: usize, dark: bool) -> Pal {
        let sys = crate::oslook::system_colors(dark);
        let a = accent_color(accent, dark);
        let panel = opaque(sys.content, Color32::BLACK);
        // Text and lines with transparency, flattened over the content background.
        let ink = over(sys.label, panel);
        let muted = over(sys.secondary, panel);
        let line = over(sys.separator, panel);
        let sidebar = opaque(sys.sidebar, panel);
        let status = opaque(sys.window, panel);
        Pal {
            dark,
            accent: a,
            accent_soft: mix(panel, a, if dark { 0.28 } else { 0.14 }),
            ink,
            muted,
            line,
            sidebar,
            panel,
            status,
            hover: mix(panel, ink, if dark { 0.08 } else { 0.05 }),
            face_bg: mix(panel, ink, 0.02),
            card: mix(panel, ink, if dark { 0.06 } else { 0.035 }),
            warn: if dark { Color32::from_rgb(0xe8, 0x9a, 0x5a) } else { Color32::from_rgb(0xc0, 0x6a, 0x2b) },
        }
    }

    pub fn visuals(&self) -> egui::Visuals {
        let mut v = if self.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
        v.panel_fill = self.panel;
        v.window_fill = self.panel;
        v.window_stroke = Stroke::new(1.0_f32, self.line);
        v.window_corner_radius = CornerRadius::same(12);
        v.extreme_bg_color = if self.dark { Color32::from_rgb(0x2a, 0x2a, 0x30) } else { Color32::from_rgb(0xf1, 0xf1, 0xf5) };
        v.faint_bg_color = self.card;
        v.selection.bg_fill = self.accent;
        v.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
        v.hyperlink_color = self.accent;
        v.override_text_color = None;
        for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
            w.corner_radius = CornerRadius::same(6);
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, self.line);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, self.ink);
        v.widgets.inactive.weak_bg_fill = self.card;
        v.widgets.inactive.bg_fill = self.card;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, self.ink);
        v.widgets.hovered.weak_bg_fill = self.hover;
        v.widgets.hovered.bg_fill = self.hover;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.line);
        v.widgets.active.weak_bg_fill = self.accent_soft;
        v.widgets.active.bg_fill = self.accent;
        // Buttons, pop-up menus, fields and shadows like the system's.
        crate::oslook::apply_visuals(&mut v, self.dark);
        v
    }
}

/// A cog wheel.
pub fn paint_gear(p: &egui::Painter, c: Pos2, ink: Color32, bg: Color32) {
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let d = Vec2::angled(a);
        p.line_segment([c + d * 5.0, c + d * 8.5], Stroke::new(3.0_f32, ink));
    }
    p.circle_filled(c, 6.2, ink);
    p.circle_filled(c, 2.6, bg);
}

/// A printer, drawn in the same weight as the cog wheel: paper going in at the top, the body,
/// and the printed sheet coming out.
pub fn paint_printer(p: &egui::Painter, c: Pos2, ink: Color32, bg: Color32) {
    let st = Stroke::new(1.8_f32, ink);
    // Paper in.
    p.rect_stroke(egui::Rect::from_center_size(c + Vec2::new(0.0, -6.0), Vec2::new(9.0, 6.0)), 1.0, st, eframe::egui::StrokeKind::Inside);
    // Body.
    p.rect_filled(egui::Rect::from_center_size(c + Vec2::new(0.0, 0.5), Vec2::new(18.0, 8.5)), 2.5, ink);
    // Printed sheet, over the body.
    let sheet = egui::Rect::from_center_size(c + Vec2::new(0.0, 5.0), Vec2::new(10.0, 7.5));
    p.rect_filled(sheet, 1.0, bg);
    p.rect_stroke(sheet, 1.0, st, eframe::egui::StrokeKind::Inside);
    p.line_segment([sheet.left_center() + Vec2::new(2.5, 0.5), sheet.right_center() + Vec2::new(-2.5, 0.5)], Stroke::new(1.2_f32, ink));
    // Status light.
    p.circle_filled(c + Vec2::new(6.0, -1.0), 1.1, bg);
}

/// A sun when dark mode is on (click for light), a moon otherwise.
pub fn paint_theme_icon(p: &egui::Painter, c: Pos2, dark: bool, ink: Color32, bg: Color32) {
    if dark {
        p.circle_filled(c, 4.0, ink);
        for i in 0..8 {
            let d = Vec2::angled(i as f32 * std::f32::consts::TAU / 8.0);
            p.line_segment([c + d * 6.5, c + d * 9.0], Stroke::new(1.6_f32, ink));
        }
    } else {
        p.circle_filled(c, 7.5, ink);
        p.circle_filled(c + Vec2::new(4.0, -3.5), 6.5, bg);
    }
}

/// A small drawing of the app window in a theme, used on the theme cards.
pub fn paint_mini_window(p: &egui::Painter, r: egui::Rect, pal: &Pal) {
    p.rect_filled(r, 6.0, pal.panel);
    let side = egui::Rect::from_min_size(r.min, Vec2::new(r.width() * 0.28, r.height()));
    p.rect_filled(side, CornerRadius { nw: 6, sw: 6, ne: 0, se: 0 }, pal.sidebar);
    p.rect_filled(egui::Rect::from_min_size(side.min + Vec2::new(5.0, 10.0), Vec2::new(side.width() - 10.0, 6.0)), 2.0, pal.accent_soft);
    for i in 0..3 {
        let y = r.top() + 10.0 + i as f32 * 20.0;
        let x = side.right() + 8.0;
        p.rect_filled(egui::Rect::from_min_size(Pos2::new(x, y), Vec2::new(28.0, 3.0)), 1.0, pal.muted);
        p.rect_filled(egui::Rect::from_min_size(Pos2::new(x, y + 6.0), Vec2::new(r.right() - x - 10.0, 7.0)), 2.0, pal.ink);
    }
    p.rect_stroke(r, 6.0, Stroke::new(1.0_f32, pal.line), eframe::egui::StrokeKind::Inside);
}
