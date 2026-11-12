//! Looking inside a font's OpenType tables: which features it has, what each one swaps,
//! applying switched-on features to preview text, and the facts shown on the Info page.

use std::collections::HashMap;
use ttf_parser::gsub::SubstitutionSubtable;
use ttf_parser::opentype_layout::{Coverage, LayoutTable};
use ttf_parser::{GlyphId, Tag};

/// The features with their own drop-down, in the order they're shown.
pub const MAIN_FEATURES: [&[u8; 4]; 6] = [b"aalt", b"case", b"ccmp", b"liga", b"locl", b"medi"];

/// Features that are on until switched off (as text normally shows).
pub fn on_by_default(tag: &[u8; 4]) -> bool {
    matches!(tag, b"ccmp" | b"liga" | b"locl" | b"medi")
}

/// Order features are applied in, close to how text engines do it.
pub fn apply_order(tag: &[u8; 4]) -> u8 {
    match tag {
        b"ccmp" => 0,
        b"locl" => 1,
        b"medi" => 2,
        b"case" => 3,
        b"aalt" => 4,
        b"liga" => 9,
        _ => 5, // stylistic sets
    }
}

/// One example of what a feature does: these glyphs become those glyphs.
#[derive(Clone)]
pub struct Swap {
    pub from: Vec<u16>,
    pub to: Vec<u16>,
}

#[derive(Clone, Default)]
pub struct FeatureDetail {
    pub present: bool,
    pub swaps: Vec<Swap>,
    /// All swaps, including those not kept as examples.
    pub total: usize,
    /// Rules that depend on neighbouring letters (no fixed example).
    pub contextual: usize,
}

const MAX_EXAMPLES: usize = 240;

fn coverage_glyphs(c: &Coverage) -> Vec<(u16, u16)> {
    let mut out = Vec::new();
    match c {
        Coverage::Format1 { glyphs } => {
            for (i, g) in glyphs.into_iter().enumerate() {
                out.push((g.0, i as u16));
            }
        }
        Coverage::Format2 { records } => {
            for r in records.into_iter() {
                for g in r.start.0..=r.end.0 {
                    out.push((g, r.value + (g - r.start.0)));
                }
            }
        }
    }
    out
}

/// Every lookup a feature tag uses anywhere in the font.
fn all_lookups(t: &LayoutTable, tag: Tag) -> Vec<u16> {
    let mut v = Vec::new();
    for i in 0..t.features.len() {
        if let Some(f) = t.features.get(i) {
            if f.tag == tag {
                for li in f.lookup_indices {
                    if !v.contains(&li) {
                        v.push(li);
                    }
                }
            }
        }
    }
    v.sort_unstable();
    v
}

/// The lookups a text engine would use with no particular language set:
/// the default language of the Latin (or default) script, or else the first record with that tag.
fn lookups_for_text(t: &LayoutTable, tag: Tag) -> Vec<u16> {
    let mut scripts = Vec::new();
    for want in [Tag::from_bytes(b"latn"), Tag::from_bytes(b"DFLT")] {
        if let Some(s) = t.scripts.find(want) {
            scripts.push(s);
        }
    }
    if let Some(s) = t.scripts.get(0) {
        scripts.push(s);
    }
    for s in scripts {
        if let Some(ls) = s.default_language {
            for fi in ls.feature_indices {
                if let Some(f) = t.features.get(fi) {
                    if f.tag == tag {
                        return f.lookup_indices.into_iter().collect();
                    }
                }
            }
        }
    }
    for i in 0..t.features.len() {
        if let Some(f) = t.features.get(i) {
            if f.tag == tag {
                return f.lookup_indices.into_iter().collect();
            }
        }
    }
    Vec::new()
}

pub fn feature_detail(face: &ttf_parser::Face, tag: &[u8; 4]) -> FeatureDetail {
    let mut d = FeatureDetail::default();
    let Some(gsub) = face.tables().gsub else {
        // A feature that only moves letters (in the positioning table) still counts as present.
        if let Some(gpos) = face.tables().gpos {
            d.present = !all_lookups(&gpos, Tag::from_bytes(tag)).is_empty();
        }
        return d;
    };
    let lookups = all_lookups(&gsub, Tag::from_bytes(tag));
    d.present = !lookups.is_empty();
    let push = |d: &mut FeatureDetail, from: Vec<u16>, to: Vec<u16>| {
        d.total += 1;
        if d.swaps.len() < MAX_EXAMPLES && from != to {
            d.swaps.push(Swap { from, to });
        }
    };
    for li in lookups {
        let Some(lookup) = gsub.lookups.get(li) else { continue };
        for st in lookup.subtables.into_iter::<SubstitutionSubtable>() {
            match st {
                SubstitutionSubtable::Single(s) => {
                    use ttf_parser::gsub::SingleSubstitution as S;
                    for (g, ci) in coverage_glyphs(&s.coverage()) {
                        let to = match s {
                            S::Format1 { delta, .. } => (g as i32 + delta as i32) as u16,
                            S::Format2 { substitutes, .. } => match substitutes.get(ci) {
                                Some(x) => x.0,
                                None => continue,
                            },
                        };
                        push(&mut d, vec![g], vec![to]);
                    }
                }
                SubstitutionSubtable::Alternate(a) => {
                    for (g, ci) in coverage_glyphs(&a.coverage) {
                        if let Some(set) = a.alternate_sets.get(ci) {
                            let alts: Vec<u16> = set.alternates.into_iter().map(|x| x.0).collect();
                            if !alts.is_empty() {
                                push(&mut d, vec![g], alts);
                            }
                        }
                    }
                }
                SubstitutionSubtable::Multiple(m) => {
                    for (g, ci) in coverage_glyphs(&m.coverage) {
                        if let Some(seq) = m.sequences.get(ci) {
                            push(&mut d, vec![g], seq.substitutes.into_iter().map(|x| x.0).collect());
                        }
                    }
                }
                SubstitutionSubtable::Ligature(l) => {
                    for (g, ci) in coverage_glyphs(&l.coverage) {
                        if let Some(set) = l.ligature_sets.get(ci) {
                            for lig in set.into_iter() {
                                let mut from = vec![g];
                                from.extend(lig.components.into_iter().map(|x| x.0));
                                push(&mut d, from, vec![lig.glyph.0]);
                            }
                        }
                    }
                }
                _ => d.contextual += 1,
            }
        }
    }
    d
}

/// Stylistic set numbers (1–20) the font has.
pub fn stylistic_sets(face: &ttf_parser::Face) -> Vec<u8> {
    let mut v = Vec::new();
    if let Some(gsub) = face.tables().gsub {
        for i in 0..gsub.features.len() {
            if let Some(f) = gsub.features.get(i) {
                let b = f.tag.to_bytes();
                if &b[..2] == b"ss" && b[2].is_ascii_digit() && b[3].is_ascii_digit() {
                    let n = (b[2] - b'0') * 10 + (b[3] - b'0');
                    if (1..=20).contains(&n) && !v.contains(&n) {
                        v.push(n);
                    }
                }
            }
        }
    }
    v.sort_unstable();
    v
}

pub fn ss_tag(n: u8) -> [u8; 4] {
    [b's', b's', b'0' + n / 10, b'0' + n % 10]
}

/// Change the glyphs of a line of text with the switched-on features.
/// Context rules aren't followed; single, alternate, multiple and ligature swaps are.
pub fn apply(face: &ttf_parser::Face, glyphs: &mut Vec<u16>, tags: &[[u8; 4]]) {
    let Some(gsub) = face.tables().gsub else { return };
    for tag in tags {
        let medial_only = tag == b"medi";
        for li in lookups_for_text(&gsub, Tag::from_bytes(tag)) {
            let Some(lookup) = gsub.lookups.get(li) else { continue };
            let mut i = 0;
            while i < glyphs.len() {
                if medial_only && (i == 0 || i + 1 == glyphs.len()) {
                    i += 1;
                    continue;
                }
                let g = GlyphId(glyphs[i]);
                let mut step = 1;
                for st in lookup.subtables.into_iter::<SubstitutionSubtable>() {
                    let hit = match st {
                        SubstitutionSubtable::Single(s) => {
                            use ttf_parser::gsub::SingleSubstitution as S;
                            match s.coverage().get(g) {
                                Some(ci) => {
                                    glyphs[i] = match s {
                                        S::Format1 { delta, .. } => (g.0 as i32 + delta as i32) as u16,
                                        S::Format2 { substitutes, .. } => substitutes.get(ci).map(|x| x.0).unwrap_or(g.0),
                                    };
                                    true
                                }
                                None => false,
                            }
                        }
                        SubstitutionSubtable::Alternate(a) => match a.coverage.get(g) {
                            Some(ci) => {
                                if let Some(alt) = a.alternate_sets.get(ci).and_then(|s| s.alternates.get(0)) {
                                    glyphs[i] = alt.0;
                                }
                                true
                            }
                            None => false,
                        },
                        SubstitutionSubtable::Multiple(m) => match m.coverage.get(g) {
                            Some(ci) => {
                                if let Some(seq) = m.sequences.get(ci) {
                                    let subs: Vec<u16> = seq.substitutes.into_iter().map(|x| x.0).collect();
                                    if !subs.is_empty() {
                                        step = subs.len();
                                        glyphs.splice(i..i + 1, subs);
                                    }
                                }
                                true
                            }
                            None => false,
                        },
                        SubstitutionSubtable::Ligature(l) => match l.coverage.get(g) {
                            Some(ci) => {
                                let mut done = false;
                                if let Some(set) = l.ligature_sets.get(ci) {
                                    for lig in set.into_iter() {
                                        let n = lig.components.len() as usize;
                                        if i + n < glyphs.len() + 1
                                            && i + 1 + n <= glyphs.len()
                                            && lig.components.into_iter().enumerate().all(|(k, c)| glyphs[i + 1 + k] == c.0)
                                        {
                                            glyphs.splice(i..i + 1 + n, [lig.glyph.0]);
                                            done = true;
                                            break;
                                        }
                                    }
                                }
                                done
                            }
                            None => false,
                        },
                        _ => false,
                    };
                    if hit {
                        break;
                    }
                }
                i += step;
            }
        }
    }
}

/// Glyph number → the character it draws (for the Glyphs page).
pub fn reverse_cmap(face: &ttf_parser::Face) -> HashMap<u16, u32> {
    let mut map = HashMap::new();
    if let Some(cmap) = face.tables().cmap {
        for st in cmap.subtables {
            if !st.is_unicode() {
                continue;
            }
            st.codepoints(|cp| {
                if let Some(g) = st.glyph_index(cp) {
                    map.entry(g.0).or_insert(cp);
                }
            });
        }
    }
    map
}

pub struct FontInfo {
    pub full_name: String,
    pub family: String,
    pub style: String,
    pub format: String,
    pub ps_name: String,
    pub unique: String,
    pub version: String,
    pub glyphs: u16,
    /// (axis tag, min, default, max)
    pub axes: Vec<(String, f32, f32, f32)>,
    pub scripts: Vec<String>,
    pub features: Vec<String>,
    pub italic: bool,
    pub bold: bool,
    pub monospaced: bool,
    /// Languages whose letters the font fully covers, each named in its own language.
    pub languages: Vec<&'static str>,
    /// name table: 13 licence description, 14 licence URL, 0 copyright, 7 trademark, 9 designer, 8 manufacturer.
    pub license: String,
    pub license_url: String,
    pub copyright: String,
    pub trademark: String,
    pub designer: String,
    pub manufacturer: String,
    /// OS/2 fsType: the embedding rights the font allows.
    pub fs_type: u16,
}

fn name(face: &ttf_parser::Face, id: u16) -> String {
    let mut fallback = String::new();
    for n in face.names() {
        if n.name_id != id {
            continue;
        }
        if let Some(s) = n.to_string() {
            if n.language() == ttf_parser::Language::English_UnitedStates {
                return s;
            }
            if fallback.is_empty() {
                fallback = s;
            }
        }
    }
    fallback
}

fn tag_text(t: Tag) -> String {
    String::from_utf8_lossy(&t.to_bytes()).trim().to_string()
}

pub fn info(face: &ttf_parser::Face, path: &std::path::Path, in_collection: bool) -> FontInfo {
    use ttf_parser::name_id as id;
    let first = |a: u16, b: u16| {
        let s = name(face, a);
        if s.is_empty() {
            name(face, b)
        } else {
            s
        }
    };
    let t = face.tables();
    let outlines = if t.cff2.is_some() || t.cff.is_some() {
        "OpenType PostScript (CFF)"
    } else if t.glyf.is_some() {
        "OpenType TrueType"
    } else if t.sbix.is_some() || t.cbdt.is_some() {
        "OpenType Bitmap"
    } else {
        "OpenType"
    };
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_uppercase();
    let mut format = format!("{outlines} · .{ext}");
    if in_collection {
        format.push_str(" (Collection)");
    }
    let mut scripts = Vec::new();
    let mut features = Vec::new();
    for table in [t.gsub, t.gpos].into_iter().flatten() {
        for i in 0..table.scripts.len() {
            if let Some(s) = table.scripts.get(i) {
                let x = tag_text(s.tag);
                if !scripts.contains(&x) {
                    scripts.push(x);
                }
            }
        }
        for i in 0..table.features.len() {
            if let Some(f) = table.features.get(i) {
                let x = tag_text(f.tag);
                if !features.contains(&x) {
                    features.push(x);
                }
            }
        }
    }
    features.sort();
    FontInfo {
        full_name: name(face, id::FULL_NAME),
        family: first(id::TYPOGRAPHIC_FAMILY, id::FAMILY),
        style: first(id::TYPOGRAPHIC_SUBFAMILY, id::SUBFAMILY),
        format,
        ps_name: name(face, id::POST_SCRIPT_NAME),
        unique: name(face, id::UNIQUE_ID),
        version: name(face, id::VERSION),
        glyphs: face.number_of_glyphs(),
        axes: face
            .variation_axes()
            .into_iter()
            .map(|a| (tag_text(a.tag), a.min_value, a.def_value, a.max_value))
            .collect(),
        scripts,
        features,
        italic: face.is_italic() || face.is_oblique(),
        bold: face.weight().to_number() >= 700,
        monospaced: face.is_monospaced(),
        languages: crate::langs::supported(face),
        license: name(face, id::LICENSE),
        license_url: name(face, id::LICENSE_URL),
        copyright: name(face, id::COPYRIGHT_NOTICE),
        trademark: name(face, id::TRADEMARK),
        designer: name(face, id::DESIGNER),
        manufacturer: name(face, id::MANUFACTURER),
        fs_type: face
            .raw_face()
            .table(Tag::from_bytes(b"OS/2"))
            .filter(|t| t.len() >= 10)
            .map(|t| u16::from_be_bytes([t[8], t[9]]))
            .unwrap_or(0),
    }
}
