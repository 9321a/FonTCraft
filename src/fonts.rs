//! Finding the fonts installed on this computer and drawing preview text with them.

use memmap2::Mmap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    System,
    Computer,
    User,
    /// A folder the person added as their own library (not installed on the Mac).
    Library,
    /// Families downloaded from Google Fonts (FontCraft's own folder; switchable like a library).
    Google,
    /// Fonts that Adobe Creative Cloud turns on (its own folder; Creative Cloud manages them).
    Adobe,
}

/// Where Adobe Creative Cloud keeps the fonts it turns on (Adobe Fonts).
pub fn adobe_dir() -> PathBuf {
    crate::paths::adobe_dir()
}

/// What kind of letters a family has, for the smart lists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// RightFont's classes (-[RFFont classificationString], 0x10009ee74, table at 0x10030f048):
/// 1 Sans Serif, 2 Serif, 3 Slab Serif, 4 Script, 5 Blackletter, 7 Display, 8 Symbols, 9 Monospaced
/// (monospaced is FontCraft's separate smart list).
pub enum Class {
    Serif,
    Sans,
    Slab,
    Script,
    Blackletter,
    Display,
    Symbol,
    /// Not a Latin text font (for example Chinese, Arabic or Devanagari only).
    Other,
}

pub struct FontFile {
    pub path: PathBuf,
    pub data: Arc<Mmap>,
    pub source: Source,
}

pub struct Face {
    pub file: usize,
    pub index: u32,
    pub family: String,
    pub style: String,
    pub weight: u16,
    pub italic: bool,
    pub monospaced: bool,
    pub glyphs: u16,
}

pub struct Family {
    pub name: String,
    pub faces: Vec<usize>,
    pub default_face: usize,
    pub source: Source,
    pub monospaced: bool,
    pub class: Class,
    /// Which of the person's libraries this family is in.
    pub libs: Vec<usize>,
}

#[derive(Default)]
pub struct Library {
    pub files: Vec<FontFile>,
    pub faces: Vec<Face>,
    pub families: Vec<Family>,
}

/// Counts the drawing steps in a letter's outline (a plain sans "I" is a simple box).
struct Steps(usize);
impl ttf_parser::OutlineBuilder for Steps {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {}
}

fn has_word(name: &str, words: &[&str]) -> bool {
    words.iter().any(|w| name.contains(w))
}

/// Sorts a font into serif, sans-serif, script or symbols, using the style hints the font
/// carries (OS/2 family class and PANOSE), its name, and as a last resort the shape of its "I".
pub fn classify(face: &ttf_parser::Face, family: &str) -> Class {
    let name = family.to_lowercase();
    let os2 = face.raw_face().table(ttf_parser::Tag::from_bytes(b"OS/2"));
    let (class_hi, panose) = match os2 {
        Some(t) if t.len() >= 42 => (t[30], [t[32], t[33]]),
        _ => (0, [0, 0]),
    };
    let mapped = |c: char| face.glyph_index(c).map(|g| g.0 != 0).unwrap_or(false);
    let latin = mapped('A') && mapped('a');
    let mut symbol_cmap = false;
    let mut symbol_chars = 0usize;
    let mut all_chars = 0usize;
    let mut letter_chars = 0usize;
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables {
            if st.platform_id == ttf_parser::PlatformId::Windows && st.encoding_id == 0 {
                symbol_cmap = true;
            }
            if st.is_unicode() {
                // Fonts can hold several character tables; the biggest one tells the most.
                let (mut all, mut letters, mut syms) = (0usize, 0usize, 0usize);
                st.codepoints(|cp| {
                    all += 1;
                    if (0x0370..=0x1FFF).contains(&cp) || (0x3040..=0x9FFF).contains(&cp) || (0xAC00..=0xD7AF).contains(&cp) || (0xFB50..=0xFEFF).contains(&cp) {
                        letters += 1;
                    }
                    if (0x2190..=0x2BFF).contains(&cp) || (0x1F300..=0x1FAFF).contains(&cp) || (0xE000..=0xF8FF).contains(&cp) {
                        syms += 1;
                    }
                });
                if all > all_chars {
                    (all_chars, letter_chars, symbol_chars) = (all, letters, syms);
                }
            }
        }
    }
    const SYMBOL: &[&str] = &["symbol", "sym", "dingbat", "wingding", "webding", "icon", "emoji", "ornament", "pictograph", "awesome"];
    const SCRIPT: &[&str] = &[
        "script", "hand", "brush", "chancery", "calligraph", "zapfino", "snell", "signature", "cursive", "savoye", "signpainter",
        "noteworthy", "bradley", "marker", "party", "edwardian", "vivaldi", "kunstler", "mistral",
    ];
    const SANS: &[&str] = &[
        "sans", " mono", "gothic", "grotesk", "grotesque", "helvetica", "arial", "avenir", "futura", "gill", "verdana", "tahoma",
        "trebuchet", "geneva", "lucida grande", "myriad", "frutiger", "din ", "roboto", "inter", "optima", "menlo", "monaco",
        "andale", "skia", "impact", "chalkboard", "phosphate", "krungthep", "seravek", "proxima", "montserrat", "poppins",
    ];
    const SERIF: &[&str] = &[
        "serif", "times", "roman", "garamond", "baskerville", "georgia", "didot", "caslon", "bodoni", "palatino", "antiqua",
        "cochin", "hoefler", "charter", "iowan", "athelas", "new york", "century", "courier", "typewriter",
        "plantagenet", "bookman", "cambria", "constantia", "minion", "trattatello", "engraved",
    ];
    // Symbol hints count unless the font plainly holds another alphabet (old Arabic fonts carry them too).
    // The font's own list of alphabets it supports (OS/2 Unicode ranges): Greek, Cyrillic, Hebrew,
    // Arabic, Indic, Thai and so on count as real letters even when they're reached indirectly.
    if let Some(t) = os2 {
        if t.len() >= 46 {
            let r1 = u32::from_be_bytes([t[42], t[43], t[44], t[45]]);
            if r1 & 0xFFFF_FF80 != 0 {
                letter_chars += 100;
            }
        }
    }
    let mostly_symbols = !latin && letter_chars < 20 && symbol_chars >= 20 && symbol_chars * 2 > all_chars;
    let symbol_hint = class_hi == 12 || panose[0] == 5 || symbol_cmap;
    if has_word(&name, SYMBOL) || mostly_symbols || (symbol_hint && (latin || letter_chars < 20)) {
        return Class::Symbol;
    }
    if !latin {
        return Class::Other;
    }
    // RightFont's other classes: blackletter, display (decorative), slab serif.
    const BLACKLETTER: &[&str] = &[
        "fraktur", "blackletter", "black letter", "textura", "old english", "schwabacher", "rotunda", "goudy text",
        "cloister black", "unifraktur", "gothic text", "olde english", "engravers old",
    ];
    const DISPLAY: &[&str] = &[
        "display", "decorative", "stencil", "poster", "shadow", "inline", "outline", "headline", "western", "circus",
        "comic", "graffiti", "neon", "3d", "playbill", "bauhaus", "broadway", "chalkduster", "papyrus", "herculanum",
    ];
    const SLAB: &[&str] = &["slab", "rockwell", "clarendon", "egyptian", "memphis", "arvo", "zilla", "museo slab", "american typewriter", "americantypewriter"];
    if has_word(&name, BLACKLETTER) {
        return Class::Blackletter;
    }
    if has_word(&name, SCRIPT) || class_hi == 10 || panose[0] == 3 {
        return Class::Script;
    }
    // PANOSE "Latin Decorative" or IBM class 9 "Ornamentals".
    if has_word(&name, DISPLAY) || panose[0] == 4 || class_hi == 9 {
        return Class::Display;
    }
    // IBM class 5 "Slab Serifs" (class 4 "Clarendon" also covers fonts like Georgia, so only by name).
    if has_word(&name, SLAB) || class_hi == 5 {
        return Class::Slab;
    }
    if has_word(&name, SANS) {
        return Class::Sans;
    }
    if has_word(&name, SERIF) {
        return Class::Serif;
    }
    if class_hi == 8 || (panose[0] == 2 && (11..=13).contains(&panose[1])) {
        return Class::Sans;
    }
    if matches!(class_hi, 1..=4 | 7) || (panose[0] == 2 && (2..=10).contains(&panose[1])) {
        return Class::Serif;
    }
    // No hints at all: a sans-serif capital I is a plain box; serifs add many more steps.
    let mut steps = Steps(0);
    match face.glyph_index('I').and_then(|g| face.outline_glyph(g, &mut steps)) {
        Some(_) if steps.0 <= 6 => Class::Sans,
        Some(_) => Class::Serif,
        None => Class::Other,
    }
}

fn roots() -> Vec<(PathBuf, Source)> {
    // The system's font folders on macOS, Linux or Windows (see paths::font_roots).
    let mut r: Vec<(PathBuf, Source)> = crate::paths::font_roots()
        .into_iter()
        .map(|(p, k)| (p, [Source::System, Source::Computer, Source::User][k as usize]))
        .collect();
    r.push((crate::gfonts::dir(), Source::Google));
    r.push((adobe_dir(), Source::Adobe));
    r
}

pub(crate) fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc") {
                out.push(p);
            }
        }
    }
}

fn name_of(face: &ttf_parser::Face, ids: &[u16]) -> Option<String> {
    for &id in ids {
        let mut fallback = None;
        for n in face.names() {
            if n.name_id != id {
                continue;
            }
            if let Some(s) = n.to_string() {
                if s.trim().is_empty() {
                    continue;
                }
                if n.language() == ttf_parser::Language::English_UnitedStates {
                    return Some(s);
                }
                if fallback.is_none() {
                    fallback = Some(s);
                }
            }
        }
        if fallback.is_some() {
            return fallback;
        }
    }
    None
}

/// Finds the installed fonts plus every font in the person's library folders.
pub fn scan(libraries: &[PathBuf]) -> Library {
    use ttf_parser::name_id;
    let mut lib = Library::default();
    let mut seen = std::collections::HashSet::new();
    let mut by_canon: HashMap<PathBuf, usize> = HashMap::new();
    let mut all_roots = roots();
    for l in libraries {
        all_roots.push((l.clone(), Source::Library));
    }
    for (root, source) in all_roots {
        let mut paths = Vec::new();
        walk(&root, &mut paths);
        paths.sort();
        for path in paths {
            let canon = std::fs::canonicalize(&path).unwrap_or(path.clone());
            if !seen.insert(canon.clone()) {
                continue;
            }
            let Ok(f) = std::fs::File::open(&path) else { continue };
            let Ok(map) = (unsafe { Mmap::map(&f) }) else { continue };
            let data = Arc::new(map);
            let count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
            let file_idx = lib.files.len();
            let mut any = false;
            for i in 0..count {
                let Ok(face) = ttf_parser::Face::parse(&data, i) else { continue };
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Font").to_string();
                let family = name_of(&face, &[name_id::TYPOGRAPHIC_FAMILY, name_id::FAMILY]).unwrap_or(stem);
                if family.starts_with('.') {
                    continue; // hidden system interface fonts
                }
                let style = name_of(&face, &[name_id::TYPOGRAPHIC_SUBFAMILY, name_id::SUBFAMILY])
                    .unwrap_or_else(|| "Regular".into());
                lib.faces.push(Face {
                    file: file_idx,
                    index: i,
                    family,
                    style,
                    weight: face.weight().to_number(),
                    italic: face.is_italic() || face.is_oblique(),
                    monospaced: face.is_monospaced(),
                    glyphs: face.number_of_glyphs(),
                });
                any = true;
            }
            if any {
                by_canon.insert(canon, file_idx);
                lib.files.push(FontFile { path, data, source });
            }
        }
    }

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, f) in lib.faces.iter().enumerate() {
        groups.entry(f.family.clone()).or_default().push(i);
    }
    let mut families: Vec<Family> = groups
        .into_iter()
        .map(|(name, mut faces)| {
            faces.sort_by(|&a, &b| {
                let (fa, fb) = (&lib.faces[a], &lib.faces[b]);
                (fa.weight, fa.italic, &fa.style).cmp(&(fb.weight, fb.italic, &fb.style))
            });
            faces.dedup_by(|a, b| {
                let (fa, fb) = (&lib.faces[*a], &lib.faces[*b]);
                fa.style == fb.style && fa.weight == fb.weight && fa.italic == fb.italic
            });
            let default_face = *faces
                .iter()
                .min_by_key(|&&i| {
                    let f = &lib.faces[i];
                    let plain = matches!(f.style.as_str(), "Regular" | "Roman" | "Book" | "Normal" | "Plain");
                    (f.italic, !plain, (f.weight as i32 - 400).abs())
                })
                .unwrap();
            let source = lib.files[lib.faces[faces[0]].file].source;
            let monospaced = lib.faces[default_face].monospaced;
            let df = &lib.faces[default_face];
            let class = ttf_parser::Face::parse(&lib.files[df.file].data, df.index)
                .map(|tf| classify(&tf, &name))
                .unwrap_or(Class::Other);
            Family { name, faces, default_face, source, monospaced, class, libs: Vec::new() }
        })
        .collect();
    families.sort_by_key(|f| f.name.to_lowercase());

    // Which families each library folder holds (a font there may also be installed).
    for (li, root) in libraries.iter().enumerate() {
        let mut paths = Vec::new();
        walk(root, &mut paths);
        let files: std::collections::HashSet<usize> = paths
            .iter()
            .filter_map(|p| by_canon.get(&std::fs::canonicalize(p).unwrap_or(p.clone())).copied())
            .collect();
        for fam in families.iter_mut() {
            if fam.faces.iter().any(|&f| files.contains(&lib.faces[f].file)) {
                fam.libs.push(li);
            }
        }
    }
    lib.families = families;
    lib
}


/// Text drawn in one font as a grey-level picture (one byte of ink per pixel).
pub struct Rendered {
    pub w: usize,
    pub h: usize,
    pub alpha: Vec<u8>,
    pub drew: bool,
    /// Most of the letters aren't in this font (it shows empty boxes instead).
    pub missing: bool,
}

/// The system's own fonts, used for letters a font doesn't have (instead of empty boxes):
/// the interface font first (San Francisco, Segoe UI…), then wide-coverage fonts for other scripts.
fn fallbacks() -> &'static [&'static [u8]] {
    static FB: std::sync::OnceLock<Vec<&'static [u8]>> = std::sync::OnceLock::new();
    FB.get_or_init(|| {
        let mut paths: Vec<String> = crate::oslook::ui_font_path().into_iter().collect();
        for p in [
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            "C:\\Windows\\Fonts\\seguisym.ttf",
            "C:\\Windows\\Fonts\\msyh.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ] {
            if std::path::Path::new(p).exists() {
                paths.push(p.to_string());
            }
        }
        paths
            .iter()
            .filter_map(|p| {
                let f = std::fs::File::open(p).ok()?;
                let map = unsafe { Mmap::map(&f) }.ok()?;
                let bytes: &'static [u8] = &Box::leak(Box::new(map))[..];
                ttf_parser::Face::parse(bytes, 0).ok()?;
                Some(bytes)
            })
            .collect()
    })
}

/// The system's interface font (for names, headings and icons drawn outside the window).
pub fn system_font() -> Option<&'static [u8]> {
    fallbacks().first().copied()
}

/// One glyph to draw: which font (0 = the font itself, 1… = a system fallback) and its glyph id.
type Item = (u8, u16);

/// Draw text in a font. `feats` are the switched-on OpenType features; with `wrap`,
/// long text breaks onto new lines at `max_w` pixels (otherwise it's cut off there).
/// Letters the font doesn't have are drawn with the system's own font instead of empty boxes.
pub fn render(data: &[u8], index: u32, text: &str, px: f32, max_w: usize, feats: &[[u8; 4]], wrap: bool) -> Option<Rendered> {
    let face = ttf_parser::Face::parse(data, index).ok()?;
    let fb_faces: Vec<ttf_parser::Face> = fallbacks().iter().filter_map(|b| ttf_parser::Face::parse(b, 0).ok()).collect();
    let (mut total, mut absent) = (0, 0);
    let mut lines: Vec<Vec<Item>> = Vec::new();
    for line in text.split('\n') {
        let mut items: Vec<Item> = Vec::new();
        let mut run: Vec<u16> = Vec::new();
        let flush = |run: &mut Vec<u16>, items: &mut Vec<Item>| {
            crate::ot::apply(&face, run, feats);
            items.extend(run.drain(..).map(|g| (0u8, g)));
        };
        for c in line.chars() {
            if c.is_control() {
                continue;
            }
            let g = face.glyph_index(c).map(|g| g.0).unwrap_or(0);
            if !c.is_whitespace() {
                total += 1;
                if g == 0 {
                    absent += 1;
                }
            }
            if g == 0 && !c.is_whitespace() {
                if let Some((k, fg)) = fb_faces.iter().enumerate().find_map(|(k, f)| f.glyph_index(c).map(|g| (k, g.0))) {
                    flush(&mut run, &mut items);
                    items.push((k as u8 + 1, fg));
                    continue;
                }
            }
            run.push(g);
        }
        flush(&mut run, &mut items);
        lines.push(items);
    }
    let space = face.glyph_index(' ').map(|g| (0u8, g.0));
    let mut r = draw_lines(data, index, &lines, px, max_w, wrap, space)?;
    r.missing = absent * 2 > total;
    Some(r)
}

/// Draw a row of glyphs given by number (for feature examples).
pub fn render_ids(data: &[u8], index: u32, ids: &[u16], px: f32, max_w: usize) -> Option<Rendered> {
    draw_lines(data, index, &[ids.iter().map(|g| (0u8, *g)).collect()], px, max_w, false, None)
}

fn draw_lines(data: &[u8], index: u32, lines: &[Vec<Item>], px: f32, max_w: usize, wrap: bool, space: Option<Item>) -> Option<Rendered> {
    use ab_glyph::{point, Font, FontRef, GlyphId, PxScale, ScaleFont};
    let main = FontRef::try_from_slice_and_index(data, index).ok()?;
    let mut fonts: Vec<FontRef> = vec![main];
    for b in fallbacks() {
        if let Ok(f) = FontRef::try_from_slice_and_index(b, 0) {
            fonts.push(f);
        }
    }
    let scale = PxScale::from(px);
    let sf = fonts[0].as_scaled(scale);
    let ascent = sf.ascent();
    let descent = sf.descent();
    let line_h = (ascent - descent + sf.line_gap().max(0.0)).ceil().max(px * 0.6);
    let max_wf = max_w as f32;
    let font_of = |i: u8| &fonts[(i as usize).min(fonts.len() - 1)];
    let advance = |it: Item| font_of(it.0).as_scaled(scale).h_advance(GlyphId(it.1));
    let kern = |a: Item, b: Item| if a.0 == b.0 { font_of(a.0).as_scaled(scale).kern(GlyphId(a.1), GlyphId(b.1)) } else { 0.0 };
    let measure = |gs: &[Item]| -> f32 {
        let mut x = 0.0;
        let mut prev: Option<Item> = None;
        for &g in gs {
            if let Some(p) = prev {
                x += kern(p, g);
            }
            x += advance(g);
            prev = Some(g);
        }
        x
    };

    // Break into the lines that will be drawn.
    let mut vis: Vec<Vec<Item>> = Vec::new();
    for line in lines {
        if !wrap {
            vis.push(line.clone());
            continue;
        }
        let mut cur: Vec<Item> = Vec::new();
        let words: Vec<&[Item]> = match space {
            Some(sp) => line.split(|g| *g == sp).collect(),
            None => vec![&line[..]],
        };
        for w in words {
            let mut cand = cur.clone();
            if !cand.is_empty() {
                cand.push(space.unwrap_or((0, 0)));
            }
            cand.extend_from_slice(w);
            if cur.is_empty() || measure(&cand) <= max_wf {
                cur = cand;
            } else {
                vis.push(std::mem::take(&mut cur));
                cur = w.to_vec();
            }
            // A single word longer than the line: split it between letters.
            while cur.len() > 1 && measure(&cur) > max_wf {
                let mut n = cur.len() - 1;
                while n > 1 && measure(&cur[..n]) > max_wf {
                    n -= 1;
                }
                let rest = cur.split_off(n);
                vis.push(std::mem::replace(&mut cur, rest));
            }
        }
        vis.push(cur);
    }
    if vis.is_empty() {
        vis.push(Vec::new());
    }

    // Each line's ink above and below its baseline. Lines are stacked at least a line apart and
    // further when their letters would touch, so wrapped lines never overlap; the picture holds
    // every letter's ink, so nothing is cut off.
    let mut extents: Vec<(f32, f32)> = Vec::new();
    for line in &vis {
        let (mut up, mut down) = (ascent, -descent);
        let mut x = 0.0f32;
        let mut prev: Option<Item> = None;
        for &g in line {
            if let Some(p) = prev {
                x += kern(p, g);
            }
            if x > max_wf {
                break;
            }
            if let Some(o) = font_of(g.0).outline_glyph(GlyphId(g.1).with_scale_and_position(scale, point(x, 0.0))) {
                let b = o.px_bounds();
                up = up.max(-b.min.y);
                down = down.max(b.max.y);
            }
            x += advance(g);
            prev = Some(g);
        }
        extents.push((up, down));
    }
    let gap = (px * 0.06).max(1.0);
    let mut bases: Vec<f32> = Vec::new();
    let mut y = extents[0].0.ceil() + 1.0;
    for (i, e) in extents.iter().enumerate() {
        if i > 0 {
            let prev = extents[i - 1];
            y += line_h.max(prev.1 + e.0 + gap);
        }
        bases.push(y);
    }
    let w = (vis.iter().map(|l| measure(l)).fold(0.0f32, f32::max).ceil() as usize + 2).clamp(1, max_w.max(1));
    let h = (bases.last().copied().unwrap_or(0.0) + extents.last().map(|e| e.1).unwrap_or(0.0)).ceil() as usize + 2;
    let mut alpha = vec![0u8; w * h];
    let mut drew = false;
    for (li, line) in vis.iter().enumerate() {
        let base = bases[li];
        let mut x = 0.0f32;
        let mut prev: Option<Item> = None;
        for &g in line {
            if let Some(p) = prev {
                x += kern(p, g);
            }
            if x > max_wf {
                break;
            }
            if let Some(o) = font_of(g.0).outline_glyph(GlyphId(g.1).with_scale_and_position(scale, point(x, base))) {
                let b = o.px_bounds();
                o.draw(|gx, gy, c| {
                    let px_ = b.min.x as i32 + gx as i32;
                    let py = b.min.y as i32 + gy as i32;
                    if px_ >= 0 && py >= 0 && (px_ as usize) < w && (py as usize) < h {
                        let i = py as usize * w + px_ as usize;
                        let v = (c.clamp(0.0, 1.0) * 255.0) as u8;
                        if v > 0 {
                            drew = true;
                        }
                        alpha[i] = alpha[i].max(v);
                    }
                });
            }
            x += advance(g);
            prev = Some(g);
        }
    }
    Some(Rendered { w, h, alpha, drew, missing: false })
}

/// How tall `text` is in this font, in ems: from the top of its highest letter (or the font's
/// ascender) to the bottom of its lowest (or the descender), plus the line gap. A sample drawn at
/// `px` pixels needs about `px * sample_extent(..)` pixels of height.
pub fn sample_extent(data: &[u8], index: u32, text: &str) -> f32 {
    let Ok(face) = ttf_parser::Face::parse(data, index) else { return 1.3 };
    let upem = face.units_per_em().max(1) as f32;
    let (mut top, mut bottom) = (face.ascender() as f32, face.descender() as f32);
    for c in text.chars().filter(|c| !c.is_control()) {
        if let Some(b) = face.glyph_index(c).and_then(|g| face.glyph_bounding_box(g)) {
            top = top.max(b.y_max as f32);
            bottom = bottom.min(b.y_min as f32);
        }
    }
    ((top - bottom + (face.line_gap() as f32).max(0.0)) / upem).clamp(0.6, 6.0)
}

/// A short name for a font's format, for its icon in the list: "OTF" (PostScript outlines),
/// "TTF" (TrueType outlines), "TTC"/"OTC" (collections), "VAR" (variable font).
pub fn format_label(data: &[u8], index: u32) -> &'static str {
    let collection = data.starts_with(b"ttcf");
    let Ok(face) = ttf_parser::Face::parse(data, index) else { return "?" };
    let cff = face.tables().cff.is_some() || face.tables().cff2.is_some();
    if face.is_variable() {
        "VAR"
    } else if collection {
        if cff { "OTC" } else { "TTC" }
    } else if cff {
        "OTF"
    } else {
        "TTF"
    }
}

/// One glyph, centred in its own picture (for the Glyphs page).
pub fn render_glyph(data: &[u8], index: u32, gid: u16, px: f32) -> Option<Rendered> {
    use ab_glyph::{point, Font, FontRef, GlyphId, PxScale, ScaleFont};
    let font = FontRef::try_from_slice_and_index(data, index).ok()?;
    let scale = PxScale::from(px);
    let sf = font.as_scaled(scale);
    let ascent = sf.ascent();
    let h = ((ascent - sf.descent()).ceil() as usize).max(1) + 4;
    let probe = GlyphId(gid).with_scale_and_position(scale, point(0.0, ascent + 2.0));
    let Some(o) = font.outline_glyph(probe) else {
        return Some(Rendered { w: 1, h, alpha: vec![0], drew: false, missing: false });
    };
    let b = o.px_bounds();
    // The picture holds all of the glyph's ink, even above the ascender or below the descender.
    let shift = (-b.min.y).max(0.0).ceil() as i32;
    let h = h.max((b.max.y.ceil() as i32 + shift + 2).max(1) as usize);
    let w = ((b.width().ceil() as usize + 4).max(sf.h_advance(GlyphId(gid)).ceil() as usize)).max(1);
    let dx = ((w as f32 - b.width()) / 2.0).round() as i32;
    let mut alpha = vec![0u8; w * h];
    let mut drew = false;
    o.draw(|gx, gy, c| {
        let x = dx + gx as i32;
        let y = b.min.y as i32 + shift + gy as i32;
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let v = (c.clamp(0.0, 1.0) * 255.0) as u8;
            if v > 0 {
                drew = true;
            }
            let i = y as usize * w + x as usize;
            alpha[i] = alpha[i].max(v);
        }
    });
    Some(Rendered { w, h, alpha, drew, missing: false })
}
