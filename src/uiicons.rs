//! FontCraft's own interface icons as SVG (assets/icons/ui), drawn exactly as the sidebar draws
//! them (same coordinates and strokes): the activation asterisk, the favourite star, libraries,
//! folders, families, the smart lists, and the menu actions. They are rendered with resvg wherever
//! an image is needed, such as the system's own menus; "{c}" in each file is the colour.

/// The SVG source of an icon by name.
pub fn source(name: &str) -> Option<&'static str> {
    Some(match name {
        "asterisk" => include_str!("../assets/icons/ui/asterisk.svg"),
        "star" => include_str!("../assets/icons/ui/star.svg"),
        "star-filled" => include_str!("../assets/icons/ui/star-filled.svg"),
        "library" => include_str!("../assets/icons/ui/library.svg"),
        "folder" => include_str!("../assets/icons/ui/folder.svg"),
        "set" => include_str!("../assets/icons/ui/set.svg"),
        "family" => include_str!("../assets/icons/ui/family.svg"),
        "clock" => include_str!("../assets/icons/ui/clock.svg"),
        "export" => include_str!("../assets/icons/ui/export.svg"),
        "reveal" => include_str!("../assets/icons/ui/reveal.svg"),
        "trash" => include_str!("../assets/icons/ui/trash.svg"),
        "add" => include_str!("../assets/icons/ui/add.svg"),
        "automation" => include_str!("../assets/icons/ui/automation.svg"),
        "smart-mono" => include_str!("../assets/icons/ui/smart-mono.svg"),
        "smart-sans" => include_str!("../assets/icons/ui/smart-sans.svg"),
        "smart-serif" => include_str!("../assets/icons/ui/smart-serif.svg"),
        "smart-slab" => include_str!("../assets/icons/ui/smart-slab.svg"),
        "smart-blackletter" => include_str!("../assets/icons/ui/smart-blackletter.svg"),
        "smart-display" => include_str!("../assets/icons/ui/smart-display.svg"),
        "smart-script" => include_str!("../assets/icons/ui/smart-script.svg"),
        "smart-symbol" => include_str!("../assets/icons/ui/smart-symbol.svg"),
        _ => return None,
    })
}

/// An icon in `rgb`, `px` pixels square, as straight (not premultiplied) RGBA bytes.
pub fn rgba(name: &str, rgb: [u8; 3], px: u32) -> Option<Vec<u8>> {
    use resvg::{tiny_skia, usvg};
    let svg = source(name)?.replace("{c}", &format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]));
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
    let mut pix = tiny_skia::Pixmap::new(px, px)?;
    let k = px as f32 / tree.size().width().max(1.0);
    resvg::render(&tree, tiny_skia::Transform::from_scale(k, k), &mut pix.as_mut());
    Some(pix.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect())
}

/// Paints an icon centred at `c`, `pt` points square, in `col` (drawn once from its SVG in white
/// at the screen's pixel density, kept as a texture and tinted).
pub fn paint(ui: &eframe::egui::Ui, name: &str, c: eframe::egui::Pos2, pt: f32, col: eframe::egui::Color32) {
    use eframe::egui;
    let ctx = ui.ctx();
    let px = (pt * ctx.pixels_per_point()).round().max(1.0) as u32;
    let id = egui::Id::unique(("ui-icon", name, px));
    let tex: Option<egui::TextureHandle> = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = rgba(name, [255, 255, 255], px).map(|b| ctx.load_texture(format!("ui-icon-{name}"), egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &b), egui::TextureOptions::LINEAR));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    if let Some(t) = tex {
        let r = egui::Rect::from_center_size(c, egui::vec2(pt, pt));
        ui.painter().image(t.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), col);
    }
}

/// The icon of a smart list, by its saved key (menus::smart_key).
pub fn smart(key: &str) -> &'static str {
    match key {
        "sans" => "smart-sans",
        "serif" => "smart-serif",
        "slab" => "smart-slab",
        "script" => "smart-script",
        "blackletter" => "smart-blackletter",
        "display" => "smart-display",
        "symbol" => "smart-symbol",
        _ => "smart-mono",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_icon_draws() {
        for name in [
            "asterisk", "star", "star-filled", "library", "folder", "set", "family", "clock", "export", "reveal", "trash", "add", "automation",
            "smart-mono", "smart-sans", "smart-serif", "smart-slab", "smart-blackletter", "smart-display", "smart-script", "smart-symbol",
        ] {
            let px = super::rgba(name, [0, 0, 0], 16).unwrap_or_else(|| panic!("{name} didn't render"));
            assert!(px.chunks(4).filter(|p| p[3] > 0).count() > 8, "{name} is empty");
        }
    }
}
