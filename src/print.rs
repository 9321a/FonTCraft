//! Printing the font catalog: a window with what to print and how, then FontCraft lays out the
//! pages (on the default printer's paper), saves them as a PDF and opens the system's own print
//! dialog for it (macOS: the standard print panel; Windows: the default PDF app's print; Linux:
//! the default PDF viewer, which has Print).

use crate::i18n::{self, K};
use crate::{fonts, oslook, App, View};
use eframe::egui;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct PrintDlg {
    /// 0 = all fonts, 1 = the list shown now, 2 = favourites.
    which: usize,
    /// 0 = a sample line per font, 1 = a specimen per family, 2 = names only (compact).
    layout: usize,
    size: f32,
    text: String,
    styles: bool,
    names: bool,
    columns: usize,
    paper: Paper,
    /// The paper sizes the chosen printer takes.
    sizes: Vec<Paper>,
    /// The installed printers and the one chosen (the default one first).
    printers: Vec<Printer>,
    printer: Option<usize>,
    /// The printers, or one printer's papers, being read in the background.
    loading: Option<Arc<Mutex<Option<Loaded>>>>,
    job: Option<Arc<Mutex<Option<Result<Done, String>>>>>,
    error: Option<String>,
}

impl PrintDlg {
    /// Whether the printers (or a printer's papers) are still being read.
    pub fn loading(&self) -> bool {
        self.loading.is_some()
    }
}

/// The default printer's paper, in points, and the printer's name.
#[derive(Clone)]
pub struct Paper {
    pub w: f32,
    pub h: f32,
    pub name: String,
    /// The printer's name (macOS matches it to the paper chosen in System Settings).
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub printer: String,
}

/// An installed printer: the system's id for it (a CUPS queue on macOS and Linux, the printer's
/// name on Windows) and the name people see.
#[derive(Clone, Debug, PartialEq)]
pub struct Printer {
    pub id: String,
    pub label: String,
}

/// The paper sizes a printer takes and the one it uses unless told otherwise.
#[derive(Clone)]
pub struct Papers {
    pub sizes: Vec<Paper>,
    pub default: usize,
}

/// The usual ISO and US sizes, for when no printer is installed or it can't be asked.
fn usual_sizes(printer: &str) -> Papers {
    let sizes: Vec<Paper> = [("A4", 595.0, 842.0), ("A3", 842.0, 1191.0), ("A5", 420.0, 595.0), ("US Letter", 612.0, 792.0), ("US Legal", 612.0, 1008.0), ("Tabloid", 792.0, 1224.0), ("Executive", 522.0, 756.0)]
        .into_iter()
        .map(|(name, w, h)| Paper { w, h, name: name.into(), printer: printer.to_string() })
        .collect();
    // US and Canadian English, and the Americas that use Letter, start on US Letter.
    let loc = std::env::var("LC_PAPER").or_else(|_| std::env::var("LANG")).unwrap_or_default();
    let letter = ["_US", "_CA", "_MX", "_CO", "_VE", "_CL", "_PH"].iter().any(|r| loc.contains(r));
    Papers { default: if letter { 3 } else { 0 }, sizes }
}

/// Sizes printers name by their id (PPD names), in points.
#[cfg_attr(any(windows, target_os = "android"), allow(dead_code))]
fn known_size(id: &str) -> Option<(f32, f32)> {
    Some(match id {
        "A3" => (842.0, 1191.0),
        "A4" => (595.0, 842.0),
        "A5" => (420.0, 595.0),
        "A6" => (297.0, 420.0),
        "B5" => (516.0, 729.0),
        "Letter" => (612.0, 792.0),
        "Legal" => (612.0, 1008.0),
        "Tabloid" | "11x17" => (792.0, 1224.0),
        "Executive" => (522.0, 756.0),
        "Statement" => (396.0, 612.0),
        "FanFoldGermanLegal" | "Folio" => (612.0, 936.0),
        "Env10" => (297.0, 684.0),
        "EnvDL" => (312.0, 624.0),
        "EnvC5" => (459.0, 649.0),
        "EnvMonarch" => (279.0, 540.0),
        _ => {
            // PWG names carry their size ("iso_a4_210x297mm", "na_letter_8.5x11in"), as do
            // plain ones like "4x6" (inches).
            let last = id.rsplit('_').next().unwrap_or(id);
            let (dims, k) = if let Some(d) = last.strip_suffix("mm") { (d, 72.0 / 25.4) } else { (last.strip_suffix("in").unwrap_or(last), 72.0) };
            let (w, h) = dims.split_once('x')?;
            let (w, h): (f32, f32) = (w.parse().ok()?, h.parse().ok()?);
            ((w * k).round(), (h * k).round())
        }
    })
}

/// A paper's id as print dialogs show it, when the printer's description gives no name.
#[cfg_attr(any(windows, target_os = "android"), allow(dead_code))]
fn readable_name(id: &str) -> String {
    match id {
        "Letter" => "US Letter".into(),
        "Legal" => "US Legal".into(),
        "Env10" => "Envelope #10".into(),
        "EnvDL" => "Envelope DL".into(),
        "EnvC5" => "Envelope C5".into(),
        "EnvMonarch" => "Envelope Monarch".into(),
        "FanFoldGermanLegal" => "German Legal Fanfold".into(),
        "FolioSP" => "Folio".into(),
        _ => match id.split_once('x') {
            Some((w, h)) if w.parse::<f32>().is_ok() && h.parse::<f32>().is_ok() => format!("{w} × {h} in"),
            _ => id.replace('_', " "),
        },
    }
}

/// The installed printers and which one is the default, as the system lists them.
pub fn installed_printers() -> (Vec<Printer>, Option<usize>) {
    #[cfg(windows)]
    {
        let out = powershell(
            "Add-Type -AssemblyName System.Drawing; $d = (New-Object System.Drawing.Printing.PrinterSettings).PrinterName; \
             foreach ($n in [System.Drawing.Printing.PrinterSettings]::InstalledPrinters) { \"P`t$n`t$([int]($n -eq $d))\" }",
        );
        let mut list = Vec::new();
        let mut def = None;
        for line in out.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if let ["P", name, d] = f[..] {
                if d.trim() == "1" {
                    def = Some(list.len());
                }
                list.push(Printer { id: name.to_string(), label: name.to_string() });
            }
        }
        (list, def)
    }
    #[cfg(all(unix, not(target_os = "android")))]
    {
        let run = |args: &[&str]| crate::paths::command("lpstat").args(args).env("LC_ALL", "C").output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let mut ids: Vec<String> = run(&["-e"]).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        if ids.is_empty() {
            ids = run(&["-a"]).lines().filter_map(|l| l.split_whitespace().next()).map(String::from).collect();
        }
        // "system default destination: X" (in the system's language): the name follows the last colon.
        let default_id = run(&["-d"]).lines().next().and_then(|l| l.rsplit(':').next()).map(|s| s.trim().to_string()).unwrap_or_default();
        let list: Vec<Printer> = ids.iter().map(|id| Printer { id: id.clone(), label: cups_label(id) }).collect();
        let def = list.iter().position(|p| p.id == default_id);
        (list, def)
    }
    #[cfg(target_os = "android")]
    {
        (Vec::new(), None)
    }
}

/// The name people see for a CUPS queue: its printer-info, else the queue with spaces.
#[cfg(all(unix, not(target_os = "android")))]
fn cups_label(id: &str) -> String {
    let opts = crate::paths::command("lpoptions").args(["-p", id]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    if let Some(rest) = opts.split("printer-info=").nth(1) {
        let v: String = if let Some(q) = rest.strip_prefix('\'') {
            q.split('\'').next().unwrap_or("").to_string()
        } else {
            // Unquoted, with spaces written as "\ ".
            let mut s = String::new();
            let mut esc = false;
            for c in rest.chars() {
                if esc {
                    s.push(c);
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c.is_whitespace() {
                    break;
                } else {
                    s.push(c);
                }
            }
            s
        };
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    id.replace('_', " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The paper sizes a printer takes and its default paper, from the printer itself: on macOS and
/// Linux its CUPS options (`lpoptions -p <queue> -l`, PageSize, the default marked with `*`) with
/// their names and sizes from its PPD; on Windows its driver (System.Drawing PrinterSettings:
/// PaperSizes and DefaultPageSettings). Without a printer, the usual ISO and US sizes.
pub fn papers_of(printer: Option<&Printer>) -> Papers {
    let Some(p) = printer else { return usual_sizes("") };
    #[cfg(windows)]
    {
        let script = format!(
            "Add-Type -AssemblyName System.Drawing; $s = New-Object System.Drawing.Printing.PrinterSettings; $s.PrinterName = '{}'; \
             $d = $s.DefaultPageSettings.PaperSize; \
             foreach ($p in $s.PaperSizes) {{ \"S`t$($p.PaperName)`t$($p.Width)`t$($p.Height)`t$([int]($p.RawKind -eq $d.RawKind -and $p.PaperName -eq $d.PaperName))\" }}",
            p.id.replace('\'', "''")
        );
        let mut sizes = Vec::new();
        let mut default = None;
        for line in powershell(&script).lines() {
            let f: Vec<&str> = line.split('\t').collect();
            if let ["S", name, w, h, d] = f[..] {
                // Hundredths of an inch.
                let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) else { continue };
                if w <= 0.0 || h <= 0.0 {
                    continue;
                }
                if d.trim() == "1" && default.is_none() {
                    default = Some(sizes.len());
                }
                sizes.push(Paper { w: (w * 0.72).round(), h: (h * 0.72).round(), name: name.trim().to_string(), printer: p.label.clone() });
            }
        }
        if sizes.is_empty() {
            return usual_sizes(&p.label);
        }
        Papers { default: default.unwrap_or(0), sizes }
    }
    #[cfg(all(unix, not(target_os = "android")))]
    {
        // The PPD's names and sizes (readable on macOS; Linux may keep it to the lp group).
        let ppd = std::fs::read(format!("/etc/cups/ppd/{}.ppd", p.id)).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default();
        let mut dims: Vec<(String, String, f32, f32)> = Vec::new();
        for line in ppd.lines().filter_map(|l| l.strip_prefix("*PaperDimension ")) {
            let Some((key, val)) = line.split_once(':') else { continue };
            let (id, label) = key.split_once('/').unwrap_or((key, key));
            let nums: Vec<f32> = val.trim().trim_matches('"').split_whitespace().filter_map(|n| n.parse().ok()).collect();
            if let [w, h] = nums[..] {
                dims.push((id.trim().to_string(), label.trim().to_string(), w.round(), h.round()));
            }
        }
        let ppd_default = ppd.lines().find_map(|l| l.strip_prefix("*DefaultPageSize:")).map(|s| s.trim().to_string());
        // The printer's own list and default, as CUPS answers for it.
        let opts = crate::paths::command("lpoptions").args(["-p", &p.id, "-l"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let line = opts.lines().find(|l| l.starts_with("PageSize/") || l.starts_with("PageSize:")).or_else(|| opts.lines().find(|l| l.starts_with("media/")));
        let mut choices: Vec<String> = Vec::new();
        let mut marked: Option<String> = None;
        if let Some(l) = line.and_then(|l| l.split_once(':')).map(|(_, v)| v) {
            for c in l.split_whitespace() {
                let c = match c.strip_prefix('*') {
                    Some(c) => {
                        marked = Some(c.to_string());
                        c
                    }
                    None => c,
                };
                choices.push(c.to_string());
            }
        }
        if choices.is_empty() {
            choices = dims.iter().map(|d| d.0.clone()).collect();
        }
        // "A4.Fullbleed", "Letter.Borderless": a variant of another paper ("3.5x5" isn't).
        let variant = |id: &str| id.rsplit_once('.').filter(|(_, s)| s.chars().next().is_some_and(|c| c.is_alphabetic())).map(|(b, _)| b.to_string());
        let base = |id: &str| variant(id).unwrap_or_else(|| id.to_string());
        let mut sizes: Vec<Paper> = Vec::new();
        let mut ids: Vec<String> = Vec::new();
        for c in &choices {
            // Borderless variants and custom sizes are the same paper, or none at all.
            if c.starts_with("Custom") || variant(c).is_some() {
                continue;
            }
            let from_ppd = dims.iter().find(|d| &d.0 == c);
            let Some((w, h)) = from_ppd.map(|d| (d.2, d.3)).or_else(|| known_size(c)) else { continue };
            let name = from_ppd.map(|d| d.1.clone()).filter(|l| !l.is_empty() && l != c).unwrap_or_else(|| readable_name(c));
            sizes.push(Paper { w, h, name, printer: p.label.clone() });
            ids.push(c.clone());
        }
        if sizes.is_empty() {
            return usual_sizes(&p.label);
        }
        let want = marked.or(ppd_default).map(|m| base(&m));
        let default = want.and_then(|m| ids.iter().position(|i| *i == m)).unwrap_or(0);
        Papers { sizes, default }
    }
    #[cfg(target_os = "android")]
    {
        usual_sizes(&p.label)
    }
}

/// Runs a PowerShell script (no profile, no window) and returns what it printed, as UTF-8.
#[cfg(windows)]
fn powershell(script: &str) -> String {
    let full = format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; {script}");
    crate::paths::command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &full])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// The default printer, the printers installed, and the papers the default one takes, with its
/// default paper chosen. On macOS the paper chosen in System Settings → Printers wins when the
/// printer takes it, as the system's own print panel does.
pub fn load_printers() -> Loaded {
    let (list, def) = installed_printers();
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut papers = papers_of(def.and_then(|i| list.get(i)));
    #[cfg(target_os = "macos")]
    if let (Some(sys), Some(i)) = (mac::paper(), def) {
        if sys.printer == list[i].label {
            if let Some(k) = papers.sizes.iter().position(|s| (s.w - sys.w).abs() < 2.0 && (s.h - sys.h).abs() < 2.0) {
                papers.default = k;
            }
        }
    }
    Loaded { printers: Some((list, def)), printer: def, papers }
}

/// What a background look at the printers found.
pub struct Loaded {
    printers: Option<(Vec<Printer>, Option<usize>)>,
    printer: Option<usize>,
    papers: Papers,
}

impl Paper {
    /// The default printer's default paper (or A4 / US Letter without a printer).
    pub fn default_printer() -> Paper {
        let l = load_printers();
        l.papers.sizes[l.papers.default].clone()
    }
}

/// One font to print: family, style, its file's bytes and face index.
#[derive(Clone)]
struct Entry {
    family: String,
    style: String,
    data: Arc<memmap2::Mmap>,
    index: u32,
    styles: usize,
}

/// A grey page being drawn (2 pixels per point).
struct Page {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

const SCALE: f32 = 2.0;
const MARGIN: f32 = 40.0;

impl Page {
    fn new(paper: &Paper) -> Page {
        let (w, h) = ((paper.w * SCALE) as usize, (paper.h * SCALE) as usize);
        Page { w, h, px: vec![255; w * h] }
    }
    /// Draws a rendered picture with its top-left at (x, y) points.
    fn blit(&mut self, r: &fonts::Rendered, x: f32, y: f32, grey: u8) {
        let (ox, oy) = ((x * SCALE) as i64, (y * SCALE) as i64);
        for row in 0..r.h {
            let py = oy + row as i64;
            if py < 0 || py >= self.h as i64 {
                continue;
            }
            for col in 0..r.w {
                let px_ = ox + col as i64;
                if px_ < 0 || px_ >= self.w as i64 {
                    continue;
                }
                let a = r.alpha[row * r.w + col] as u32;
                if a == 0 {
                    continue;
                }
                let i = py as usize * self.w + px_ as usize;
                let under = self.px[i] as u32;
                self.px[i] = ((under * (255 - a) + grey as u32 * a) / 255) as u8;
            }
        }
    }
    fn hline(&mut self, y: f32, x0: f32, x1: f32) {
        let py = (y * SCALE) as usize;
        if py < self.h {
            for x in (x0 * SCALE) as usize..((x1 * SCALE) as usize).min(self.w) {
                self.px[py * self.w + x] = 205;
            }
        }
    }
}

/// Text in the system's interface font (names, headings, page numbers).
fn ui_text(text: &str, pt: f32, max_w: f32) -> Option<fonts::Rendered> {
    let ui = fonts::system_font()?;
    fonts::render(ui, 0, text, pt * SCALE, (max_w * SCALE) as usize, &[], false)
}

fn sample(e: &Entry, text: &str, pt: f32, max_w: f32, wrap: bool) -> Option<fonts::Rendered> {
    fonts::render(&e.data, e.index, text, pt * SCALE, (max_w * SCALE) as usize, &[], wrap)
}

/// Lays out every page.
fn lay_out(entries: &[Entry], dlg: &PrintSettings) -> Vec<Page> {
    let paper = &dlg.paper;
    let width = paper.w - 2.0 * MARGIN;
    let bottom = paper.h - MARGIN - 18.0;
    let mut pages = vec![Page::new(paper)];
    let mut y = MARGIN;
    let new_page = |pages: &mut Vec<Page>, y: &mut f32| {
        pages.push(Page::new(paper));
        *y = MARGIN;
    };
    match dlg.layout {
        // Names only, in columns: each family's name set in its own font.
        2 => {
            let cols = dlg.columns.clamp(1, 4);
            let col_w = width / cols as f32;
            let line = 22.0;
            let per_col = ((bottom - MARGIN) / line).floor().max(1.0) as usize;
            for (i, e) in entries.iter().enumerate() {
                let slot = i % (per_col * cols);
                if i > 0 && slot == 0 {
                    new_page(&mut pages, &mut y);
                }
                let (c, r) = (slot / per_col, slot % per_col);
                let x = MARGIN + c as f32 * col_w;
                let yy = MARGIN + r as f32 * line;
                let page = pages.last_mut().unwrap();
                if let Some(img) = sample(e, &e.family, 13.0, col_w - 8.0, false) {
                    page.blit(&img, x, yy, 0);
                }
            }
        }
        // A specimen per family: name, alphabet, figures and the sample text.
        1 => {
            for e in entries {
                let blocks = [
                    ("ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(), dlg.size),
                    ("abcdefghijklmnopqrstuvwxyz".to_string(), dlg.size),
                    ("0123456789 !?&@#%".to_string(), dlg.size),
                    (dlg.text.clone(), (dlg.size * 0.6).max(10.0)),
                ];
                let mut imgs: Vec<fonts::Rendered> = Vec::new();
                for (t, pt) in &blocks {
                    if let Some(r) = sample(e, t, *pt, width, true) {
                        imgs.push(r);
                    }
                }
                let head = ui_text(&format!("{} — {}", e.family, e.style), 13.0, width);
                let need = 26.0 + imgs.iter().map(|r| r.h as f32 / SCALE + 6.0).sum::<f32>() + 14.0;
                if y + need > bottom && y > MARGIN + 1.0 {
                    new_page(&mut pages, &mut y);
                }
                let page = pages.last_mut().unwrap();
                if let Some(h) = &head {
                    page.blit(h, MARGIN, y, 60);
                }
                y += 24.0;
                for r in &imgs {
                    page.blit(r, MARGIN, y, 0);
                    y += r.h as f32 / SCALE + 6.0;
                }
                page.hline(y + 4.0, MARGIN, MARGIN + width);
                y += 14.0;
            }
        }
        // One sample line per font, with its name above.
        _ => {
            for e in entries {
                let img = sample(e, &dlg.text, dlg.size, width, false);
                let label = if dlg.names {
                    let s = if e.styles == 1 { String::new() } else { format!("  ·  {}", e.styles) };
                    format!("{} {}{}", e.family, e.style, s)
                } else {
                    e.family.clone()
                };
                let head = ui_text(&label, 8.5, width);
                let h = img.as_ref().map(|r| r.h as f32 / SCALE).unwrap_or(dlg.size);
                let need = 14.0 + h + 10.0;
                if y + need > bottom && y > MARGIN + 1.0 {
                    new_page(&mut pages, &mut y);
                }
                let page = pages.last_mut().unwrap();
                if let Some(hd) = &head {
                    page.blit(hd, MARGIN, y, 110);
                }
                y += 14.0;
                if let Some(r) = &img {
                    page.blit(r, MARGIN, y, 0);
                }
                y += h + 6.0;
                page.hline(y, MARGIN, MARGIN + width);
                y += 4.0;
            }
        }
    }
    // Page numbers.
    let n = pages.len();
    for (i, p) in pages.iter_mut().enumerate() {
        if let Some(r) = ui_text(&format!("FontCraft  ·  {} / {}", i + 1, n), 7.5, width) {
            p.blit(&r, MARGIN, paper.h - MARGIN, 120);
        }
    }
    pages
}

/// Writes the pages as a PDF (one grey picture per page, compressed).
fn write_pdf(pages: &[Page], paper: &Paper, path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let n = pages.len();
    // Objects: 1 catalog, 2 pages, then per page: page, content, image.
    let obj = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, body: &[u8]| {
        offsets.push(out.len());
        let id = offsets.len();
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    obj(&mut out, &mut offsets, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: String = (0..n).map(|i| format!("{} 0 R ", 3 + i * 3)).collect();
    obj(&mut out, &mut offsets, format!("<< /Type /Pages /Kids [{kids}] /Count {n} >>").as_bytes());
    for (i, p) in pages.iter().enumerate() {
        let (page_id, content_id, image_id) = (3 + i * 3, 4 + i * 3, 5 + i * 3);
        let _ = page_id;
        obj(
            &mut out,
            &mut offsets,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /XObject << /Im0 {image_id} 0 R >> >> /Contents {content_id} 0 R >>",
                paper.w, paper.h
            )
            .as_bytes(),
        );
        let content = format!("q {} 0 0 {} 0 0 cm /Im0 Do Q", paper.w, paper.h);
        obj(&mut out, &mut offsets, format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()).as_bytes());
        let packed = miniz_oxide::deflate::compress_to_vec_zlib(&p.px, 6);
        let mut body = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
            p.w,
            p.h,
            packed.len()
        )
        .into_bytes();
        body.extend_from_slice(&packed);
        body.extend_from_slice(b"\nendstream");
        obj(&mut out, &mut offsets, &body);
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for o in &offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1).as_bytes());
    std::fs::File::create(path)?.write_all(&out)
}

#[derive(Clone)]
struct PrintSettings {
    layout: usize,
    size: f32,
    text: String,
    names: bool,
    columns: usize,
    paper: Paper,
}

/// A laid-out catalog: its PDF and its pages.
struct Done {
    path: PathBuf,
    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios"), allow(dead_code))]
    pages: Vec<Page>,
    #[cfg_attr(any(target_os = "macos", target_os = "android", target_os = "ios"), allow(dead_code))]
    paper: Paper,
}

/// Opens the system's own print dialog: macOS's print panel for the PDF; on Windows and Linux the
/// system's Print dialog (comdlg32, GTK) printing the pages themselves (printsys.rs); where that
/// can't open, the PDF in the default viewer.
fn print_pdf(done: Done) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        mac::print(&done.path)
    }
    #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_os = "android"))))]
    {
        let sheets: Vec<crate::printsys::Sheet> = done.pages.into_iter().map(|p| crate::printsys::Sheet { w: p.w, h: p.h, px: p.px }).collect();
        match crate::printsys::print(sheets, done.paper.w, done.paper.h, "FontCraft") {
            Ok(()) => Ok(()),
            Err(_) => open_with_viewer(&done.path),
        }
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        open_with_viewer(&done.path)
    }
}

/// The PDF in the default viewer (which has Print).
#[cfg(not(target_os = "macos"))]
fn open_with_viewer(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::paths::command("cmd").args(["/C", "start", ""]).arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        crate::paths::command("xdg-open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::Paper;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSSize, NSString};

    #[link(name = "PDFKit", kind = "framework")]
    extern "C" {}

    fn string_of(s: *mut AnyObject) -> String {
        if s.is_null() {
            return String::new();
        }
        unsafe {
            let c: *const std::os::raw::c_char = msg_send![s, UTF8String];
            if c.is_null() { String::new() } else { std::ffi::CStr::from_ptr(c).to_string_lossy().to_string() }
        }
    }

    pub fn paper() -> Option<Paper> {
        let run = || unsafe {
            let info: *mut AnyObject = msg_send![class!(NSPrintInfo), sharedPrintInfo];
            if info.is_null() {
                return None;
            }
            let size: NSSize = msg_send![info, paperSize];
            let name: *mut AnyObject = msg_send![info, localizedPaperName];
            let printer: *mut AnyObject = msg_send![info, printer];
            let pname: *mut AnyObject = if printer.is_null() { std::ptr::null_mut() } else { msg_send![printer, name] };
            Some(Paper { w: size.width as f32, h: size.height as f32, name: string_of(name), printer: string_of(pname) })
        };
        objc2::exception::catch(std::panic::AssertUnwindSafe(run)).ok().flatten()
    }

    /// The standard macOS print panel for the PDF (default printer and its options).
    pub fn print(path: &std::path::Path) -> Result<(), String> {
        let p = path.to_string_lossy().to_string();
        let run = move || unsafe {
            let url: Retained<AnyObject> = msg_send![class!(NSURL), fileURLWithPath: &*NSString::from_str(&p)];
            let doc: Option<Retained<AnyObject>> = msg_send![msg_send![class!(PDFDocument), alloc], initWithURL: &*url];
            let Some(doc) = doc else { return false };
            let info: *mut AnyObject = msg_send![class!(NSPrintInfo), sharedPrintInfo];
            let op: *mut AnyObject = msg_send![&doc, printOperationForPrintInfo: info, scalingMode: 1isize, autoRotate: true];
            if op.is_null() {
                return false;
            }
            let _: () = msg_send![op, setShowsPrintPanel: true];
            let _: () = msg_send![op, setShowsProgressPanel: true];
            let _: bool = msg_send![op, runOperation];
            true
        };
        match objc2::exception::catch(std::panic::AssertUnwindSafe(run)) {
            Ok(true) => Ok(()),
            Ok(false) => Err("PDF".into()),
            Err(e) => Err(e.map(|e| format!("{e:?}")).unwrap_or_else(|| "?".into())),
        }
    }
}

impl App {
    fn print_entries(&self, which: usize, styles: bool) -> Vec<Entry> {
        let fams: Vec<usize> = match which {
            1 => (0..self.lib.families.len()).filter(|&i| self.family_matches(i, self.view)).collect(),
            2 => (0..self.lib.families.len()).filter(|&i| self.family_matches(i, View::Favorites)).collect(),
            _ => (0..self.lib.families.len()).filter(|&i| self.family_matches(i, View::All)).collect(),
        };
        let mut out = Vec::new();
        for fi in fams {
            let fam = &self.lib.families[fi];
            let faces: Vec<usize> = if styles { fam.faces.clone() } else { vec![fam.default_face] };
            for f in faces {
                let face = &self.lib.faces[f];
                out.push(Entry {
                    family: fam.name.clone(),
                    style: face.style.clone(),
                    data: self.lib.files[face.file].data.clone(),
                    index: face.index,
                    styles: fam.faces.len(),
                });
            }
        }
        out
    }

    /// The printer button's window.
    pub(crate) fn print_window(&mut self, ctx: &egui::Context) {
        let Some(mut dlg) = self.print.take() else { return };
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        // A finished job: open the system's print dialog.
        if let Some(job) = &dlg.job {
            let done = job.lock().unwrap().take();
            match done {
                Some(Ok(done)) => {
                    dlg.job = None;
                    match print_pdf(done) {
                        Ok(()) => return,
                        Err(e) => dlg.error = Some(t(K::PrintFailed).replace("{err}", &e)),
                    }
                }
                Some(Err(e)) => {
                    dlg.job = None;
                    dlg.error = Some(t(K::PrintFailed).replace("{err}", &e));
                }
                None => ctx.request_repaint_after(std::time::Duration::from_millis(150)),
            }
        }
        // The printers, or a printer's papers, read in the background.
        if let Some(slot) = &dlg.loading {
            let got = slot.lock().unwrap().take();
            match got {
                Some(l) => {
                    dlg.loading = None;
                    // The papers of the printer chosen now (not of one chosen before it).
                    let fresh = l.printers.is_some() || l.printer == dlg.printer;
                    if let Some((list, def)) = l.printers {
                        dlg.printers = list;
                        dlg.printer = def.or(if dlg.printers.is_empty() { None } else { Some(0) });
                    }
                    if fresh && !l.papers.sizes.is_empty() {
                        dlg.paper = l.papers.sizes[l.papers.default.min(l.papers.sizes.len() - 1)].clone();
                        dlg.sizes = l.papers.sizes;
                    }
                }
                None => ctx.request_repaint_after(std::time::Duration::from_millis(150)),
            }
        }
        let count = self.print_entries(dlg.which, dlg.styles).len();
        let mut go = false;
        let mut cancel = false;
        let (_, closed) = oslook::dialog(ctx, "print", t(K::PrintTitle), 560.0, pal.panel, None, |ui| {
            let mut pick: Option<usize> = None;
            egui::Grid::new("print-grid").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
                ui.label(egui::RichText::new(t(K::PrintWhich)).strong().color(pal.ink));
                ui.vertical(|ui| {
                    oslook::radio(ui, &mut dlg.which, 0, t(K::PrintAll));
                    oslook::radio(ui, &mut dlg.which, 1, t(K::PrintShown));
                    oslook::radio(ui, &mut dlg.which, 2, t(K::PrintFavs));
                    oslook::check(ui, &mut dlg.styles, t(K::PrintStyles));
                    ui.label(egui::RichText::new(t(K::LibFamilies).replace("{n}", &count.to_string())).size(12.0).color(pal.muted));
                });
                ui.end_row();
                ui.label(egui::RichText::new(t(K::PrintLayout)).strong().color(pal.ink));
                ui.vertical(|ui| {
                    oslook::radio(ui, &mut dlg.layout, 0, t(K::LayoutLines));
                    oslook::radio(ui, &mut dlg.layout, 1, t(K::LayoutSpecimen));
                    oslook::radio(ui, &mut dlg.layout, 2, t(K::LayoutNames));
                    if dlg.layout == 0 {
                        oslook::check(ui, &mut dlg.names, t(K::PrintNames));
                    }
                });
                ui.end_row();
                if dlg.layout == 2 {
                    ui.label(egui::RichText::new(t(K::PrintColumns)).strong().color(pal.ink));
                    ui.add(egui::Slider::new(&mut dlg.columns, 1..=4));
                    ui.end_row();
                } else {
                    ui.label(egui::RichText::new(t(K::PrintSize)).strong().color(pal.ink));
                    ui.add(egui::Slider::new(&mut dlg.size, 9.0..=72.0).suffix(" pt").integer());
                    ui.end_row();
                    ui.label(egui::RichText::new(t(K::PrintText)).strong().color(pal.ink));
                    oslook::field(ui, egui::TextEdit::singleline(&mut dlg.text).desired_width(320.0).margin(egui::Margin::symmetric(8, 5)));
                    ui.end_row();
                }
                // The printer (the system's default one first) and the paper it takes, from its
                // own list, with its default paper chosen; the page count follows, below.
                ui.label(egui::RichText::new(t(K::PrinterLabel)).strong().color(pal.ink));
                ui.horizontal(|ui| {
                    let current = dlg.printer.and_then(|i| dlg.printers.get(i)).map(|p| p.label.clone()).unwrap_or_else(|| "—".into());
                    ui.add_enabled_ui(!dlg.printers.is_empty(), |ui| {
                        oslook::combo("print-printer", current).width(320.0).show_ui(ui, |ui| {
                            for (i, p) in dlg.printers.iter().enumerate() {
                                if ui.selectable_label(dlg.printer == Some(i), &p.label).clicked() && dlg.printer != Some(i) {
                                    pick = Some(i);
                                }
                            }
                        });
                    });
                    if dlg.loading.is_some() {
                        ui.spinner();
                    }
                });
                ui.end_row();
                ui.label(egui::RichText::new(t(K::PaperLabel)).strong().color(pal.ink));
                let mm = |p: &Paper| format!("{}  ({} × {} mm)", p.name, (p.w * 25.4 / 72.0).round(), (p.h * 25.4 / 72.0).round());
                let sizes = dlg.sizes.clone();
                oslook::combo("print-paper", mm(&dlg.paper)).width(320.0).show_ui(ui, |ui| {
                    for p in &sizes {
                        if ui.selectable_label(p.name == dlg.paper.name, mm(p)).clicked() {
                            dlg.paper = p.clone();
                        }
                    }
                });
                ui.end_row();
            });
            if let Some(i) = pick {
                dlg.printer = Some(i);
                let printer = dlg.printers[i].clone();
                let slot = Arc::new(Mutex::new(None));
                let out = slot.clone();
                let ctx2 = ui.ctx().clone();
                std::thread::spawn(move || {
                    let papers = papers_of(Some(&printer));
                    *out.lock().unwrap() = Some(Loaded { printers: None, printer: Some(i), papers });
                    ctx2.request_repaint();
                });
                dlg.loading = Some(slot);
            }
            // A rough page count, so a big catalog doesn't come as a surprise.
            let per_page = match dlg.layout {
                2 => ((dlg.paper.h - 2.0 * MARGIN - 18.0) / 22.0).floor().max(1.0) * dlg.columns as f32,
                1 => ((dlg.paper.h - 2.0 * MARGIN) / (dlg.size * 4.6 + 50.0)).floor().max(1.0),
                _ => ((dlg.paper.h - 2.0 * MARGIN - 18.0) / (dlg.size * 1.35 + 24.0)).floor().max(1.0),
            };
            let pages = (count as f32 / per_page).ceil().max(1.0) as usize;
            ui.label(egui::RichText::new(t(K::PrintPages).replace("{n}", &pages.to_string())).size(12.5).color(pal.muted));
            if let Some(e) = &dlg.error {
                ui.label(egui::RichText::new(e).color(pal.warn));
            }
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), oslook::metrics().control_h + 4.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let busy = dlg.job.is_some();
                if oslook::default_button(ui, !busy && count > 0, t(K::PrintBtn)).clicked() {
                    go = true;
                }
                if ui.button(t(K::Cancel)).clicked() {
                    cancel = true;
                }
                if busy {
                    ui.spinner();
                    ui.label(egui::RichText::new(t(K::Preparing)).size(12.5).color(pal.muted));
                }
            });
        });
        if go {
            let entries = self.print_entries(dlg.which, dlg.styles);
            let settings = PrintSettings { layout: dlg.layout, size: dlg.size, text: dlg.text.clone(), names: dlg.names, columns: dlg.columns, paper: dlg.paper.clone() };
            let job = Arc::new(Mutex::new(None));
            let out = job.clone();
            std::thread::spawn(move || {
                let pages = lay_out(&entries, &settings);
                let path = std::env::temp_dir().join("FontCraft Catalog.pdf");
                let paper = settings.paper.clone();
                let r = write_pdf(&pages, &settings.paper, &path).map(|_| Done { path, pages, paper }).map_err(|e| e.to_string());
                *out.lock().unwrap() = Some(r);
            });
            dlg.job = Some(job);
            dlg.error = None;
        }
        if !closed && !cancel {
            self.print = Some(dlg);
        }
    }

    pub(crate) fn open_print(&mut self) {
        // A4 or US Letter until the printers answer (in the background: a network printer can
        // take a moment).
        let usual = usual_sizes("");
        let paper = usual.sizes[usual.default].clone();
        let slot = Arc::new(Mutex::new(None));
        let out = slot.clone();
        std::thread::spawn(move || {
            *out.lock().unwrap() = Some(load_printers());
        });
        self.print = Some(PrintDlg {
            which: 1,
            layout: 0,
            size: 24.0,
            text: self.preview.clone(),
            styles: false,
            names: true,
            columns: 3,
            paper,
            sizes: usual.sizes,
            printers: Vec::new(),
            printer: None,
            loading: Some(slot),
            job: None,
            error: None,
        });
    }
}

/// Lays out and saves a catalog PDF without any window (`--print-test <file.pdf>`, for checks).
pub fn test_pdf(app_fonts: &fonts::Library, path: &std::path::Path) -> std::io::Result<usize> {
    let entries: Vec<Entry> = app_fonts
        .families
        .iter()
        .take(40)
        .map(|fam| {
            let f = &app_fonts.faces[fam.default_face];
            Entry { family: fam.name.clone(), style: f.style.clone(), data: app_fonts.files[f.file].data.clone(), index: f.index, styles: fam.faces.len() }
        })
        .collect();
    let settings = PrintSettings { layout: 0, size: 24.0, text: "The quick brown fox jumps over the lazy dog".into(), names: true, columns: 3, paper: Paper::default_printer() };
    let pages = lay_out(&entries, &settings);
    write_pdf(&pages, &settings.paper, path)?;
    Ok(pages.len())
}

#[cfg(test)]
mod tests {
    #[test]
    fn printers_and_papers() {
        let l = super::load_printers();
        let (list, def) = l.printers.clone().unwrap();
        eprintln!("printers: {:?}\ndefault: {:?}", list.iter().map(|p| &p.label).collect::<Vec<_>>(), def.map(|i| &list[i].label));
        eprintln!("papers: {:?}", l.papers.sizes.iter().map(|p| format!("{} {}x{}", p.name, p.w, p.h)).collect::<Vec<_>>());
        eprintln!("default paper: {}", l.papers.sizes[l.papers.default].name);
        assert!(!l.papers.sizes.is_empty());
        assert_eq!(super::known_size("iso_a4_210x297mm"), Some((595.0, 842.0)));
        assert_eq!(super::known_size("4x6"), Some((288.0, 432.0)));
    }
}
