//! The host system's look: its interface font, its kind of buttons, check boxes, radio buttons,
//! pop-up menus and text fields, and real system windows (with the system's own frame, close and
//! zoom buttons) for FontCraft's dialogs. Sizes follow each system's guidelines:
//! FontCraft shows its text 1 pt larger than the system's own (the person's choice).
//! macOS (13 pt text, 22 pt controls, 14 pt check boxes), Windows 11 (30 px controls, 18 px boxes),
//! Linux/GNOME (30 px controls, 16 px boxes).

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetText};

#[derive(Clone, Copy)]
pub struct Metrics {
    /// Check box / radio button size.
    pub check: f32,
    /// Corner rounding of buttons and fields.
    pub rounding: f32,
    /// Height of a regular control.
    pub control_h: f32,
    /// Body text size.
    pub text: f32,
}

pub fn metrics() -> Metrics {
    if cfg!(target_os = "macos") {
        Metrics { check: 14.0, rounding: 5.0, control_h: 22.0, text: 14.0 }
    } else if cfg!(target_os = "windows") {
        Metrics { check: 18.0, rounding: 4.0, control_h: 30.0, text: 14.0 }
    } else {
        Metrics { check: 16.0, rounding: 6.0, control_h: 30.0, text: 14.0 }
    }
}

/// The system's interface font file (read in place from the system, not copied).
pub fn ui_font_path() -> Option<String> {
    let candidates: &[&str] = if cfg!(target_os = "macos") {
        &["/System/Library/Fonts/SFNS.ttf", "/System/Library/Fonts/HelveticaNeue.ttc"]
    } else if cfg!(target_os = "windows") {
        &["C:\\Windows\\Fonts\\segoeui.ttf"]
    } else {
        &["/usr/share/fonts/truetype/cantarell/Cantarell-VF.otf", "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    };
    for c in candidates {
        if std::path::Path::new(c).exists() {
            return Some(c.to_string());
        }
    }
    if cfg!(all(unix, not(target_os = "macos"))) {
        let o = crate::paths::command("fc-match").args(["-f", "%{file}", "sans-serif"]).output().ok()?;
        let p = String::from_utf8_lossy(&o.stdout).to_string();
        if !p.is_empty() {
            return Some(p);
        }
    }
    None
}

/// Spacing and text sizes like the system's.
pub fn apply_spacing(style: &mut egui::Style) {
    let m = metrics();
    style.spacing.button_padding = Vec2::new(10.0, 3.0);
    style.spacing.interact_size.y = m.control_h;
    style.spacing.item_spacing = Vec2::new(8.0, 7.0);
    style.spacing.icon_width = 16.0;
    style.spacing.icon_width_inner = 10.0;
    style.spacing.combo_width = 120.0;
    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles.insert(TextStyle::Body, FontId::new(m.text, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Button, FontId::new(m.text, FontFamily::Proportional));
    style.text_styles.insert(TextStyle::Small, FontId::new(11.5, FontFamily::Proportional));
}

/// Push buttons, pop-up menus and fields in the system's colours.
pub fn apply_visuals(v: &mut egui::Visuals, dark: bool) {
    let m = metrics();
    let (fill, hover, press, border) = if dark {
        (Color32::from_rgb(0x4b, 0x4b, 0x50), Color32::from_rgb(0x55, 0x55, 0x5a), Color32::from_rgb(0x62, 0x62, 0x68), Color32::from_rgb(0x5a, 0x5a, 0x60))
    } else {
        (Color32::WHITE, Color32::from_rgb(0xf8, 0xf8, 0xf9), Color32::from_rgb(0xe4, 0xe4, 0xe7), Color32::from_rgb(0xc6, 0xc6, 0xcb))
    };
    for (w, f) in [(&mut v.widgets.inactive, fill), (&mut v.widgets.hovered, hover), (&mut v.widgets.active, press), (&mut v.widgets.open, press)] {
        w.weak_bg_fill = f;
        w.bg_stroke = Stroke::new(1.0_f32, border);
        w.corner_radius = egui::CornerRadius::same(m.rounding as u8);
        w.expansion = 0.0;
    }
    v.widgets.inactive.bg_fill = if dark { Color32::from_rgb(0x3a, 0x3a, 0x3f) } else { Color32::from_rgb(0xdc, 0xdc, 0xe0) };
    v.widgets.hovered.bg_fill = v.widgets.inactive.bg_fill;
    v.extreme_bg_color = if dark { Color32::from_rgb(0x2b, 0x2b, 0x30) } else { Color32::WHITE };
    v.menu_corner_radius = egui::CornerRadius::same(7);
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    v.window_shadow = egui::epaint::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(if dark { 110 } else { 50 }) };
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(if dark { 100 } else { 40 }) };
}

fn darker(c: Color32, t: f32) -> Color32 {
    let m = |x: u8| (x as f32 * (1.0 - t)).round() as u8;
    Color32::from_rgb(m(c.r()), m(c.g()), m(c.b()))
}

/// Draws a check box or radio button in the system's style.
fn paint_mark(ui: &Ui, r: Rect, on: bool, pressed: bool, radio: bool) {
    let v = ui.visuals();
    let accent = v.selection.bg_fill;
    let p = ui.painter();
    let round = if cfg!(target_os = "macos") { 3.5 } else { 4.0 };
    if on {
        let fill = if pressed { darker(accent, 0.15) } else { accent };
        if radio {
            p.circle_filled(r.center(), r.width() / 2.0, fill);
            p.circle_filled(r.center(), r.width() * 0.19, Color32::WHITE);
        } else {
            p.rect_filled(r, round, fill);
            let s = r.width();
            let pts = [r.min + Vec2::new(0.24 * s, 0.53 * s), r.min + Vec2::new(0.42 * s, 0.71 * s), r.min + Vec2::new(0.77 * s, 0.30 * s)];
            p.add(egui::Shape::line(pts.to_vec(), Stroke::new(s * 0.13, Color32::WHITE)));
        }
    } else {
        let (fill, border) = if v.dark_mode {
            (if pressed { Color32::from_rgb(0x55, 0x55, 0x5a) } else { Color32::from_rgb(0x3c, 0x3c, 0x41) }, Color32::from_rgb(0x66, 0x66, 0x6c))
        } else {
            (if pressed { Color32::from_rgb(0xe2, 0xe2, 0xe6) } else { Color32::WHITE }, Color32::from_rgb(0xb2, 0xb2, 0xb8))
        };
        if radio {
            p.circle(r.center(), r.width() / 2.0 - 0.5, fill, Stroke::new(1.0_f32, border));
        } else {
            p.rect(r.shrink(0.5), round, fill, Stroke::new(1.0_f32, border), eframe::egui::StrokeKind::Inside);
        }
    }
}

fn mark_widget(ui: &mut Ui, on: bool, text: WidgetText, radio: bool) -> Response {
    let m = metrics();
    let gap = 6.0;
    let galley = text.into_galley(ui, None, (ui.available_width() - m.check - gap).max(40.0), egui::TextStyle::Body);
    let size = Vec2::new(m.check + gap + galley.size().x, galley.size().y.max(m.check + 4.0));
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let kind = if radio { egui::Role::RadioButton } else { egui::Role::CheckBox };
    resp.widget_info(|| WidgetInfo::selected(kind, true, on, galley.text()));
    if ui.is_rect_visible(rect) {
        let mark = Rect::from_min_size(Pos2::new(rect.left(), rect.center().y - m.check / 2.0), Vec2::splat(m.check));
        paint_mark(ui, mark, on, resp.is_pointer_button_down_on(), radio);
        let color = ui.visuals().text_color();
        ui.painter().galley(Pos2::new(mark.right() + gap, rect.center().y - galley.size().y / 2.0), galley, color);
    }
    resp
}

/// A check box like the system's.
pub fn check(ui: &mut Ui, on: &mut bool, text: impl Into<WidgetText>) -> Response {
    let mut resp = mark_widget(ui, *on, text.into(), false);
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    resp
}

/// A radio button like the system's; one of a group (`current` holds the chosen value).
pub fn radio<T: PartialEq>(ui: &mut Ui, current: &mut T, value: T, text: impl Into<WidgetText>) -> Response {
    let mut resp = mark_widget(ui, *current == value, text.into(), true);
    if resp.clicked() && *current != value {
        *current = value;
        resp.mark_changed();
    }
    resp
}

/// A pop-up menu button like the system's (on macOS: the accent-coloured double arrow).
pub fn combo(id_salt: impl egui::AsIdSalt, selected: impl Into<WidgetText>) -> egui::ComboBox<'static> {
    egui::ComboBox::from_id_salt(id_salt).selected_text(selected).icon(|ui, rect, _visuals, _open| paint_dropdown_arrow(ui, rect))
}

/// A pop-up button that opens a menu (which may have submenus), drawn like `combo`.
pub fn menu_popup<R>(ui: &mut Ui, text: &str, width: f32, add: impl FnOnce(&mut Ui) -> R) -> Option<R> {
    let button = egui::Button::new(format!("{text}        ")).min_size(Vec2::new(width, metrics().control_h));
    let (response, inner) = egui::containers::menu::MenuButton::from_button(button).ui(ui, add);
    let r = response.rect;
    paint_dropdown_arrow(ui, Rect::from_center_size(Pos2::new(r.right() - 12.0, r.center().y), Vec2::splat(16.0)));
    inner.map(|i| i.inner)
}

/// A pop-up button like `menu_popup` whose menu is the system's own (macOS, Windows): it opens the
/// given items at the pointer; the chosen item's id comes back on a later frame. Elsewhere None is
/// returned and `fallback` draws FontCraft's own menu instead.
pub fn native_popup<R>(ui: &mut Ui, owner: &str, text: &str, width: f32, items: impl FnOnce() -> Vec<crate::nativemenu::NItem>, fallback: impl FnOnce(&mut Ui) -> R) -> (Option<String>, Option<R>) {
    if !crate::nativemenu::available() {
        return (None, menu_popup(ui, text, width, fallback));
    }
    let r = ui.add(egui::Button::new(format!("{text}        ")).min_size(Vec2::new(width, metrics().control_h)));
    paint_dropdown_arrow(ui, Rect::from_center_size(Pos2::new(r.rect.right() - 12.0, r.rect.center().y), Vec2::splat(16.0)));
    if r.clicked() {
        crate::nativemenu::popup(owner, items());
    }
    (crate::nativemenu::take(owner), None)
}

/// A text field with the system's accent focus ring.
pub fn field(ui: &mut Ui, edit: egui::TextEdit) -> Response {
    let r = ui.add(edit);
    if r.has_focus() {
        let ring = ui.visuals().selection.bg_fill.gamma_multiply(0.55);
        ui.painter().rect_stroke(r.rect.expand(2.0), metrics().rounding + 2.0, Stroke::new(3.0_f32, ring), eframe::egui::StrokeKind::Inside);
    }
    r
}

/// The dialog's main (default) button: filled with the accent colour.
pub fn default_button(ui: &mut Ui, enabled: bool, text: &str) -> Response {
    let accent = ui.visuals().selection.bg_fill;
    ui.add_enabled(enabled, egui::Button::new(egui::RichText::new(text).color(Color32::WHITE)).fill(accent))
}

/// A dialog in its own system window (system frame, title, close button), placed over the main
/// window and sized to fit its content every frame, so it grows or shrinks when its content changes.
/// `fixed_h` keeps a set height instead (for content that scrolls on its own).
/// Returns what `add` returned and whether the dialog was closed (close button or Esc).
/// While taking test pictures, dialogs are drawn inside the main window instead.
pub fn dialog<R>(ctx: &egui::Context, key: &str, title: &str, width: f32, fill: Color32, fixed_h: Option<f32>, add: impl FnOnce(&mut Ui) -> R) -> (Option<R>, bool) {
    dialog_at(ctx, key, title, width, fill, fixed_h, false, add)
}

/// Like `dialog`, but centred on the screen (kept centred while it sizes itself to its content,
/// until the person moves it).
pub fn dialog_centered<R>(ctx: &egui::Context, key: &str, title: &str, width: f32, fill: Color32, fixed_h: Option<f32>, add: impl FnOnce(&mut Ui) -> R) -> (Option<R>, bool) {
    dialog_at(ctx, key, title, width, fill, fixed_h, true, add)
}

#[allow(clippy::too_many_arguments)]
fn dialog_at<R>(ctx: &egui::Context, key: &str, title: &str, width: f32, fill: Color32, fixed_h: Option<f32>, centered: bool, add: impl FnOnce(&mut Ui) -> R) -> (Option<R>, bool) {
    let vid = egui::ViewportId::from_hash_of(("dialog", key));
    let hid = egui::Id::unique(("dialog-h", key));
    let pid = egui::Id::unique(("dialog-pos", key));
    let margin = 20.0;
    let h: f32 = fixed_h.unwrap_or_else(|| ctx.data(|d| d.get_temp(hid)).unwrap_or(320.0));
    // Centred: the middle of the screen FontCraft's window is on (the main screen starts at 0, 0;
    // on another screen, the middle of FontCraft's window).
    let centre = || -> Option<Pos2> {
        let outer = ctx.input(|i| i.viewport().outer_rect)?;
        let screen = ctx.input(|i| i.viewport().monitor_size);
        let c = match screen {
            Some(m) if outer.center().x >= 0.0 && outer.center().y >= 0.0 && outer.center().x <= m.x && outer.center().y <= m.y => Pos2::new(m.x / 2.0, m.y / 2.0),
            _ => outer.center(),
        };
        Some(Pos2::new(c.x - width / 2.0, (c.y - h / 2.0).max(30.0)))
    };
    let pos: Option<Pos2> = if ctx.embed_viewports() {
        // Drawn inside FontCraft's window (test pictures): placed in the window's own coordinates.
        let r = ctx.content_rect();
        Some(if centered { Pos2::new(r.center().x - width / 2.0, (r.center().y - h / 2.0).max(r.top() + 20.0)) } else { Pos2::new(r.center().x - width / 2.0, r.top() + 60.0) })
    } else {
        if centered { centre() } else { None }.or_else(|| ctx.data(|d| d.get_temp(pid))).or_else(|| {
            let outer = ctx.input(|i| i.viewport().outer_rect)?;
            let p = Pos2::new(outer.center().x - width / 2.0, outer.top() + 70.0);
            ctx.data_mut(|d| d.insert_temp(pid, p));
            Some(p)
        })
    };
    let mut builder = egui::ViewportBuilder::default()
        .with_title(title)
        .with_inner_size([width, h])
        .with_resizable(false)
        .with_minimize_button(false)
        .with_maximize_button(false);
    if let Some(p) = pos {
        builder = builder.with_position(p);
    }
    let mut add = Some(add);
    let mut out = None;
    let mut closed = false;
    // Test pictures (one window): the dialog is drawn as a window inside FontCraft's, centred or
    // near the top, since egui's own embedded viewport window can't be placed.
    if ctx.embed_viewports() {
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::unique(("dialog-window", key)))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .min_width(width - 2.0 * margin)
            .max_width(width - 2.0 * margin)
            .anchor(if centered { egui::Align2::CENTER_CENTER } else { egui::Align2::CENTER_TOP }, Vec2::new(0.0, if centered { 0.0 } else { 40.0 }))
            .frame(egui::Frame::window(&ctx.global_style()).fill(fill).inner_margin(egui::Margin::same(margin as i8)))
            .show(ctx, |ui| {
                if let Some(add) = add.take() {
                    out = Some(add(ui));
                }
            });
        return (out, !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)));
    }
    let _ = vid;
    ctx.show_viewport_immediate(vid, builder, |vui, class| {
        let ctx = &vui.ctx().clone();
        let Some(add) = add.take() else { return };
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            closed = true;
        }
        if class == egui::ViewportClass::EmbeddedWindow {
            // egui (0.37) already wraps an embedded viewport in a window of its own, placed from
            // the builder; the dialog's content goes straight into it.
            egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::same((margin) as i8)).show(vui, |ui| {
                ui.set_width(width - 2.0 * margin);
                out = Some(add(ui));
            });
            return;
        }
        if ctx.input(|i| i.viewport().close_requested()) {
            closed = true;
        }
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::same((margin) as i8))).show(vui, |ui| {
            if fixed_h.is_some() {
                out = Some(add(ui));
                return;
            }
            let max_h = ctx.input(|i| i.viewport().monitor_size).map(|m| m.y - 160.0).unwrap_or(860.0).max(300.0);
            let o = egui::ScrollArea::vertical().id_salt(("dialog-scroll", key)).auto_shrink([false, true]).show(ui, add);
            out = Some(o.inner);
            let want = (o.content_size.y + 2.0 * margin + 2.0).clamp(140.0, max_h);
            if (want - h).abs() > 1.0 {
                ctx.data_mut(|d| d.insert_temp(hid, want));
                ctx.request_repaint_of(egui::ViewportId::ROOT);
            }
        });
    });
    (out, closed)
}

/// Where a dialog's content is, for placing something beside it: the left edge of its window and
/// the top of its content, in screen points (in test pictures, in FontCraft's window, where the
/// dialog is drawn inside).
pub fn dialog_origin(ui: &Ui) -> (f32, f32) {
    let ctx = ui.ctx();
    if ctx.embed_viewports() {
        let r = ui.max_rect();
        return (r.left() - 20.0, 0.0);
    }
    let (outer, inner) = ctx.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect));
    match (outer, inner) {
        (Some(o), Some(i)) => (o.left(), i.top()),
        _ => (0.0, 0.0),
    }
}

/// A small borderless window just left of `left_edge` (outside the window it belongs to, so it
/// covers nothing there), centred on `y` and kept on screen; it takes no clicks or focus and goes
/// away when it is no longer called. Coordinates are screen points (window points in test pictures).
pub fn float_beside(ctx: &egui::Context, key: &str, left_edge: f32, y: f32, size: Vec2, fill: Color32, add: impl FnOnce(&mut Ui)) {
    let gap = 10.0;
    let mut pos = Pos2::new(left_edge - gap - size.x, y - size.y / 2.0);
    pos.y = pos.y.max(if ctx.embed_viewports() { ctx.content_rect().top() } else { 30.0 });
    if let Some(m) = ctx.input(|i| i.viewport().monitor_size) {
        pos.y = pos.y.min(m.y - size.y - 10.0).max(30.0);
    }
    pos.x = pos.x.max(0.0);
    if ctx.embed_viewports() {
        egui::Area::new(egui::Id::unique(("float-beside", key))).fixed_pos(pos).order(egui::Order::Tooltip).interactable(false).show(ctx, |ui| {
            egui::Frame::popup(&ctx.global_style()).fill(fill).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
                ui.set_min_size(size - Vec2::splat(16.0));
                add(ui);
            });
        });
        return;
    }
    let builder = egui::ViewportBuilder::default()
        .with_title(key)
        .with_decorations(false)
        .with_resizable(false)
        .with_position(pos)
        .with_inner_size(size)
        .with_always_on_top()
        .with_taskbar(false)
        .with_active(false)
        .with_mouse_passthrough(true);
    let mut add = Some(add);
    ctx.show_viewport_immediate(egui::ViewportId::from_hash_of(("float-beside", key)), builder, |vui, _| {
        let Some(add) = add.take() else { return };
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(fill).inner_margin(egui::Margin::same(8))).show(vui, add);
    });
}

/// The system's accent colour (the one its buttons and selections use), read again every few
/// seconds so a change in the system's settings shows: macOS `NSColor.controlAccentColor`, Windows
/// the accent colour of its Colours settings (DWM `AccentColor`), GNOME 47+ `accent-color`.
/// None where the system has none to give (FontCraft's blue stands in).
pub fn system_accent() -> Option<Color32> {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static CACHE: Mutex<Option<(Instant, Option<Color32>)>> = Mutex::new(None);
    let mut c = CACHE.lock().ok()?;
    let every = if cfg!(target_os = "macos") { Duration::from_secs(2) } else { Duration::from_secs(10) };
    if let Some((t, v)) = *c {
        if t.elapsed() < every {
            return v;
        }
    }
    let v = read_accent();
    *c = Some((Instant::now(), v));
    v
}

#[cfg(target_os = "macos")]
fn read_accent() -> Option<Color32> {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    unsafe {
        let color: *mut AnyObject = msg_send![class!(NSColor), controlAccentColor];
        let space: *mut AnyObject = msg_send![class!(NSColorSpace), sRGBColorSpace];
        if color.is_null() || space.is_null() {
            return None;
        }
        let rgb: *mut AnyObject = msg_send![color, colorUsingColorSpace: space];
        if rgb.is_null() {
            return None;
        }
        let (r, g, b): (f64, f64, f64) = (msg_send![rgb, redComponent], msg_send![rgb, greenComponent], msg_send![rgb, blueComponent]);
        let u = |x: f64| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some(Color32::from_rgb(u(r), u(g), u(b)))
    }
}

#[cfg(target_os = "windows")]
fn read_accent() -> Option<Color32> {
    // "AccentColor    REG_DWORD    0xff3c78d8": the bytes are alpha, blue, green, red.
    let o = crate::paths::command("reg").args(["query", r"HKCU\Software\Microsoft\Windows\DWM", "/v", "AccentColor"]).output().ok()?;
    let s = String::from_utf8_lossy(&o.stdout);
    let hex = s.split_whitespace().find(|w| w.starts_with("0x"))?;
    let v = u32::from_str_radix(hex.trim_start_matches("0x"), 16).ok()?;
    Some(Color32::from_rgb(v as u8, (v >> 8) as u8, (v >> 16) as u8))
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
fn read_accent() -> Option<Color32> {
    // GNOME's named accents (libadwaita's colours).
    let o = crate::paths::command("gsettings").args(["get", "org.gnome.desktop.interface", "accent-color"]).output().ok()?;
    let name = String::from_utf8_lossy(&o.stdout).trim().trim_matches('\'').to_string();
    let v: u32 = match name.as_str() {
        "blue" => 0x3584e4,
        "teal" => 0x2190a4,
        "green" => 0x3a944a,
        "yellow" => 0xc88800,
        "orange" => 0xed5b00,
        "red" => 0xe62d42,
        "pink" => 0xd56199,
        "purple" => 0x9141ac,
        "slate" => 0x6f8396,
        _ => return None,
    };
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

#[cfg(target_os = "android")]
fn read_accent() -> Option<Color32> {
    None
}

/// The system's own drop-down arrow as an alpha mask (white, tinted when drawn), `px` pixels tall:
/// macOS the SF Symbol "chevron.up.chevron.down" its pop-up buttons use (drawn by AppKit);
/// Windows the ChevronDown glyph (U+E70D) of Segoe Fluent Icons (Windows 11) or Segoe MDL2
/// Assets (Windows 10); Linux the icon theme's "pan-down-symbolic" (Adwaita's when the theme has
/// none). None when the system can't give one (FontCraft then draws the same shape itself).
/// Which of the system's arrows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Arrow {
    /// A pop-up button's (macOS up-and-down chevron; elsewhere a down chevron).
    PopUp,
    /// An open disclosure (pointing down).
    Down,
    /// A closed disclosure (pointing right).
    Right,
}

fn system_arrow_image(kind: Arrow, px: u32) -> Option<egui::ColorImage> {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::NSString;
        let run = || unsafe {
            let name = NSString::from_str(match kind {
                Arrow::PopUp => "chevron.up.chevron.down",
                Arrow::Down => "chevron.down",
                Arrow::Right => "chevron.right",
            });
            sf_symbol_mask(&name, px, 0.0)
        };
        objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
    }
    #[cfg(target_os = "windows")]
    {
        win_arrow(kind, px)
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        linux_arrow(kind, px)
    }
    #[cfg(target_os = "android")]
    {
        let _ = (kind, px);
        None
    }
}

/// An SF Symbol drawn by AppKit as a white alpha mask `px` pixels tall (`weight` 0 = the symbol's
/// regular weight; otherwise an NSFontWeight such as 0.23 for semibold).
#[cfg(target_os = "macos")]
unsafe fn sf_symbol_mask(name: &objc2_foundation::NSString, px: u32, weight: f64) -> Option<egui::ColorImage> {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    {
        {
            let mut img: *mut AnyObject = msg_send![class!(NSImage), imageWithSystemSymbolName: name, accessibilityDescription: std::ptr::null::<AnyObject>()];
            if !img.is_null() && weight != 0.0 {
                let cfg: *mut AnyObject = msg_send![class!(NSImageSymbolConfiguration), configurationWithPointSize: px as f64, weight: weight];
                if !cfg.is_null() {
                    let w: *mut AnyObject = msg_send![img, imageWithSymbolConfiguration: cfg];
                    if !w.is_null() {
                        img = w;
                    }
                }
            }
            if img.is_null() {
                return None;
            }
            let size: NSSize = msg_send![img, size];
            if size.height <= 0.0 {
                return None;
            }
            let h = px as isize;
            let w = ((size.width / size.height) * px as f64).ceil().max(1.0) as isize;
            let space = NSString::from_str("NSDeviceRGBColorSpace");
            let rep: *mut AnyObject = msg_send![class!(NSBitmapImageRep), alloc];
            let rep: *mut AnyObject = msg_send![rep, initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>(), pixelsWide: w, pixelsHigh: h, bitsPerSample: 8isize, samplesPerPixel: 4isize, hasAlpha: true, isPlanar: false, colorSpaceName: &*space, bytesPerRow: w * 4, bitsPerPixel: 32isize];
            if rep.is_null() {
                return None;
            }
            let ctx: *mut AnyObject = msg_send![class!(NSGraphicsContext), graphicsContextWithBitmapImageRep: rep];
            let _: () = msg_send![class!(NSGraphicsContext), saveGraphicsState];
            let _: () = msg_send![class!(NSGraphicsContext), setCurrentContext: ctx];
            let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w as f64, h as f64));
            let _: () = msg_send![img, drawInRect: rect];
            let _: () = msg_send![class!(NSGraphicsContext), restoreGraphicsState];
            let data: *const u8 = msg_send![rep, bitmapData];
            if data.is_null() {
                return None;
            }
            let bytes = std::slice::from_raw_parts(data, (w * h * 4) as usize);
            // A template symbol is drawn in black: its alpha is the shape.
            let pixels = bytes.chunks_exact(4).map(|p| Color32::from_white_alpha(p[3])).collect();
            Some(egui::ColorImage::new([w as usize, h as usize], pixels))
        }
    }
}

#[cfg(target_os = "windows")]
fn win_arrow(kind: Arrow, px: u32) -> Option<egui::ColorImage> {
    {
        use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
        let data = ["C:\\Windows\\Fonts\\SegoeIcons.ttf", "C:\\Windows\\Fonts\\segmdl2.ttf"].iter().find_map(|p| std::fs::read(p).ok())?;
        let font = FontRef::try_from_slice(&data).ok()?;
        // ChevronDown E70D, ChevronRight E76C.
        let id = font.glyph_id(if kind == Arrow::Right { '\u{E76C}' } else { '\u{E70D}' });
        let scale = PxScale::from(px as f32);
        let glyph = id.with_scale_and_position(scale, ab_glyph::point(0.0, font.as_scaled(scale).ascent()));
        let o = font.outline_glyph(glyph)?;
        let b = o.px_bounds();
        let (w, h) = (b.width().ceil().max(1.0) as usize, b.height().ceil().max(1.0) as usize);
        let mut img = egui::ColorImage::filled([w, h], Color32::TRANSPARENT);
        o.draw(|x, y, c| {
            let (x, y) = (x as usize, y as usize);
            if x < w && y < h {
                img.pixels[y * w + x] = Color32::from_white_alpha((c.clamp(0.0, 1.0) * 255.0) as u8);
            }
        });
        Some(img)
    }
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
fn linux_arrow(kind: Arrow, px: u32) -> Option<egui::ColorImage> {
    {
        let theme = crate::paths::command("gsettings").args(["get", "org.gnome.desktop.interface", "icon-theme"]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().trim_matches('\'').to_string()).unwrap_or_default();
        let mut candidates = Vec::new();
        for t in [theme.as_str(), "Adwaita"] {
            if t.is_empty() {
                continue;
            }
            for sub in ["symbolic/ui", "scalable/ui", "16x16/ui", "scalable/actions"] {
                let icon = if kind == Arrow::Right { "pan-end-symbolic" } else { "pan-down-symbolic" };
                candidates.push(format!("/usr/share/icons/{t}/{sub}/{icon}.svg"));
            }
        }
        let svg = candidates.iter().find_map(|p| std::fs::read_to_string(p).ok())?;
        use resvg::{tiny_skia, usvg};
        let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
        let s = px;
        let mut pix = tiny_skia::Pixmap::new(s, s)?;
        let k = s as f32 / tree.size().height().max(1.0);
        resvg::render(&tree, tiny_skia::Transform::from_scale(k, k), &mut pix.as_mut());
        let pixels = pix.pixels().iter().map(|p| Color32::from_white_alpha(p.alpha())).collect();
        Some(egui::ColorImage::new([s as usize, s as usize], pixels))
    }
}

/// A system symbol (macOS: the SF Symbol `name`, the same pictures Finder's sidebar uses) drawn
/// `pt` points tall, centred at `c` and tinted `tint`. Returns false when the system has none, so
/// the caller draws its own picture instead.
pub fn symbol(ui: &Ui, name: &str, c: Pos2, pt: f32, tint: Color32) -> bool {
    let ctx = ui.ctx();
    let ppp = ctx.pixels_per_point();
    let px = (pt * ppp).round().max(4.0) as u32;
    let id = egui::Id::unique(("sf-symbol", name, px));
    let tex = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let img = symbol_image(name, px);
        let t = img.map(|img| ctx.load_texture(format!("sf-{name}"), img, egui::TextureOptions::LINEAR));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    match tex {
        Some(t) => {
            let mut size = t.size_vec2() / ppp;
            // Wide symbols keep to the same box as the others.
            let max_w = pt * 1.35;
            if size.x > max_w {
                size *= max_w / size.x;
            }
            ui.painter().image(t.id(), Rect::from_center_size(c, size), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
            true
        }
        None => false,
    }
}

fn symbol_image(name: &str, px: u32) -> Option<egui::ColorImage> {
    #[cfg(target_os = "macos")]
    {
        let run = || unsafe { sf_symbol_mask(&objc2_foundation::NSString::from_str(name), px, 0.0) };
        objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (name, px);
        None
    }
}

/// The system's arrow image as a cached texture (None when the system gives none).
fn arrow_texture(ctx: &egui::Context, kind: Arrow, px: u32) -> Option<egui::TextureHandle> {
    let id = egui::Id::unique(("system-arrow", kind, px));
    ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = system_arrow_image(kind, px).map(|img| ctx.load_texture("system-arrow", img, egui::TextureOptions::LINEAR));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    })
}

/// The system's disclosure arrow (chevron right when closed, down when open), 9 points, centred
/// at `c` in `ink`: every fold arrow in FontCraft (sidebar sections, libraries and folders, font
/// families, details panel) is this one, so they all look alike.
pub fn disclosure(ui: &Ui, c: Pos2, open: bool, ink: Color32) {
    let ppp = ui.ctx().pixels_per_point();
    let px = (9.0 * ppp).round() as u32;
    let kind = if open { Arrow::Down } else { Arrow::Right };
    match arrow_texture(ui.ctx(), kind, px) {
        Some(t) => {
            let size = t.size_vec2() / ppp;
            ui.painter().image(t.id(), Rect::from_center_size(c, size), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), ink);
        }
        None => {
            let st = Stroke::new(1.3_f32, ink);
            if open {
                ui.painter().line_segment([c + Vec2::new(-3.0, -1.5), c + Vec2::new(0.0, 1.5)], st);
                ui.painter().line_segment([c + Vec2::new(0.0, 1.5), c + Vec2::new(3.0, -1.5)], st);
            } else {
                ui.painter().line_segment([c + Vec2::new(-1.5, -3.0), c + Vec2::new(1.5, 0.0)], st);
                ui.painter().line_segment([c + Vec2::new(1.5, 0.0), c + Vec2::new(-1.5, 3.0)], st);
            }
        }
    }
}


/// Draws the system's drop-down arrow centred in `rect` (macOS: on its accent-coloured square, as
/// its pop-up buttons show it); FontCraft's own drawing of the same shape where the system gives none.
pub fn paint_dropdown_arrow(ui: &Ui, rect: Rect) {
    let ctx = ui.ctx();
    let ppp = ctx.pixels_per_point();
    let px = (if cfg!(target_os = "macos") { 11.0 } else { 12.0 } * ppp).round() as u32;
    let tex = arrow_texture(ctx, Arrow::PopUp, px);
    let c = rect.center();
    let p = ui.painter();
    let accent = ui.visuals().selection.bg_fill;
    let ink = if cfg!(target_os = "macos") {
        p.rect_filled(Rect::from_center_size(c, Vec2::splat(16.0)), 4.0, accent);
        Color32::WHITE
    } else {
        ui.visuals().text_color()
    };
    match tex {
        Some(t) => {
            let size = t.size_vec2() / ppp;
            p.image(t.id(), Rect::from_center_size(c, size), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), ink);
        }
        None => {
            let st = Stroke::new(1.4_f32, ink);
            if cfg!(target_os = "macos") {
                p.line_segment([c + Vec2::new(-3.0, -1.5), c + Vec2::new(0.0, -4.5)], st);
                p.line_segment([c + Vec2::new(0.0, -4.5), c + Vec2::new(3.0, -1.5)], st);
                p.line_segment([c + Vec2::new(-3.0, 1.5), c + Vec2::new(0.0, 4.5)], st);
                p.line_segment([c + Vec2::new(0.0, 4.5), c + Vec2::new(3.0, 1.5)], st);
            } else {
                p.line_segment([c + Vec2::new(-4.0, -2.0), c + Vec2::new(0.0, 2.0)], st);
                p.line_segment([c + Vec2::new(0.0, 2.0), c + Vec2::new(4.0, -2.0)], st);
            }
        }
    }
}

/// The host system's own interface colours for its light or dark look.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SysColors {
    /// Window and sidebar background.
    pub window: Color32,
    /// Lists, documents and panels (the content background).
    pub content: Color32,
    /// The sidebar.
    pub sidebar: Color32,
    /// Text.
    pub label: Color32,
    /// Secondary text.
    pub secondary: Color32,
    /// Lines between parts.
    pub separator: Color32,
}

/// The system's colours for light (`dark` false) or dark: macOS reads its named colours from
/// AppKit (windowBackgroundColor, controlBackgroundColor, labelColor, secondaryLabelColor,
/// separatorColor) resolved in the Aqua or Dark Aqua appearance; Windows uses Windows 11's Fluent
/// values and Linux GNOME's libadwaita values (the colours those systems' own apps use).
pub fn system_colors(dark: bool) -> SysColors {
    use std::sync::Mutex;
    static CACHE: Mutex<[Option<SysColors>; 2]> = Mutex::new([None, None]);
    if let Ok(c) = CACHE.lock() {
        if let Some(v) = c[dark as usize] {
            return v;
        }
    }
    let v = read_system_colors(dark);
    if let Ok(mut c) = CACHE.lock() {
        c[dark as usize] = Some(v);
    }
    v
}

fn rgba_u8(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0).round() as u8)
}

#[cfg(target_os = "macos")]
fn read_system_colors(dark: bool) -> SysColors {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;
    let fallback = fallback_colors(dark);
    let run = || unsafe {
        let name = NSString::from_str(if dark { "NSAppearanceNameDarkAqua" } else { "NSAppearanceNameAqua" });
        let appearance: *mut AnyObject = msg_send![class!(NSAppearance), appearanceNamed: &*name];
        if appearance.is_null() {
            return fallback;
        }
        let previous: *mut AnyObject = msg_send![class!(NSAppearance), currentAppearance];
        let _: () = msg_send![class!(NSAppearance), setCurrentAppearance: appearance];
        let space: *mut AnyObject = msg_send![class!(NSColorSpace), sRGBColorSpace];
        let read = |c: *mut AnyObject, or: Color32| -> Color32 {
            if c.is_null() {
                return or;
            }
            let rgb: *mut AnyObject = msg_send![c, colorUsingColorSpace: space];
            if rgb.is_null() {
                return or;
            }
            let (r, g, b, a): (f64, f64, f64, f64) = (msg_send![rgb, redComponent], msg_send![rgb, greenComponent], msg_send![rgb, blueComponent], msg_send![rgb, alphaComponent]);
            let u = |x: f64| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
            rgba_u8(u(r), u(g), u(b), a as f32)
        };
        let window = read(msg_send![class!(NSColor), windowBackgroundColor], fallback.window);
        let content = read(msg_send![class!(NSColor), controlBackgroundColor], fallback.content);
        let label = read(msg_send![class!(NSColor), labelColor], fallback.label);
        let secondary = read(msg_send![class!(NSColor), secondaryLabelColor], fallback.secondary);
        let separator = read(msg_send![class!(NSColor), separatorColor], fallback.separator);
        let _: () = msg_send![class!(NSAppearance), setCurrentAppearance: previous];
        let sidebar = if dark { Color32::from_rgb(0x24, 0x24, 0x26) } else { Color32::from_rgb(0xec, 0xec, 0xec) };
        SysColors { window, content, sidebar, label, secondary, separator }
    };
    objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or(fallback)
}

#[cfg(not(target_os = "macos"))]
fn read_system_colors(dark: bool) -> SysColors {
    fallback_colors(dark)
}

/// The published colours of each system's own design (used where they can't be read).
fn fallback_colors(dark: bool) -> SysColors {
    if cfg!(target_os = "windows") {
        // Windows 11 Fluent: Mica base, layer, text primary/secondary, divider.
        if dark {
            SysColors { window: Color32::from_rgb(0x20, 0x20, 0x20), content: Color32::from_rgb(0x2b, 0x2b, 0x2b), sidebar: Color32::from_rgb(0x20, 0x20, 0x20), label: Color32::WHITE, secondary: rgba_u8(255, 255, 255, 0.786), separator: rgba_u8(255, 255, 255, 0.084) }
        } else {
            SysColors { window: Color32::from_rgb(0xf3, 0xf3, 0xf3), content: Color32::WHITE, sidebar: Color32::from_rgb(0xf3, 0xf3, 0xf3), label: rgba_u8(0, 0, 0, 0.894), secondary: rgba_u8(0, 0, 0, 0.62), separator: rgba_u8(0, 0, 0, 0.08) }
        }
    } else if cfg!(target_os = "macos") {
        // macOS Finder / Source List: light sidebar #ECECEC on #FFFFFF content; dark #242426 on #1E1E1E content.
        if dark {
            SysColors { window: Color32::from_rgb(0x24, 0x24, 0x26), content: Color32::from_rgb(0x1e, 0x1e, 0x1e), sidebar: Color32::from_rgb(0x24, 0x24, 0x26), label: rgba_u8(255, 255, 255, 0.85), secondary: rgba_u8(255, 255, 255, 0.55), separator: rgba_u8(255, 255, 255, 0.1) }
        } else {
            SysColors { window: Color32::from_rgb(0xec, 0xec, 0xec), content: Color32::WHITE, sidebar: Color32::from_rgb(0xec, 0xec, 0xec), label: rgba_u8(0, 0, 0, 0.85), secondary: rgba_u8(0, 0, 0, 0.5), separator: rgba_u8(0, 0, 0, 0.1) }
        }
    } else if dark {
        // GNOME libadwaita: window, view and sidebar backgrounds, foreground, borders.
        SysColors { window: Color32::from_rgb(0x22, 0x22, 0x26), content: Color32::from_rgb(0x1d, 0x1d, 0x20), sidebar: Color32::from_rgb(0x2e, 0x2e, 0x32), label: Color32::WHITE, secondary: rgba_u8(255, 255, 255, 0.55), separator: rgba_u8(255, 255, 255, 0.15) }
    } else {
        SysColors { window: Color32::from_rgb(0xfa, 0xfa, 0xfb), content: Color32::WHITE, sidebar: Color32::from_rgb(0xeb, 0xeb, 0xed), label: rgba_u8(0, 0, 6, 0.8), secondary: rgba_u8(0, 0, 6, 0.55), separator: rgba_u8(0, 0, 6, 0.15) }
    }
}

/// Text written by the system's own text engine, as Finder writes its sidebar (macOS: AppKit's
/// system font at `pt` points and `weight`, an NSFontWeight: 0 regular, 0.3 semibold). egui's copy
/// of the same font draws the large-size design, about 12 % narrower at 13 points than what Finder
/// shows ("Todas las fuentes": 93.8 against AppKit's 106.1 points), so the sidebar asks the system.
/// Placed by `align` at `pos`, tinted `color`; None where the system can't (FontCraft then draws it).
pub fn sys_text(ui: &Ui, pos: Pos2, align: egui::Align2, text: &str, pt: f32, weight: f32, color: Color32) -> Option<Rect> {
    if text.is_empty() {
        return None;
    }
    let ctx = ui.ctx();
    let ppp = ctx.pixels_per_point();
    let px = (pt * ppp * 4.0).round() as u32;
    let id = egui::Id::unique(("sys-text", text, px, (weight * 100.0) as i32));
    let tex = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = sys_text_image(text, pt as f64 * ppp as f64, weight as f64).map(|img| ctx.load_texture("sys-text", img, egui::TextureOptions::LINEAR));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    let t = tex?;
    let size = t.size_vec2() / ppp;
    let rect = align.anchor_size(pos, size);
    ui.painter().image(t.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), color);
    Some(rect)
}

/// How wide the system writes `text` (see `sys_text`), in points.
pub fn sys_text_width(text: &str, pt: f32, weight: f32) -> Option<f32> {
    use std::collections::HashMap;
    use std::sync::Mutex;
    static CACHE: Mutex<Option<HashMap<(String, u32, i32), f32>>> = Mutex::new(None);
    let key = (text.to_string(), (pt * 100.0) as u32, (weight * 100.0) as i32);
    if let Ok(mut c) = CACHE.lock() {
        if let Some(w) = c.get_or_insert_with(HashMap::new).get(&key) {
            return Some(*w);
        }
    }
    let w = sys_text_measure(text, pt as f64, weight as f64)?;
    if let Ok(mut c) = CACHE.lock() {
        c.get_or_insert_with(HashMap::new).insert(key, w);
    }
    Some(w)
}

#[cfg(target_os = "macos")]
unsafe fn sys_text_attrs(pt: f64, weight: f64) -> *mut objc2::runtime::AnyObject {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    let font: *mut AnyObject = msg_send![class!(NSFont), systemFontOfSize: pt, weight: weight];
    if font.is_null() {
        return std::ptr::null_mut();
    }
    let key = objc2_foundation::NSString::from_str("NSFont");
    msg_send![class!(NSDictionary), dictionaryWithObject: font, forKey: &*key]
}

#[cfg(target_os = "macos")]
fn sys_text_measure(text: &str, pt: f64, weight: f64) -> Option<f32> {
    use objc2::msg_send;
    use objc2_foundation::{NSSize, NSString};
    let run = || unsafe {
        let attrs = sys_text_attrs(pt, weight);
        if attrs.is_null() {
            return None;
        }
        let s = NSString::from_str(text);
        let size: NSSize = msg_send![&*s, sizeWithAttributes: attrs];
        Some(size.width as f32)
    };
    objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
}

#[cfg(target_os = "macos")]
fn sys_text_image(text: &str, px: f64, weight: f64) -> Option<egui::ColorImage> {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSPoint, NSSize, NSString};
    let run = || unsafe {
        let attrs = sys_text_attrs(px, weight);
        if attrs.is_null() {
            return None;
        }
        let s = NSString::from_str(text);
        let size: NSSize = msg_send![&*s, sizeWithAttributes: attrs];
        let w = (size.width.ceil() as isize + 2).max(1);
        let h = (size.height.ceil() as isize).max(1);
        let space = NSString::from_str("NSDeviceRGBColorSpace");
        let rep: *mut AnyObject = msg_send![class!(NSBitmapImageRep), alloc];
        let rep: *mut AnyObject = msg_send![rep, initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>(), pixelsWide: w, pixelsHigh: h, bitsPerSample: 8isize, samplesPerPixel: 4isize, hasAlpha: true, isPlanar: false, colorSpaceName: &*space, bytesPerRow: w * 4, bitsPerPixel: 32isize];
        if rep.is_null() {
            return None;
        }
        let gctx: *mut AnyObject = msg_send![class!(NSGraphicsContext), graphicsContextWithBitmapImageRep: rep];
        let _: () = msg_send![class!(NSGraphicsContext), saveGraphicsState];
        let _: () = msg_send![class!(NSGraphicsContext), setCurrentContext: gctx];
        let _: () = msg_send![&*s, drawAtPoint: NSPoint::new(1.0, 0.0), withAttributes: attrs];
        let _: () = msg_send![class!(NSGraphicsContext), restoreGraphicsState];
        let data: *const u8 = msg_send![rep, bitmapData];
        if data.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(data, (w * h * 4) as usize);
        let pixels = bytes.chunks_exact(4).map(|p| Color32::from_white_alpha(p[3])).collect();
        Some(egui::ColorImage::new([w as usize, h as usize], pixels))
    };
    objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
}

#[cfg(not(target_os = "macos"))]
fn sys_text_measure(_text: &str, _pt: f64, _weight: f64) -> Option<f32> {
    None
}

#[cfg(not(target_os = "macos"))]
fn sys_text_image(_text: &str, _px: f64, _weight: f64) -> Option<egui::ColorImage> {
    None
}

/// The sidebar size chosen in System Settings › Appearance (macOS NSTableViewDefaultSizeMode:
/// 1 small, 2 medium, 3 large), which sets the size of Finder's sidebar text and icons. Read again
/// every few seconds, so a change there shows here too. Medium elsewhere.
pub fn sidebar_size_mode() -> u8 {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static CACHE: Mutex<Option<(Instant, u8)>> = Mutex::new(None);
    if let Ok(c) = CACHE.lock() {
        if let Some((t, v)) = *c {
            if t.elapsed() < Duration::from_secs(5) {
                return v;
            }
        }
    }
    let v = read_sidebar_size_mode().clamp(1, 3);
    if let Ok(mut c) = CACHE.lock() {
        *c = Some((Instant::now(), v));
    }
    v
}

#[cfg(target_os = "macos")]
fn read_sidebar_size_mode() -> u8 {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    let run = || unsafe {
        let d: *mut AnyObject = msg_send![class!(NSUserDefaults), standardUserDefaults];
        if d.is_null() {
            return 2;
        }
        let key = objc2_foundation::NSString::from_str("NSTableViewDefaultSizeMode");
        let v: isize = msg_send![d, integerForKey: &*key];
        if v == 0 { 2 } else { v as u8 }
    };
    objc2::exception::catch(std::panic::AssertUnwindSafe(run)).unwrap_or(2)
}

#[cfg(not(target_os = "macos"))]
fn read_sidebar_size_mode() -> u8 {
    2
}

/// A system symbol sized to go with text of `pt` points, as Finder's sidebar sizes its icons to
/// its text (macOS: the SF Symbol in a point-size configuration at its large scale, drawn at its
/// own width and height). Returns the rect it took, or None (the caller draws its own picture).
pub fn symbol_for_text(ui: &Ui, name: &str, c: Pos2, pt: f32, tint: Color32) -> Option<Rect> {
    let ctx = ui.ctx();
    let ppp = ctx.pixels_per_point();
    let id = egui::Id::unique(("sf-symbol-text", name, (pt * 10.0) as u32, (ppp * 10.0) as u32));
    let tex = ctx.data_mut(|d| d.get_temp::<Option<(egui::TextureHandle, Vec2)>>(id)).unwrap_or_else(|| {
        let t = symbol_text_image(name, pt as f64, ppp as f64).map(|(img, size)| (ctx.load_texture(format!("sf-{name}"), img, egui::TextureOptions::LINEAR), size));
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    let (t, size) = tex?;
    let rect = Rect::from_center_size(c, size);
    ui.painter().image(t.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), tint);
    Some(rect)
}

#[cfg(target_os = "macos")]
fn symbol_text_image(name: &str, pt: f64, ppp: f64) -> Option<(egui::ColorImage, Vec2)> {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
    let run = || unsafe {
        let n = NSString::from_str(name);
        let img: *mut AnyObject = msg_send![class!(NSImage), imageWithSystemSymbolName: &*n, accessibilityDescription: std::ptr::null::<AnyObject>()];
        if img.is_null() {
            return None;
        }
        // NSFontWeightRegular, NSImageSymbolScaleLarge (3).
        let cfg: *mut AnyObject = msg_send![class!(NSImageSymbolConfiguration), configurationWithPointSize: pt, weight: 0.0f64, scale: 3isize];
        let img: *mut AnyObject = if cfg.is_null() { img } else { msg_send![img, imageWithSymbolConfiguration: cfg] };
        if img.is_null() {
            return None;
        }
        let size: NSSize = msg_send![img, size];
        if size.width <= 0.0 || size.height <= 0.0 {
            return None;
        }
        let w = (size.width * ppp).ceil().max(1.0) as isize;
        let h = (size.height * ppp).ceil().max(1.0) as isize;
        let space = NSString::from_str("NSDeviceRGBColorSpace");
        let rep: *mut AnyObject = msg_send![class!(NSBitmapImageRep), alloc];
        let rep: *mut AnyObject = msg_send![rep, initWithBitmapDataPlanes: std::ptr::null_mut::<*mut u8>(), pixelsWide: w, pixelsHigh: h, bitsPerSample: 8isize, samplesPerPixel: 4isize, hasAlpha: true, isPlanar: false, colorSpaceName: &*space, bytesPerRow: w * 4, bitsPerPixel: 32isize];
        if rep.is_null() {
            return None;
        }
        let gctx: *mut AnyObject = msg_send![class!(NSGraphicsContext), graphicsContextWithBitmapImageRep: rep];
        let _: () = msg_send![class!(NSGraphicsContext), saveGraphicsState];
        let _: () = msg_send![class!(NSGraphicsContext), setCurrentContext: gctx];
        let _: () = msg_send![img, drawInRect: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w as f64, h as f64))];
        let _: () = msg_send![class!(NSGraphicsContext), restoreGraphicsState];
        let data: *const u8 = msg_send![rep, bitmapData];
        if data.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(data, (w * h * 4) as usize);
        let pixels = bytes.chunks_exact(4).map(|p| Color32::from_white_alpha(p[3])).collect();
        Some((egui::ColorImage::new([w as usize, h as usize], pixels), Vec2::new(size.width as f32, size.height as f32)))
    };
    objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
}

#[cfg(not(target_os = "macos"))]
fn symbol_text_image(_name: &str, _pt: f64, _ppp: f64) -> Option<(egui::ColorImage, Vec2)> {
    None
}

/// Brings a dialog (by its key) in front of every other app's window, as an alert about the app in
/// front: it floats above other windows while open and takes the keyboard, without first bringing
/// FontCraft's main window up. Nothing in test pictures (the dialog is drawn inside the window).
pub fn dialog_to_front(ctx: &egui::Context, key: &str) {
    if ctx.embed_viewports() {
        return;
    }
    let vid = egui::ViewportId::from_hash_of(("dialog", key));
    ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
    ctx.send_viewport_cmd_to(vid, egui::ViewportCommand::Focus);
    ctx.request_repaint_of(vid);
}
