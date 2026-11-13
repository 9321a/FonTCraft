//! FontCraft's own icons, made from the person's drawings in `assets/icons/source` (the app: an
//! "A" with a butterfly and FontCraft's green asterisk; the .fcl document: a page with the asterisk
//! and an "FCL" band), each in a light and a dark version. They are drawn at any size with resvg.
//! `--make-icons <folder>` writes every size plus the macOS .icns, the Windows .ico and the Linux
//! theme PNGs; `--make-android-icons` the Android launcher icons (adaptive, with a night version).
//! The running app uses the version that matches the system's light/dark look.

use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    App,
    Document,
}

const APP_LIGHT: &str = include_str!("../assets/icons/source/app-light.svg");
const APP_DARK: &str = include_str!("../assets/icons/source/app-dark.svg");
const FCL_LIGHT: &str = include_str!("../assets/icons/source/fcl-light.svg");
const FCL_DARK: &str = include_str!("../assets/icons/source/fcl-dark.svg");

/// Draws one icon, `s` pixels square, as RGBA bytes: the drawing as large as fits, centred, with
/// `margin` (a share of the side) left clear around it.
fn render_svg(svg: &str, s: usize, margin: f32) -> Vec<u8> {
    use resvg::{tiny_skia, usvg};
    let mut out = vec![0u8; s * s * 4];
    let Ok(tree) = usvg::Tree::from_str(svg, &usvg::Options::default()) else { return out };
    let Some(mut pix) = tiny_skia::Pixmap::new(s as u32, s as u32) else { return out };
    let size = tree.size();
    let room = s as f32 * (1.0 - 2.0 * margin);
    let k = (room / size.width()).min(room / size.height());
    let (dx, dy) = ((s as f32 - size.width() * k) / 2.0, (s as f32 - size.height() * k) / 2.0);
    resvg::render(&tree, tiny_skia::Transform::from_row(k, 0.0, 0.0, k, dx, dy), &mut pix.as_mut());
    // tiny-skia keeps premultiplied colour; PNG and the window icon want it straight.
    for (o, p) in out.chunks_exact_mut(4).zip(pix.pixels()) {
        let c = p.demultiply();
        o.copy_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}

/// The app icon on its own container, adapted to the design language of each host OS:
/// - macOS: rounded squircle on Apple's icon grid (824/1024, 22.5% radius), soft drop shadow, gradient.
/// - iOS: squircle container (1000/1024, 22.5% radius), clean surface without drop shadow.
/// - Windows: Fluent Design (880/1024, 12% radius), clean modern elevation.
/// - Android: Material Design adaptive squircle/circle (900/1024, 26% radius).
/// - Linux: GNOME / FreeDesktop HIG (860/1024, 18% radius).
/// Scalable up to 512x512 pixels.
fn render_tile(s: usize, dark: bool) -> Vec<u8> {
    use resvg::tiny_skia::{Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, SpreadMode, Stroke, Transform};
    let sf = s as f32;
    let Some(mut pix) = Pixmap::new(s as u32, s as u32) else { return vec![0; s * s * 4] };

    let (body_ratio, radius_ratio, has_shadow, shadow_layers) = if cfg!(target_os = "macos") {
        (824.0 / 1024.0, 0.225, true, 8)
    } else if cfg!(target_os = "ios") {
        (1000.0 / 1024.0, 0.225, false, 0)
    } else if cfg!(target_os = "windows") {
        (880.0 / 1024.0, 0.120, true, 3)
    } else if cfg!(target_os = "android") {
        (900.0 / 1024.0, 0.260, false, 0)
    } else {
        (860.0 / 1024.0, 0.180, true, 4)
    };

    let body = sf * body_ratio;
    let x0 = (sf - body) / 2.0;
    let y0 = if has_shadow { x0 - sf * 0.008 } else { x0 };
    let rounded = |x: f32, y: f32, w: f32, h: f32, r: f32| {
        // A rounded rectangle with circular corners (k: the cubic handle for a quarter circle).
        let k = 0.5523 * r;
        let mut pb = PathBuilder::new();
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
        pb.line_to(x, y + r);
        pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
        pb.close();
        pb.finish()
    };
    let radius = body * radius_ratio;
    // Shadow: a few widening, fading layers a little below the tile when the host OS uses drop shadows.
    if has_shadow && s >= 32 {
        for i in 0..shadow_layers {
            let grow = sf * 0.0035 * i as f32;
            if let Some(p) = rounded(x0 - grow, y0 + sf * 0.012 - grow + sf * 0.004, body + 2.0 * grow, body + 2.0 * grow, radius + grow) {
                let mut paint = Paint::default();
                paint.set_color(Color::from_rgba8(0, 0, 0, if dark { 16 } else { 10 }));
                paint.anti_alias = true;
                pix.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
    }
    let Some(tile) = rounded(x0, y0, body, body, radius) else { return vec![0; s * s * 4] };
    let (top, bottom, edge) = if dark {
        (Color::from_rgba8(0x3a, 0x3a, 0x3c, 255), Color::from_rgba8(0x1c, 0x1c, 0x1e, 255), Color::from_rgba8(255, 255, 255, 38))
    } else {
        (Color::from_rgba8(0xff, 0xff, 0xff, 255), Color::from_rgba8(0xe9, 0xe9, 0xee, 255), Color::from_rgba8(0, 0, 0, 30))
    };
    let mut paint = Paint::default();
    paint.anti_alias = true;
    if let Some(g) = LinearGradient::new(Point::from_xy(0.0, y0), Point::from_xy(0.0, y0 + body), vec![GradientStop::new(0.0, top), GradientStop::new(1.0, bottom)], SpreadMode::Pad, Transform::identity()) {
        paint.shader = g;
    }
    pix.fill_path(&tile, &paint, FillRule::Winding, Transform::identity(), None);
    let mut line = Paint::default();
    line.anti_alias = true;
    line.set_color(edge);
    pix.stroke_path(&tile, &line, &Stroke { width: (sf / 512.0).max(0.5), ..Stroke::default() }, Transform::identity(), None);
    // The drawing, inside the tile with room around it (scaling up to 512x512).
    let inner = (body * 0.74).round().max(1.0) as usize;
    let art = render_svg(if dark { APP_DARK } else { APP_LIGHT }, inner, 0.0);
    if let Some(art_pix) = resvg::tiny_skia::Pixmap::from_vec(premultiply(art), resvg::tiny_skia::IntSize::from_wh(inner as u32, inner as u32).unwrap_or(resvg::tiny_skia::IntSize::from_wh(1, 1).unwrap())) {
        let ax = ((sf - inner as f32) / 2.0).round() as i32;
        let ay = (y0 + (body - inner as f32) / 2.0).round() as i32;
        pix.draw_pixmap(ax, ay, art_pix.as_ref(), &resvg::tiny_skia::PixmapPaint::default(), Transform::identity(), None);
    }
    let mut out = vec![0u8; s * s * 4];
    for (o, p) in out.chunks_exact_mut(4).zip(pix.pixels()) {
        let c = p.demultiply();
        o.copy_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}

/// Straight RGBA to the premultiplied form tiny-skia draws with.
fn premultiply(mut rgba: Vec<u8>) -> Vec<u8> {
    for p in rgba.chunks_exact_mut(4) {
        let a = p[3] as u16;
        for c in &mut p[..3] {
            *c = ((*c as u16 * a + 127) / 255) as u8;
        }
    }
    rgba
}

/// One icon, `s` pixels square, as RGBA bytes.
pub fn render(kind: Kind, s: usize, dark: bool) -> Vec<u8> {
    match (kind, dark) {
        // The app: its drawing on its own container tile, light or dark.
        (Kind::App, d) => render_tile(s, d),
        // A document icon fills the height, as document icons do.
        (Kind::Document, false) => render_svg(FCL_LIGHT, s, 0.02),
        (Kind::Document, true) => render_svg(FCL_DARK, s, 0.02),
    }
}

/// The window/dock icon matching the system's look, rendered at full 512x512 resolution.
pub fn window_icon(dark: bool) -> eframe::egui::IconData {
    let s = 512;
    eframe::egui::IconData { rgba: render(Kind::App, s, dark), width: s as u32, height: s as u32 }
}

/// Puts the light or dark tile in the Dock while FontCraft runs (macOS: NSApp's application icon
/// image; eframe sets it once at start and winit can't change it afterwards). Main thread only.
#[cfg(target_os = "macos")]
pub fn set_dock_icon(dark: bool) {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    let bytes = png(&render(Kind::App, 512, dark), 512);
    let run = || unsafe {
        let data = objc2_foundation::NSData::with_bytes(&bytes);
        let img: Option<Retained<AnyObject>> = msg_send![msg_send![class!(NSImage), alloc], initWithData: &*data];
        let Some(img) = img else { return };
        let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
        if !app.is_null() {
            let _: () = msg_send![app, setApplicationIconImage: &*img];
        }
    };
    let _ = objc2::exception::catch(std::panic::AssertUnwindSafe(run));
}

/// Whether the system is set to its dark look.
pub fn system_is_dark() -> bool {
    let out = |prog: &str, args: &[&str]| -> String {
        crate::paths::command(prog).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default()
    };
    if cfg!(target_os = "macos") {
        out("defaults", &["read", "-g", "AppleInterfaceStyle"]).contains("Dark")
    } else if cfg!(target_os = "windows") {
        // Windows 10/11: AppsUseLightTheme = 0 is the dark look (the taskbar and windows).
        out("reg", &["query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "/v", "AppsUseLightTheme"]).contains("0x0")
    } else if cfg!(target_os = "android") {
        out("cmd", &["uimode"]).to_lowercase().contains("yes")
            || out("settings", &["get", "secure", "ui_night_mode"]).trim() == "2"
    } else {
        // GNOME, Cinnamon, Budgie and others: the desktop's colour scheme, else a "-dark" GTK theme.
        let scheme = out("gsettings", &["get", "org.gnome.desktop.interface", "color-scheme"]);
        scheme.contains("dark") || out("gsettings", &["get", "org.gnome.desktop.interface", "gtk-theme"]).to_lowercase().contains("dark")
    }
}

fn png(rgba: &[u8], s: usize) -> Vec<u8> {
    let img = image::RgbaImage::from_raw(s as u32, s as u32, rgba.to_vec()).unwrap_or_default();
    let mut out = std::io::Cursor::new(Vec::new());
    let _ = img.write_to(&mut out, image::ImageFormat::Png);
    out.into_inner()
}

/// A Windows .ico holding PNG pictures.
fn ico(pictures: &[(usize, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0, 0, 1, 0];
    out.extend_from_slice(&(pictures.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * pictures.len();
    for (s, data) in pictures {
        let d = if *s >= 256 { 0 } else { *s as u8 };
        out.extend_from_slice(&[d, d, 0, 0, 1, 0, 32, 0]);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += data.len();
    }
    for (_, data) in pictures {
        out.extend_from_slice(data);
    }
    out
}

/// Writes every icon file into `dir`.
pub fn make_all(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut made = Vec::new();
    let sizes = [16usize, 32, 48, 64, 128, 256, 512];
    for (kind, base) in [(Kind::App, "fontcraft"), (Kind::Document, "fontcraft-library")] {
        for dark in [false, true] {
            let suffix = if dark { "-dark" } else { "" };
            let png_dir = dir.join("png");
            std::fs::create_dir_all(&png_dir)?;
            let mut ico_pics = Vec::new();
            for s in sizes {
                let bytes = png(&render(kind, s, dark), s);
                std::fs::write(png_dir.join(format!("{base}{suffix}-{s}.png")), &bytes)?;
                if [16, 32, 48, 256].contains(&s) {
                    ico_pics.push((s, bytes));
                }
            }
            std::fs::write(dir.join(format!("{base}{suffix}.ico")), ico(&ico_pics))?;
            made.push(format!("{base}{suffix}"));
            // macOS .icns, made with the system's iconutil from an iconset.
            if cfg!(target_os = "macos") {
                let set = dir.join(format!("{base}{suffix}.iconset"));
                std::fs::create_dir_all(&set)?;
                for (s, name) in [
                    (16, "icon_16x16.png"),
                    (32, "icon_16x16@2x.png"),
                    (32, "icon_32x32.png"),
                    (64, "icon_32x32@2x.png"),
                    (128, "icon_128x128.png"),
                    (256, "icon_128x128@2x.png"),
                    (256, "icon_256x256.png"),
                    (512, "icon_256x256@2x.png"),
                    (512, "icon_512x512.png"),
                ] {
                    std::fs::write(set.join(name), png(&render(kind, s, dark), s))?;
                }
                let icns = dir.join(format!("{base}{suffix}.icns"));
                let ok = crate::paths::command("iconutil").arg("-c").arg("icns").arg(&set).arg("-o").arg(&icns).status().map(|s| s.success()).unwrap_or(false);
                if ok {
                    let _ = std::fs::remove_dir_all(&set);
                }
            }
        }
    }
    Ok(made)
}

/// Android launcher icons: the classic one per screen density (`mipmap-*/ic_launcher.png`), and the
/// adaptive icon Android 8+ shows (the drawing inside the 66 dp safe circle of a 108 dp layer, on a
/// light background; with the night look, the dark drawing on a dark background; a monochrome layer
/// for Android 13's themed icons).
pub fn make_android(res: &Path) -> std::io::Result<()> {
    let w = |dir: &str, name: &str, bytes: Vec<u8>| -> std::io::Result<()> {
        let d = res.join(dir);
        std::fs::create_dir_all(&d)?;
        std::fs::write(d.join(name), bytes)
    };
    for (dpi, s) in [("mdpi", 48usize), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)] {
        w(&format!("mipmap-{dpi}"), "ic_launcher.png", png(&render(Kind::App, s, false), s))?;
        let fs = s * 108 / 48;
        w(&format!("mipmap-{dpi}"), "ic_launcher_foreground.png", png(&render_svg(APP_LIGHT, fs, 0.21), fs))?;
        w(&format!("mipmap-night-{dpi}"), "ic_launcher_foreground.png", png(&render_svg(APP_DARK, fs, 0.21), fs))?;
    }
    // High-resolution 512x512 launcher / store icons (light and dark).
    w("drawable-nodpi", "ic_launcher_512.png", png(&render(Kind::App, 512, false), 512))?;
    w("drawable-night-nodpi", "ic_launcher_512.png", png(&render(Kind::App, 512, true), 512))?;
    w("mipmap-xxxhdpi", "ic_launcher_512.png", png(&render(Kind::App, 512, false), 512))?;
    let adaptive = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">\n    <background android:drawable=\"@color/ic_launcher_background\"/>\n    <foreground android:drawable=\"@mipmap/ic_launcher_foreground\"/>\n    <monochrome android:drawable=\"@mipmap/ic_launcher_foreground\"/>\n</adaptive-icon>\n";
    w("mipmap-anydpi-v26", "ic_launcher.xml", adaptive.as_bytes().to_vec())?;
    let color = |c: &str| format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<resources>\n    <color name=\"ic_launcher_background\">{c}</color>\n</resources>\n");
    w("values", "ic_launcher_background.xml", color("#FFFFFF").into_bytes())?;
    w("values-night", "ic_launcher_background.xml", color("#1C1C1E").into_bytes())?;
    Ok(())
}
