//! Auto Activation: working with design and office apps.
//!
//! - Finds the supported apps installed on this Mac (with their icons).
//! - Watches which of them are open and which one was last in front.
//! - "Use this font": turns the font on, then changes the selected text in that app to it — with
//!   AppleScript where the app offers it (Word, Excel), otherwise by copying the selection, swapping
//!   the font in the copied rich text, and pasting it back.
//! - Inserting a glyph: puts the character, in the chosen font, into the app's document.
//! - Fonts turned on this way stay on until their document closes (apps that report their
//!   documents), the app quits, or FontCraft quits (if that option is on).

// Android has no other apps to script, paste into or watch: the desktop catalog goes unused there.
#![cfg_attr(target_os = "android", allow(dead_code))]

use crate::i18n::{self, K};
use crate::App;
use eframe::egui::{self, Align2, Color32, ColorImage, Margin, Rect, Sense, Stroke, TextureHandle, TextureOptions, Vec2};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum Docs {
    /// Pages, Keynote, Numbers, TextEdit: "documents".
    Standard,
    Word,
    Excel,
    PowerPoint,
}

/// What FontCraft can do with an app.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum How {
    /// Change the selected text's font with the app's own AppleScript (RightFont's scripts).
    Script,
    /// Change the selected text's font by copying it, swapping the font, and pasting it back
    /// (apps that paste rich text with its fonts; RightFont's fallback).
    Paste,
    /// Only turn fonts on for the app (its text tools then list them) and insert characters; the
    /// app doesn't take a font from pasted text and has no script to set it.
    Activate,
}

/// When an app reads the list of installed fonts.
#[derive(Clone, Copy, PartialEq)]
enum FontRead {
    /// It sees fonts as soon as they are turned on.
    Live,
    /// Only when it opens, but it has a menu command (one of these names, in any of its languages)
    /// that reads them again: FontCraft chooses it for the person (macOS).
    Menu(&'static [&'static str]),
    /// Only when it opens, on every system: it must be reopened to see a new font.
    AtStart,
    /// Only when it opens, on macOS (LibreOffice: its Mac version doesn't listen for font changes).
    AtStartOnMac,
}

/// VectorCraft's Type › "Refresh Font List" in each of its languages (its own translation table).
const VECTORCRAFT_REFRESH: &[&str] = &[
    "Refresh Font List", "Actualizar lista de fuentes", "Actualiser la liste des polices", "Schriftliste aktualisieren",
    "Aggiorna elenco font", "Atualizar Lista de Fontes", "Lettertypelijst vernieuwen", "Uppdatera typsnittslistan",
    "Opdatér skrifttypeliste", "Päivitä fonttiluettelo", "Odśwież listę czcionek", "Obnovit seznam písem",
    "Yazı Tipi Listesini Yenile", "Betűtípuslista frissítése", "Ανανέωση λίστας γραμματοσειρών", "Обновить список шрифтов",
    "Оновити список шрифтів", "Обновяване на списъка с шрифтове", "Segarkan daftar font", "Làm mới danh sách phông chữ",
    "刷新字体列表", "重新整理字體清單", "フォントリストを更新", "글꼴 목록 새로 고침", "Actualitza la llista de tipus de lletra",
    "Osvježi listu fontova", "შრიფტების სიის განახლება", "रिफ्रेश फ़ॉन्ट सूची",
];
/// DesignCraft's Type › "Reload Fonts" (command font.reload), in English and French, its two languages.
const DESIGNCRAFT_RELOAD: &[&str] = &["Reload Fonts", "Recharger les polices"];

struct AppDef {
    name: &'static str,
    /// macOS bundle id; also the app's key on every system (settings, running list).
    bundle: &'static str,
    docs: Option<Docs>,
    /// Uses PostScript font names (Cocoa and Adobe apps); others get the family name.
    ps_names: bool,
    /// RightFont's own AppleScript for setting the selected text's font ({font} = the font name).
    /// None: copy the selection, swap its font, paste it back (RightFont's fallback too).
    script: Option<&'static str>,
    /// For apps without a script: whether pasting rich text sets the font (Paste) or not (Activate).
    how: How,
    /// When it reads the installed fonts.
    fonts: FontRead,
    /// Linux: desktop-file names or program names it is found by.
    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
    linux: &'static [&'static str],
    /// Windows: program file names (searched in Program Files and the user's programs).
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    win: &'static [&'static str],
}

/// Defaults for the catalog entries below.
const MAC: AppDef = AppDef { name: "", bundle: "", docs: None, ps_names: true, script: None, how: How::Paste, fonts: FontRead::Live, linux: &[], win: &[] };

impl AppDef {
    fn how(&self) -> How {
        if self.script.is_some() { How::Script } else { self.how }
    }

    /// The menu command that makes it read the fonts again (macOS), if it has one.
    fn reload_menu(&self) -> Option<&'static [&'static str]> {
        match self.fonts {
            FontRead::Menu(names) if cfg!(target_os = "macos") => Some(names),
            _ => None,
        }
    }

    /// Whether it only sees fonts turned on before it opened (and has no command to look again here).
    fn needs_reopen(&self) -> bool {
        match self.fonts {
            FontRead::Live => false,
            FontRead::Menu(_) => !cfg!(target_os = "macos"),
            FontRead::AtStart => true,
            FontRead::AtStartOnMac => cfg!(target_os = "macos"),
        }
    }
}

// RightFont's scripts, from its main program (strings 0x10028c6d5 … 0x10028d4fa, see MECHANICS.md).
const PHOTOSHOP: &str = "tell application id \"com.adobe.Photoshop\"\nset font of text object of current layer of current document to \"{font}\"\nend tell";
const ILLUSTRATOR: &str = "tell application id \"{bundle}\"\nactivate\nset text font of every character of selection of current document to text font \"{font}\"\nend tell";
const INDESIGN: &str = "tell application id \"{bundle}\"\nset applied font of every character of selection of active window to \"{font}\"\nend tell";
const AFTER_EFFECTS: &str = "tell application id \"{bundle}\"\nDoScript \"var item = app.project.activeItem;if (item != null) {var layers = item.selectedLayers;for (var i = 0; i < layers.length; i++) {var layer = layers[i];if (layer.toString() == '[object TextLayer]') {var textProp = layer.property('Source Text');var textDocument = textProp.value;textDocument.font = '{font}';textProp.setValue(textDocument);}}}\"\nend tell";
const QUARK: &str = "tell application \"QuarkXPress\"\nactivate\ntell document 1\ntell story 1 of current box\nset font to \"{font}\"\nend tell\nend tell\nend tell";
const OMNIGRAFFLE: &str = "tell application id \"OGfl\"\nactivate\nset lstSeln to selection of front window\nrepeat with theItem in lstSeln\ntell canvas of front window\nset font of text of theItem to \"{font}\"\nend tell\nend repeat\nend tell";
const WORD_EXCEL: &str = "tell application id \"{bundle}\"\nactivate\nset fontname to font object of selection of active window\nset name of fontname to \"{font}\"\nend tell";
const POWERPOINT: &str = "tell application id \"{bundle}\"\nactivate\nset fontname to font of text range of selection of active window\nset font name of fontname to \"{font}\"\nend tell";
const KEYNOTE: &str = "tell application id \"{bundle}\"\nset fontItems to selection of front document\nrepeat with textItem in fontItems\nset itemType to class of textItem as string\nif (itemType is equal to \"text item\" or itemType is equal to \"shape\") then\ntell textItem\nset the font of its object text to \"{font}\"\nend tell\nend if\nend repeat\nend tell";
const PAGES: &str = "tell application id \"{bundle}\"\nset fontItems to selection of front document\nrepeat with textItem in fontItems\nset itemType to class of textItem as string\nif (itemType is equal to \"document\") then\ntell textItem\nset the font of its body text to \"{font}\"\nend tell\nelse if (itemType is equal to \"shape\") then\ntell textItem\nset the font of its object text to \"{font}\"\nend tell\nend if\nend repeat\nend tell";
const TEXTEDIT: &str = "tell application \"System Events\" to tell process \"TextEdit\"\nset {c1, c2} to value of attribute \"AXSelectedTextRange\" of text area 1 of scroll area 1 of window 1\nend tell\ntell application \"TextEdit\" to tell document 1\ntell characters c1 thru c2\nset font to \"{font}\"\nend tell\nend tell";
const PIXELMATOR: &str = "tell application id \"{bundle}\"\ntell front document\ntell current layer\nset font of text content to \"{font}\"\nend tell\nend tell\nend tell";

/// RightFont's list of apps it works with (strings 0x10028abb5 … 0x10028b05e, same order),
/// plus VectorCraft, LibreOffice and ONLYOFFICE, which the person asked for.
const CATALOG: &[AppDef] = &[
    AppDef { name: "Adobe Photoshop", bundle: "com.adobe.Photoshop", docs: None, ps_names: true, script: Some(PHOTOSHOP), ..MAC },
    AppDef { name: "Adobe Illustrator", bundle: "com.adobe.illustrator", docs: None, ps_names: true, script: Some(ILLUSTRATOR), ..MAC },
    AppDef { name: "Adobe Illustrator (Beta)", bundle: "com.adobe.illustratorBeta", docs: None, ps_names: true, script: Some(ILLUSTRATOR), ..MAC },
    AppDef { name: "Adobe InDesign", bundle: "com.adobe.InDesign", docs: None, ps_names: false, script: Some(INDESIGN), ..MAC },
    AppDef { name: "Adobe InCopy", bundle: "com.adobe.InCopy", docs: None, ps_names: false, script: Some(INDESIGN), ..MAC },
    AppDef { name: "Adobe After Effects", bundle: "com.adobe.AfterEffects", docs: None, ps_names: true, script: Some(AFTER_EFFECTS), ..MAC },
    AppDef { name: "QuarkXPress", bundle: "com.quark.QuarkXPress", docs: None, ps_names: false, script: Some(QUARK), ..MAC },
    AppDef { name: "Sketch", bundle: "com.bohemiancoding.sketch3", docs: None, ps_names: true, script: None, ..MAC },
    // RightFont lists these too (strings 0x10028c6b5 and 0x10028ac83).
    AppDef { name: "Sketch Beta", bundle: "com.bohemiancoding.sketch3.beta", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Adobe Muse", bundle: "com.adobe.Muse.application", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity", bundle: "com.canva.affinity", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Designer", bundle: "com.seriflabs.affinitydesigner", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Designer 2", bundle: "com.seriflabs.affinitydesigner2", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Photo", bundle: "com.seriflabs.affinityphoto", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Photo 2", bundle: "com.seriflabs.affinityphoto2", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Publisher", bundle: "com.seriflabs.affinitypublisher", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Affinity Publisher 2", bundle: "com.seriflabs.affinitypublisher2", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Pixelmator", bundle: "com.pixelmatorteam.pixelmator", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Swift Publisher 5", bundle: "com.belightsoft.SwiftPublisher5", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Framer", bundle: "com.motif.framer", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "OmniGraffle 7", bundle: "com.omnigroup.OmniGraffle7", docs: None, ps_names: true, script: Some(OMNIGRAFFLE), ..MAC },
    AppDef { name: "SketchBook", bundle: "com.autodesk.SketchBook", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Pages", bundle: "com.apple.iWork.Pages", docs: Some(Docs::Standard), ps_names: true, script: Some(PAGES), ..MAC },
    AppDef { name: "Keynote", bundle: "com.apple.iWork.Keynote", docs: Some(Docs::Standard), ps_names: true, script: Some(KEYNOTE), ..MAC },
    AppDef { name: "Numbers", bundle: "com.apple.iWork.Numbers", docs: Some(Docs::Standard), ps_names: true, script: None, ..MAC },
    AppDef { name: "iMovie", bundle: "com.apple.iMovieApp", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Final Cut Pro", bundle: "com.apple.FinalCut", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "MindNode", bundle: "com.ideasoncanvas.mindnode.macos", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Pixelmator Pro", bundle: "com.pixelmatorteam.pixelmator.x", docs: None, ps_names: true, script: Some(PIXELMATOR), ..MAC },
    AppDef { name: "Adobe XD", bundle: "com.adobe.xd", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Figma", bundle: "com.figma.Desktop", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Microsoft Word", bundle: "com.microsoft.Word", docs: Some(Docs::Word), ps_names: false, script: Some(WORD_EXCEL), ..MAC },
    AppDef { name: "Microsoft Excel", bundle: "com.microsoft.Excel", docs: Some(Docs::Excel), ps_names: false, script: Some(WORD_EXCEL), ..MAC },
    AppDef { name: "Microsoft PowerPoint", bundle: "com.microsoft.Powerpoint", docs: Some(Docs::PowerPoint), ps_names: false, script: Some(POWERPOINT), ..MAC },
    AppDef { name: "TextEdit", bundle: "com.apple.TextEdit", docs: Some(Docs::Standard), ps_names: true, script: Some(TEXTEDIT), ..MAC },
    // Apple Creator Studio apps (their newer bundle ids, also in RightFont's list).
    AppDef { name: "Pages", bundle: "com.apple.Pages", docs: Some(Docs::Standard), ps_names: true, script: Some(PAGES), ..MAC },
    AppDef { name: "Keynote", bundle: "com.apple.Keynote", docs: Some(Docs::Standard), ps_names: true, script: Some(KEYNOTE), ..MAC },
    AppDef { name: "Numbers", bundle: "com.apple.Numbers", docs: Some(Docs::Standard), ps_names: true, script: None, ..MAC },
    AppDef { name: "Final Cut Pro", bundle: "com.apple.FinalCutApp", docs: None, ps_names: true, script: None, ..MAC },
    AppDef { name: "Pixelmator Pro", bundle: "com.apple.pixelmator", docs: None, ps_names: true, script: Some(PIXELMATOR), ..MAC },
    // Added at the person's request (not in RightFont's list), for macOS, Windows and Linux.
    // Office suites take pasted rich text with its font.
    // LibreOffice for Mac reads the fonts once, when it opens (no font-change listener in its
    // libraries), so a font turned on while it is open shows only after reopening it.
    // On a Mac LibreOffice is one app for all its modules.
    AppDef { name: "LibreOffice", bundle: "org.libreoffice.script", ps_names: false, fonts: FontRead::AtStartOnMac, ..MAC },
    // On Linux and Windows each module has its own launcher (desktop entry or program); all of them
    // run as LibreOffice's one process, soffice.bin.
    AppDef { name: "LibreOffice Writer", bundle: "office.libreoffice.writer", ps_names: false, linux: &["libreoffice-writer", "org.libreoffice.LibreOffice.writer", "soffice.bin"], win: &["swriter.exe", "soffice.bin"], ..MAC },
    AppDef { name: "LibreOffice Calc", bundle: "office.libreoffice.calc", ps_names: false, linux: &["libreoffice-calc", "org.libreoffice.LibreOffice.calc", "soffice.bin"], win: &["scalc.exe", "soffice.bin"], ..MAC },
    AppDef { name: "LibreOffice Impress", bundle: "office.libreoffice.impress", ps_names: false, linux: &["libreoffice-impress", "org.libreoffice.LibreOffice.impress", "soffice.bin"], win: &["simpress.exe", "soffice.bin"], ..MAC },
    AppDef { name: "LibreOffice Draw", bundle: "office.libreoffice.draw", ps_names: false, linux: &["libreoffice-draw", "org.libreoffice.LibreOffice.draw", "soffice.bin"], win: &["sdraw.exe", "soffice.bin"], ..MAC },
    AppDef { name: "LibreOffice Base", bundle: "office.libreoffice.base", ps_names: false, linux: &["libreoffice-base", "org.libreoffice.LibreOffice.base", "soffice.bin"], win: &["sbase.exe", "soffice.bin"], ..MAC },
    AppDef { name: "LibreOffice Math", bundle: "office.libreoffice.math", ps_names: false, linux: &["libreoffice-math", "org.libreoffice.LibreOffice.math", "soffice.bin"], win: &["smath.exe", "soffice.bin"], ..MAC },
    AppDef { name: "ONLYOFFICE", bundle: "asc.onlyoffice.ONLYOFFICE", ps_names: false, linux: &["onlyoffice-desktopeditors", "org.onlyoffice.desktopeditors", "DesktopEditors"], win: &["DesktopEditors.exe", "editors.exe"], ..MAC },
    AppDef { name: "Microsoft Word", bundle: "win.microsoft.word", ps_names: false, win: &["WINWORD.EXE"], ..MAC },
    AppDef { name: "Microsoft PowerPoint", bundle: "win.microsoft.powerpoint", ps_names: false, win: &["POWERPNT.EXE"], ..MAC },
    AppDef { name: "Microsoft Publisher", bundle: "win.microsoft.publisher", ps_names: false, win: &["MSPUB.EXE"], ..MAC },
    AppDef { name: "Adobe Photoshop", bundle: "win.adobe.photoshop", win: &["Photoshop.exe"], ..MAC },
    AppDef { name: "Adobe Illustrator", bundle: "win.adobe.illustrator", win: &["Illustrator.exe"], ..MAC },
    AppDef { name: "Adobe InDesign", bundle: "win.adobe.indesign", ps_names: false, win: &["InDesign.exe"], ..MAC },
    AppDef { name: "Affinity", bundle: "win.affinity", win: &["Affinity.exe", "Designer.exe", "Photo.exe", "Publisher.exe"], ..MAC },
    AppDef { name: "Figma", bundle: "win.figma", win: &["Figma.exe"], ..MAC },
    AppDef { name: "CorelDRAW", bundle: "com.corel.coreldraw", ps_names: false, win: &["CorelDRW.exe"], ..MAC },
    AppDef { name: "Lunacy", bundle: "com.icons8.Lunacy", linux: &["lunacy"], win: &["Lunacy.exe"], ..MAC },
    AppDef { name: "Linearity Curve", bundle: "com.linearity.vn", ..MAC },
    // The ArtCraft apps (Rust, winit + Skia) read the installed fonts when they open; VectorCraft
    // and DesignCraft also have a menu command to read them again, which FontCraft chooses for them
    // before changing the font. The selected text's font changes with VectorCraft's own shortcut,
    // DesignCraft's control command, or copy → swap font → paste (family names).
    AppDef { name: "VectorCraft", bundle: "ai.storyteller.vectorcraft", ps_names: false, fonts: FontRead::Menu(VECTORCRAFT_REFRESH), linux: &["vectorcraft"], win: &["VectorCraft.exe"], ..MAC },
    AppDef { name: "DesignCraft", bundle: "ai.storyteller.designcraft", ps_names: false, fonts: FontRead::Menu(DESIGNCRAFT_RELOAD), linux: &["designcraft"], win: &["DesignCraft.exe"], ..MAC },
    AppDef { name: "PhotoCraft", bundle: "ai.storyteller.photocraft", ps_names: false, fonts: FontRead::AtStart, linux: &["photocraft"], win: &["PhotoCraft.exe"], ..MAC },
    AppDef { name: "PdfCraft", bundle: "ai.storyteller.pdfcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["pdfcraft"], win: &["PdfCraft.exe"], ..MAC },
    AppDef { name: "TextureCraft", bundle: "ai.storyteller.texturecraft", ps_names: false, fonts: FontRead::AtStart, linux: &["texturecraft"], win: &["TextureCraft.exe"], ..MAC },
    AppDef { name: "SolidCraft", bundle: "io.tangencylabs.solidcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["solidcraft"], win: &["SolidCraft.exe"], ..MAC },
    // More ArtCraft apps the person uses (bundle ids in the family's pattern; the program names find
    // them on every system). The text, slide and sheet apps get the selected text's font by paste.
    AppDef { name: "WordCraft", bundle: "ai.storyteller.wordcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["wordcraft"], win: &["WordCraft.exe"], ..MAC },
    AppDef { name: "DeckCraft", bundle: "ai.storyteller.deckcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["deckcraft"], win: &["DeckCraft.exe"], ..MAC },
    AppDef { name: "GridCraft", bundle: "ai.storyteller.gridcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["gridcraft"], win: &["GridCraft.exe"], ..MAC },
    AppDef { name: "FilmCraft", bundle: "ai.storyteller.filmcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["filmcraft"], win: &["FilmCraft.exe"], ..MAC },
    AppDef { name: "PotCraft", bundle: "ai.storyteller.potcraft", ps_names: false, fonts: FontRead::AtStart, linux: &["potcraft"], win: &["PotCraft.exe"], ..MAC },
    AppDef { name: "AniCraft", bundle: "ai.storyteller.anicraft", ps_names: false, fonts: FontRead::AtStart, linux: &["anicraft"], win: &["AniCraft.exe"], ..MAC },
    AppDef { name: "Canva", bundle: "com.canva.CanvaDesktop", linux: &["canva"], win: &["Canva.exe"], ..MAC },
    AppDef { name: "Corel PHOTO-PAINT", bundle: "com.corel.photopaint", win: &["PhotoPnt.exe"], ..MAC },
    AppDef { name: "Adobe Express", bundle: "com.adobe.express", win: &["Adobe Express.exe", "Express.exe"], ..MAC },
    AppDef { name: "Microsoft Designer", bundle: "com.microsoft.designer", win: &["Designer.exe"], ..MAC },
    AppDef { name: "Microsoft Visio", bundle: "win.microsoft.visio", win: &["VISIO.EXE"], ..MAC },
    AppDef { name: "Boxy SVG", bundle: "com.boxy-svg.boxy-svg-mac", linux: &["boxy-svg"], win: &["Boxy SVG.exe", "boxy-svg.exe"], ..MAC },
    AppDef { name: "Inkscape", bundle: "org.inkscape.Inkscape", ps_names: false, how: How::Activate, linux: &["org.inkscape.Inkscape", "inkscape"], win: &["inkscape.exe"], ..MAC },
    AppDef { name: "Scribus", bundle: "net.scribus", ps_names: false, how: How::Activate, linux: &["scribus", "net.scribus.Scribus"], win: &["Scribus.exe"], ..MAC },
    AppDef { name: "Krita", bundle: "org.krita", ps_names: false, how: How::Activate, linux: &["org.kde.krita", "krita"], win: &["krita.exe"], ..MAC },
    AppDef { name: "GIMP", bundle: "org.gimp.gimp-2.10", ps_names: false, how: How::Activate, linux: &["org.gimp.GIMP", "gimp"], win: &["gimp-2.10.exe", "gimp-3.0.exe", "gimp.exe"], ..MAC },
    AppDef { name: "GIMP", bundle: "org.gimp.gimp-3", ps_names: false, how: How::Activate, ..MAC },
    AppDef { name: "Motion", bundle: "com.apple.motionapp", how: How::Activate, ..MAC },
];

pub struct InstalledApp {
    def: &'static AppDef,
    pub path: PathBuf,
    /// How its running program shows in the process list (path prefix on macOS, program names elsewhere).
    procs: Vec<String>,
    /// Its version, shown next to its name (RightFont's versionStringForApp:, 0x100005f14).
    pub version: Option<String>,
    icon: Option<ColorImage>,
    tex: Option<TextureHandle>,
}

impl InstalledApp {
    pub fn def_name(&self) -> &'static str {
        self.def.name
    }
}

/// What the watcher thread sees.
#[derive(Default)]
struct Watch {
    running: HashSet<String>,
    /// The supported app that was in front most recently (FontCraft itself doesn't count).
    front: Option<String>,
    /// Open document names, for apps that have fonts turned on for one of their documents.
    docs: HashMap<String, Vec<String>>,
    want_docs: HashSet<String>,
    /// The document open in front in the app FontCraft works with, and that app's key.
    front_doc: Option<(PathBuf, String)>,
}

/// The file of the document open in front in an app, best effort on each system:
/// macOS the front window's AXDocument (System Events); Linux the document named on the command
/// line of the app's process (/proc); Windows the same, from the process list (PowerShell).
fn front_document_of(bundle: &str, procs: &[String]) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let _ = procs;
        if stopped_checking().lock().is_ok_and(|s| s.contains(bundle)) {
            return None;
        }
        let url = match osascript_code(&format!(
            "tell application \"System Events\" to tell (first process whose bundle identifier is \"{bundle}\")\n\
             try\n\
             return value of attribute \"AXDocument\" of window 1\n\
             end try\n\
             end tell"
        )) {
            Ok(u) => u,
            Err(code) if STOP_CHECKING_ERRORS.contains(&code) => {
                if let Ok(mut s) = stopped_checking().lock() {
                    s.insert(bundle.to_string());
                }
                return None;
            }
            Err(_) => String::new(),
        };
        if let Some(path) = url.strip_prefix("file://") {
            let p = PathBuf::from(percent_decode(path));
            if p.is_file() {
                return Some(p);
            }
        }
        // Apps that don't report their document (LibreOffice, ONLYOFFICE…): the front window's title
        // ("Report.odt - LibreOffice Writer", "Report.docx") names the file; Spotlight finds it and the
        // most recently changed match is taken.
        let title = osascript(&format!("tell application \"System Events\" to tell (first process whose bundle identifier is \"{bundle}\") to get name of window 1"))?;
        let name = [" - ", " — ", " – "].iter().fold(title.trim().to_string(), |t, sep| t.split(sep).next().unwrap_or(&t).trim().to_string());
        let name = name.trim_start_matches('*').trim().to_string();
        if name.is_empty() || name.contains('\'') || !crate::docfonts::readable(std::path::Path::new(&name)) {
            return None;
        }
        let out = crate::paths::command("mdfind").arg(format!("kMDItemFSName == '{name}'")).output().ok()?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(PathBuf::from)
            .filter(|p| p.is_file())
            .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        let _ = bundle;
        let rd = std::fs::read_dir("/proc").ok()?;
        for e in rd.flatten() {
            let dir = e.path();
            let comm = std::fs::read_to_string(dir.join("comm")).unwrap_or_default();
            if !procs.iter().any(|p| p == comm.trim()) {
                continue;
            }
            let cmd = std::fs::read(dir.join("cmdline")).unwrap_or_default();
            for arg in cmd.split(|b| *b == 0).skip(1).collect::<Vec<_>>().into_iter().rev() {
                let p = PathBuf::from(String::from_utf8_lossy(arg).to_string());
                if p.is_file() && crate::docfonts::readable(&p) {
                    return Some(p);
                }
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    {
        let _ = bundle;
        let names: Vec<String> = procs.iter().map(|p| format!("Name='{}'", if p.ends_with(".exe") || p.ends_with(".bin") { p.clone() } else { format!("{p}.exe") })).collect();
        let q = format!("(Get-CimInstance Win32_Process -Filter \"{}\").CommandLine", names.join(" or "));
        let out = crate::paths::command("powershell").args(["-NoProfile", "-Command", &q]).output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        for line in text.lines() {
            for part in line.split('"') {
                let p = PathBuf::from(part.trim());
                if p.is_file() && crate::docfonts::readable(&p) {
                    return Some(p);
                }
            }
        }
        None
    }
    #[cfg(target_os = "android")]
    {
        let _ = (bundle, procs);
        None
    }
}

/// The file names of the documents open in an app's windows (macOS: each window's AXDocument), so
/// fonts turned on for one of them go off when it closes. None when the app can't be asked.
fn window_documents(bundle: &str) -> Option<Vec<String>> {
    #[cfg(target_os = "macos")]
    {
        let out = osascript(&format!(
            "tell application \"System Events\" to tell (first process whose bundle identifier is \"{bundle}\")\n\
             set out to \"\"\n\
             repeat with w in windows\n\
             try\n\
             set d to value of attribute \"AXDocument\" of w\n\
             if d is not missing value then set out to out & d & linefeed\n\
             end try\n\
             end repeat\n\
             return out\n\
             end tell"
        ))?;
        Some(
            out.lines()
                .filter_map(|u| u.strip_prefix("file://"))
                .map(|p| percent_decode(p).trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string())
                .filter(|n| !n.is_empty())
                .collect(),
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bundle;
        None
    }
}

/// "%20" and the like back to their characters.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// A font turned on for an app (and maybe one of its documents).
struct Record {
    files: Vec<PathBuf>,
    bundle: String,
    doc: Option<String>,
}

pub enum Outcome {
    Applied(String, Option<String>),
    /// The font was only turned on for the app (app name, font name).
    Activated(String, String),
    Inserted(String),
    NoSelection(String),
    NeedsAccess,
    /// The app read the fonts again (its own menu command).
    Refreshed(String),
    /// The app has to be reopened to see the fonts just turned on.
    Reopen(String),
}

#[derive(Default)]
pub struct AutoAct {
    pub apps: Vec<InstalledApp>,
    watch: Arc<Mutex<Watch>>,
    records: Vec<Record>,
    pending: Arc<Mutex<Vec<(Vec<PathBuf>, String, Outcome)>>>,
    /// The installed apps as the watcher thread sees them.
    shared_apps: Arc<Mutex<Vec<(String, PathBuf, Option<Docs>, Vec<String>)>>>,
    started: bool,
    /// The Auto Activation settings were open last frame (they look for apps again when opened).
    pub settings_seen: bool,
    /// A glyph being dragged from the Glyphs page: (glyph id, font face).
    pub dragging: Option<(u16, usize)>,
    /// Where the target app's icon is in the toolbar (drop a glyph there to insert it).
    pub target_rect: Option<Rect>,
    /// The fonts that were on last frame (to tell open apps about fonts turned on since).
    seen_on: Option<HashSet<PathBuf>>,
}

impl AutoAct {
    /// The document open in front in the app FontCraft works with, and that app's key.
    pub fn front_document(&self) -> Option<(PathBuf, String)> {
        self.watch.lock().ok()?.front_doc.clone()
    }

    /// Remembers fonts turned on for an app (and its document, when the app reports documents),
    /// so they go off when the document or the app closes.
    pub fn record(&mut self, files: Vec<PathBuf>, bundle: String, doc: Option<String>) {
        if doc.is_some() {
            if let Ok(mut w) = self.watch.lock() {
                w.want_docs.insert(bundle.clone());
            }
        }
        self.records.push(Record { files, bundle, doc });
    }
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> u8;
}

/// Whether macOS lets FontCraft send keystrokes to other apps (Accessibility permission).
/// Whether FontCraft may send keys to other apps, asked the way RightFont asks right before each
/// copy or paste (`-[RFFontActivator runCopyOnProcess:]` 0x10005191c, `runPasteOnProcess:`
/// 0x100051a38, `fontAtTE:` 0x100052398): AXIsProcessTrustedWithOptions with
/// {kAXTrustedCheckOptionPrompt: kCFBooleanTrue}. When it may not, macOS shows its own request
/// (once per app identity) and the action is skipped.
pub fn trusted_or_ask() -> bool {
    #[cfg(target_os = "macos")]
    {
        crate::fcl::ask_accessibility()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn trusted() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { AXIsProcessTrusted() != 0 }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// The PNG pictures inside an .icns file; the one closest to 64 px.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn read_icns(path: &Path) -> Option<ColorImage> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 8 || &data[..4] != b"icns" {
        return None;
    }
    let mut best: Option<image::RgbaImage> = None;
    let mut i = 8;
    while i + 8 <= data.len() {
        let len = u32::from_be_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
        if len < 8 || i + len > data.len() {
            break;
        }
        let body = &data[i + 8..i + len];
        if body.starts_with(b"\x89PNG") {
            if let Ok(img) = image::load_from_memory_with_format(body, image::ImageFormat::Png) {
                let img = img.to_rgba8();
                let score = |w: u32| (w as i64 - 64).abs();
                if best.as_ref().map(|b| score(img.width()) < score(b.width())).unwrap_or(true) {
                    best = Some(img);
                }
            }
        }
        i += len;
    }
    let img = best?;
    let (w, h) = (img.width() as usize, img.height() as usize);
    Some(ColorImage::from_rgba_unmultiplied([w, h], img.as_raw()))
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn app_icon(app: &Path) -> Option<ColorImage> {
    let res = app.join("Contents/Resources");
    if let Ok(plist) = plist::Value::from_file(app.join("Contents/Info.plist")) {
        if let Some(dict) = plist.as_dictionary() {
            if let Some(name) = dict.get("CFBundleIconFile").or_else(|| dict.get("CFBundleIconName")).and_then(|v| v.as_string()) {
                let file = if name.ends_with(".icns") { name.to_string() } else { format!("{name}.icns") };
                if let Some(img) = read_icns(&res.join(&file)) {
                    return Some(img);
                }
            }
        }
    }
    // Fallback: check standard icon names in Resources
    for cand in ["AppIcon.icns", "app.icns", "icon.icns"] {
        if let Some(img) = read_icns(&res.join(cand)) {
            return Some(img);
        }
    }
    None
}

/// An app's version from its Info.plist (CFBundleShortVersionString, the key RightFont reads,
/// string at 0x100296281).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn bundle_version(app: &Path) -> Option<String> {
    let plist = plist::Value::from_file(app.join("Contents/Info.plist")).ok()?;
    let v = plist.as_dictionary()?.get("CFBundleShortVersionString")?.as_string()?.trim().to_string();
    (!v.is_empty()).then_some(v)
}

/// Whether an app is strictly excluded (email clients like Outlook, browsers, viewers, dev tools).
fn is_excluded(bundle: &str, name: &str) -> bool {
    let b = bundle.to_ascii_lowercase();
    let n = name.to_ascii_lowercase();
    // Email & communication (e.g. Microsoft Outlook, Thunderbird, Apple Mail)
    b.contains("outlook") || n.contains("outlook")
    || b.contains("thunderbird") || n.contains("thunderbird")
    || b.contains("mail") || n.contains("mail")
    // Web browsers
    || b.contains("chrome") || n.contains("chrome")
    || b.contains("safari") || n.contains("safari")
    || b.contains("firefox") || n.contains("firefox")
    || b.contains("msedge") || n.contains("msedge") || b.contains("edge") || n.contains("edge")
    || b.contains("brave") || n.contains("brave")
    || b.contains("opera") || n.contains("opera")
    || b.contains("arc") || n.contains("arc")
    || b.contains("vivaldi") || n.contains("vivaldi")
    || b.contains("browser") || n.contains("browser")
    // Viewers & non-typographic readers (Preview, Acrobat Reader)
    || b.contains("preview") || n.contains("preview")
    || b.contains("reader") || n.contains("reader")
    || (b.contains("acrobat") && !b.contains("distiller"))
    // Dev tools & code editors
    || b.contains("vscode") || b.contains("visualstudio") || n.contains("code") || n.contains("visual studio")
    || b.contains("sublime") || n.contains("sublime")
    || b.contains("xcode") || n.contains("xcode")
    || b.contains("terminal") || n.contains("terminal") || b.contains("iterm") || n.contains("iterm")
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn bundle_id(app: &Path) -> Option<String> {
    let plist = plist::Value::from_file(app.join("Contents/Info.plist")).ok()?;
    Some(plist.as_dictionary()?.get("CFBundleIdentifier")?.as_string()?.to_string())
}

/// The supported apps installed on this computer, checked each time FontCraft starts (and when the
/// Auto Activation settings open), so apps installed or removed since then appear or disappear.
/// macOS: bundles in the Applications folders; Linux: desktop entries (system, user, Flatpak, Snap);
/// Windows: programs in Program Files and the user's programs.
pub fn find_installed() -> Vec<InstalledApp> {
    let mut found: Vec<InstalledApp> = Vec::new();
    // Android: the installed apps that set text in fonts, from the package list (the app declares
    // QUERY_ALL_PACKAGES so Android shows it every package). FontCraft injects fonts into shared
    // font folders and broadcasts events to ArtCraft and Android creative/office apps.
    #[cfg(target_os = "android")]
    {
        const ANDROID_APPS: &[(&str, &str)] = &[
            ("VectorCraft", "ai.storyteller.vectorcraft"),
            ("DesignCraft", "ai.storyteller.designcraft"),
            ("PhotoCraft", "ai.storyteller.photocraft"),
            ("WordCraft", "ai.storyteller.wordcraft"),
            ("DeckCraft", "ai.storyteller.deckcraft"),
            ("GridCraft", "ai.storyteller.gridcraft"),
            ("FilmCraft", "ai.storyteller.filmcraft"),
            ("PotCraft", "ai.storyteller.potcraft"),
            ("SolidCraft", "io.tangencylabs.solidcraft"),
            ("PdfCraft", "ai.storyteller.pdfcraft"),
            ("TextureCraft", "ai.storyteller.texturecraft"),
            ("Microsoft Word", "com.microsoft.office.word"),
            ("Microsoft Excel", "com.microsoft.office.excel"),
            ("Microsoft PowerPoint", "com.microsoft.office.powerpoint"),
            ("Microsoft 365", "com.microsoft.office.officehubrow"),
            ("Google Docs", "com.google.android.apps.docs.editors.docs"),
            ("Google Sheets", "com.google.android.apps.docs.editors.sheets"),
            ("Google Slides", "com.google.android.apps.docs.editors.slides"),
            ("LibreOffice", "org.documentfoundation.libreoffice"),
            ("Collabora Office", "com.collabora.libreoffice"),
            ("ONLYOFFICE", "com.onlyoffice.documents"),
            ("WPS Office", "cn.wps.moffice_eng"),
            ("Polaris Office", "com.infraware.office.link"),
            ("Canva", "com.canva.editor"),
            ("Adobe Express", "com.adobe.spark.post"),
            ("Adobe Photoshop Express", "com.adobe.psmobile"),
            ("Adobe Fresco", "com.adobe.fresco"),
            ("Infinite Painter", "com.brakefield.painter"),
            ("ibis Paint X", "jp.ne.ibis.ibispaintx.app"),
            ("Krita", "org.krita"),
            ("Picsart", "com.picsart.studio"),
            ("PixelLab", "com.imaginstudio.imagetools.pixellab"),
            ("Phonto", "com.youthhr.phonto"),
            ("Affinity", "com.canva.affinity"),
        ];
        let list = crate::paths::command("pm").args(["list", "packages"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let have: HashSet<&str> = list.lines().filter_map(|l| l.strip_prefix("package:")).map(str::trim).collect();
        for (name, pkg) in ANDROID_APPS {
            if have.contains(pkg) {
                let def: &'static AppDef = Box::leak(Box::new(AppDef { name, bundle: pkg, how: How::Activate, ..MAC }));
                found.push(InstalledApp { def, icon: None, path: PathBuf::from(pkg), procs: vec![pkg.to_string()], version: None, tex: None });
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![PathBuf::from("/Applications"), PathBuf::from("/System/Applications")];
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join("Applications"));
        }
        let mut candidates: Vec<(PathBuf, String, String)> = Vec::new();
        for d in &dirs {
            let Ok(rd) = std::fs::read_dir(d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                let mut add = |p: PathBuf| {
                    if let Some(id) = bundle_id(&p) {
                        let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                        if !is_excluded(&id, &stem) {
                            candidates.push((p, id, stem));
                        }
                    }
                };
                if p.extension().is_some_and(|x| x == "app") {
                    add(p);
                } else if p.is_dir() {
                    if let Ok(inner) = std::fs::read_dir(&p) {
                        for q in inner.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "app")) {
                            add(q);
                        }
                    }
                }
            }
        }
        for def in CATALOG {
            if def.bundle.is_empty() || found.iter().any(|f| f.def.bundle == def.bundle) {
                continue;
            }
            if let Some((path, _, _)) = candidates.iter().find(|(_, id, stem)| id == def.bundle || def.name.eq_ignore_ascii_case(stem)) {
                let procs = vec![format!("{}/Contents/MacOS/", path.display())];
                found.push(InstalledApp { def, icon: app_icon(path), path: path.clone(), procs, version: bundle_version(path), tex: None });
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let dirs = [
            "/usr/share/applications".to_string(),
            "/usr/local/share/applications".to_string(),
            "/var/lib/flatpak/exports/share/applications".to_string(),
            "/var/lib/snapd/desktop/applications".to_string(),
            format!("{home}/.local/share/applications"),
            format!("{home}/.local/share/flatpak/exports/share/applications"),
        ];
        let mut entries: Vec<(PathBuf, String, String, String)> = Vec::new(); // (file, stem, exec, icon)
        for d in &dirs {
            let Ok(rd) = std::fs::read_dir(d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().is_none_or(|x| x != "desktop") {
                    continue;
                }
                let text = std::fs::read_to_string(&p).unwrap_or_default();
                let field = |k: &str| text.lines().find_map(|l| l.strip_prefix(k)).unwrap_or("").trim().to_string();
                let hidden = field("NoDisplay=") == "true";
                if hidden {
                    continue;
                }
                let exec = field("Exec=").split_whitespace().next().unwrap_or("").rsplit('/').next().unwrap_or("").to_string();
                let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                if is_excluded(&stem, &exec) {
                    continue;
                }
                entries.push((p, stem, exec, field("Icon=")));
            }
        }
        for def in CATALOG.iter().filter(|d| !d.linux.is_empty()) {
            if found.iter().any(|f| f.def.name == def.name) {
                continue;
            }
            if let Some((path, _, exec, icon)) = entries.iter().find(|(_, stem, exec, _)| def.linux.iter().any(|n| n == stem || n == exec) || def.name.eq_ignore_ascii_case(stem)) {
                let procs = def.linux.iter().map(|n| n.to_string()).chain(std::iter::once(exec.clone())).collect();
                found.push(InstalledApp { def, icon: linux_icon(icon), path: path.clone(), procs, version: None, tex: None });
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        let mut roots: Vec<PathBuf> = ["ProgramFiles", "ProgramFiles(x86)", "ProgramW6432"].iter().filter_map(|v| std::env::var(v).ok()).map(PathBuf::from).collect();
        if let Ok(l) = std::env::var("LOCALAPPDATA") {
            roots.push(PathBuf::from(l).join("Programs"));
        }
        let vendor_subdirs = [
            "", "Adobe", "Microsoft Office", "Microsoft Office/root/Office16", "Microsoft Office/Office16",
            "Affinity", "Canva", "Corel", "Figma", "Inkscape", "Inkscape/bin", "GIMP 2", "GIMP 2/bin",
            "GIMP 3", "GIMP 3/bin", "Krita (x64)", "Krita (x64)/bin", "Scribus", "Boxy SVG",
            "LibreOffice", "LibreOffice/program", "ONLYOFFICE", "ONLYOFFICE/DesktopEditors",
            "Storyteller", "ArtCraft", "Icons8/Lunacy",
        ];
        let mut candidates: Vec<PathBuf> = Vec::new();
        for r in &roots {
            for sub in &vendor_subdirs {
                let dir = if sub.is_empty() { r.clone() } else { r.join(sub) };
                if let Ok(rd) = std::fs::read_dir(&dir) {
                    for e in rd.flatten() {
                        let p = e.path();
                        if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("exe")) {
                            candidates.push(p);
                        } else if p.is_dir() {
                            if let Ok(sub_rd) = std::fs::read_dir(&p) {
                                for se in sub_rd.flatten() {
                                    let sp = se.path();
                                    if sp.extension().is_some_and(|x| x.eq_ignore_ascii_case("exe")) {
                                        candidates.push(sp);
                                    } else if sp.is_dir() && sp.file_name().is_some_and(|n| n == "bin" || n == "program" || n == "Support Files") {
                                        if let Ok(bin_rd) = std::fs::read_dir(&sp) {
                                            for be in bin_rd.flatten() {
                                                let bp = be.path();
                                                if bp.extension().is_some_and(|x| x.eq_ignore_ascii_case("exe")) {
                                                    candidates.push(bp);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for def in CATALOG.iter().filter(|d| !d.win.is_empty()) {
            if found.iter().any(|f| f.def.name == def.name) {
                continue;
            }
            let hit = candidates.iter().find(|p| p.file_name().is_some_and(|n| def.win.iter().any(|w| n.to_string_lossy().eq_ignore_ascii_case(w))));
            if let Some(path) = hit {
                let procs = def.win.iter().map(|w| w.to_lowercase()).collect();
                found.push(InstalledApp { def, icon: None, path: path.clone(), procs, version: None, tex: None });
            }
        }
    }
    found.sort_by_key(|a| a.def.name.to_lowercase());
    found
}

/// A Linux app's icon: a PNG from the icon theme (the size nearest 64 px) or an absolute path.
#[cfg(all(unix, not(target_os = "macos")))]
fn linux_icon(name: &str) -> Option<ColorImage> {
    let mut paths: Vec<PathBuf> = Vec::new();
    if name.starts_with('/') {
        paths.push(PathBuf::from(name));
    } else if !name.is_empty() {
        let home = std::env::var("HOME").unwrap_or_default();
        for base in ["/usr/share/icons/hicolor".to_string(), format!("{home}/.local/share/icons/hicolor"), "/var/lib/flatpak/exports/share/icons/hicolor".to_string()] {
            for size in ["64x64", "48x48", "128x128", "96x96", "256x256", "32x32"] {
                paths.push(PathBuf::from(format!("{base}/{size}/apps/{name}.png")));
            }
        }
        paths.push(PathBuf::from(format!("/usr/share/pixmaps/{name}.png")));
    }
    let p = paths.into_iter().find(|p| p.exists())?;
    let img = image::open(p).ok()?.to_rgba8();
    Some(ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw()))
}

/// Which of the apps are open now, by their process names or bundle identifiers.
fn running_now(apps: &[(String, Vec<String>)]) -> HashSet<String> {
    let mut running = HashSet::new();
    #[cfg(target_os = "macos")]
    {
        for (key, _) in apps {
            if app_running(key) {
                running.insert(key.clone());
            }
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    let list = crate::paths::command("ps").args(["-axo", "comm="]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    #[cfg(target_os = "windows")]
    let list = crate::paths::command("tasklist").args(["/fo", "csv", "/nh"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase()).unwrap_or_default();
    #[cfg(not(target_os = "macos"))]
    for (key, procs) in apps {
        let hit = list.lines().any(|l| {
            let l = l.trim().trim_matches('"');
            procs.iter().any(|p| {
                let first = l.split("\",\"").next().unwrap_or(l);
                first.rsplit('/').next() == Some(p.as_str()) || first == p.as_str()
            })
        });
        if hit {
            running.insert(key.clone());
        }
    }
    running
}

/// AppleScript errors after which RightFont stops checking an app's document for missing fonts
/// (-[RFFontActivator shouldStopMissingFontTimerForError:includeTargetNotRunning:] 0x10004d488):
/// -600 app not running, -609 connection invalid, -610 no user interaction allowed, -1712 timed
/// out, -1750 scripting system error, -2741 script syntax error.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const STOP_CHECKING_ERRORS: [i32; 6] = [-600, -609, -610, -1712, -1750, -2741];

/// Apps whose document checks stopped after such an error (asked again when they come to the front).
fn stopped_checking() -> &'static Mutex<HashSet<String>> {
    static S: std::sync::OnceLock<Mutex<HashSet<String>>> = std::sync::OnceLock::new();
    S.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Runs AppleScript; the error number osascript reports ("… (-600)") when it fails.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn osascript_code(script: &str) -> Result<String, i32> {
    let out = crate::paths::command("osascript").arg("-e").arg(script).output().map_err(|_| 0)?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    let code = err.rsplit_once('(').and_then(|(_, r)| r.split(')').next()).and_then(|n| n.trim().parse::<i32>().ok()).unwrap_or(0);
    Err(code)
}

fn osascript(script: &str) -> Option<String> {
    let out = crate::paths::command("osascript").arg("-e").arg(script).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn doc_list_script(bundle: &str, kind: Docs) -> String {
    let what = match kind {
        Docs::Standard | Docs::Word => "documents",
        Docs::Excel => "workbooks",
        Docs::PowerPoint => "presentations",
    };
    format!("tell application id \"{bundle}\" to get name of every {}", &what[..what.len() - 1])
}

fn front_doc_script(bundle: &str, kind: Docs) -> String {
    let what = match kind {
        Docs::Standard => "front document",
        Docs::Word => "active document",
        Docs::Excel => "active workbook",
        Docs::PowerPoint => "active presentation",
    };
    format!("tell application id \"{bundle}\" to get name of {what}")
}

/// Waits until `bundle` is the app in front (up to 2 s), then a moment for its window to take the
/// keyboard.
fn wait_front(bundle: &str) {
    let mut waited = 0;
    while !front_bundle().is_some_and(|b| b.eq_ignore_ascii_case(bundle)) && waited < 2000 {
        std::thread::sleep(Duration::from_millis(100));
        waited += 100;
    }
    std::thread::sleep(Duration::from_millis(150));
}

/// The app in front right now (its bundle id), asked directly from Cocoa or lsappinfo.
fn front_bundle() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        let run = || unsafe {
            let ws: *mut AnyObject = msg_send![class!(NSWorkspace), sharedWorkspace];
            if ws.is_null() { return None; }
            let front_app: Option<Retained<AnyObject>> = msg_send![ws, frontmostApplication];
            let Some(front_app) = front_app else { return None };
            let bid: Option<Retained<objc2_foundation::NSString>> = msg_send![&*front_app, bundleIdentifier];
            bid.map(|s| s.to_string())
        };
        if let Ok(Some(bundle)) = objc2::exception::catch(std::panic::AssertUnwindSafe(run)) {
            return Some(bundle);
        }
    }
    let asn = String::from_utf8_lossy(&crate::paths::command("lsappinfo").arg("front").output().ok()?.stdout).trim().to_string();
    let info = String::from_utf8_lossy(&crate::paths::command("lsappinfo").args(["info", "-only", "bundleid", &asn]).output().ok()?.stdout).to_string();
    info.split('"').nth(3).map(String::from)
}

/// What's on the clipboard: (rich text, plain text). Read straight from macOS's pasteboard
/// (the `pbpaste` tool can't hand back rich text reliably).
/// Whether an app (bundle id) is open, asked of the system directly (macOS). Elsewhere, and for an
/// empty id, it counts as open (its fonts then stay on until FontCraft quits).
fn app_running(bundle: &str) -> bool {
    #[cfg(target_os = "macos")]
    if !bundle.is_empty() {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        let id = objc2_foundation::NSString::from_str(bundle);
        let n: usize = unsafe {
            let list: *mut AnyObject = msg_send![class!(NSRunningApplication), runningApplicationsWithBundleIdentifier: &*id];
            if list.is_null() { 0 } else { msg_send![list, count] }
        };
        return n > 0;
    }
    let _ = bundle;
    true
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
/// Chooses the app's menu command that reads the fonts again (any menu of its menu bar, any of
/// `names`), without bringing it to the front. Whether it was found and chosen.
fn reload_fonts_menu(bundle: &str, names: &[&str]) -> bool {
    let list = names.iter().map(|n| format!("\"{}\"", n.replace('"', ""))).collect::<Vec<_>>().join(", ");
    let script = format!(
        "tell application \"System Events\" to tell (first process whose bundle identifier is \"{bundle}\")\n\
         repeat with mb in menu bar items of menu bar 1\n\
         try\n\
         set ns to name of every menu item of menu 1 of mb\n\
         set j to 0\n\
         repeat with n in ns\n\
         set j to j + 1\n\
         if {{{list}}} contains (n as text) then\n\
         click menu item j of menu 1 of mb\n\
         return \"ok\"\n\
         end if\n\
         end repeat\n\
         end try\n\
         end repeat\n\
         repeat with mb in menu bar items of menu bar 1\n\
         try\n\
         repeat with mi in menu items of menu 1 of mb\n\
         try\n\
         set ns to name of every menu item of menu 1 of mi\n\
         set j to 0\n\
         repeat with n in ns\n\
         set j to j + 1\n\
         if {{{list}}} contains (n as text) then\n\
         click menu item j of menu 1 of mi\n\
         return \"ok\"\n\
         end if\n\
         end repeat\n\
         end try\n\
         end repeat\n\
         end try\n\
         end repeat\n\
         return \"none\"\n\
         end tell"
    );
    osascript(&script).is_some_and(|r| r.trim() == "ok")
}

#[cfg(target_os = "macos")]
fn pb_get() -> (Option<String>, String) {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeRTF, NSPasteboardTypeString};
    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        let rtf = pb.dataForType(NSPasteboardTypeRTF).map(|d| String::from_utf8_lossy(&d.to_vec()).to_string());
        let text = pb.stringForType(NSPasteboardTypeString).map(|s| s.to_string()).unwrap_or_default();
        (rtf, text)
    }
}

/// Puts rich text (when given) and plain text on the clipboard, replacing what was there.
#[cfg(target_os = "macos")]
fn pb_set(rtf: Option<&str>, text: &str) {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeRTF, NSPasteboardTypeString};
    use objc2_foundation::{NSData, NSString};
    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        if let Some(r) = rtf {
            pb.setData_forType(Some(&NSData::with_bytes(r.as_bytes())), NSPasteboardTypeRTF);
        }
        pb.setString_forType(&NSString::from_str(text), NSPasteboardTypeString);
    }
}

/// The HTML version on the clipboard, if any.
#[cfg(target_os = "macos")]
fn pb_get_html() -> Option<String> {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeHTML};
    unsafe {
        let pb = NSPasteboard::generalPasteboard();
        pb.stringForType(NSPasteboardTypeHTML).map(|s| s.to_string())
    }
}

/// Rich text, HTML (each when given) and plain text on the clipboard, replacing what was there.
#[cfg(target_os = "macos")]
fn pb_set_all(rtf: Option<&str>, html: Option<&str>, text: &str) {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeHTML};
    use objc2_foundation::NSString;
    pb_set(rtf, text);
    if let Some(h) = html {
        unsafe {
            NSPasteboard::generalPasteboard().setString_forType(&NSString::from_str(h), NSPasteboardTypeHTML);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn pb_get() -> (Option<String>, String) {
    (None, String::new())
}
#[cfg(not(target_os = "macos"))]
fn pb_set(_rtf: Option<&str>, _text: &str) {}
#[cfg(not(target_os = "macos"))]
fn pb_get_html() -> Option<String> {
    None
}
#[cfg(not(target_os = "macos"))]
fn pb_set_all(_rtf: Option<&str>, _html: Option<&str>, _text: &str) {}

/// Command+`key` sent to the app's own process, as RightFont's copy and paste templates do
/// ("tell application \"System Events\" to tell application process \"%@\" to keystroke \"c\" using
/// command down", strings 0x10028b730 and 0x10028b813), so it reaches that app even when another
/// window is still in front. The process is found by its bundle id; without one, the front app gets it.
fn keystroke(bundle: &str, key: &str) {
    let to_process = format!(
        "tell application \"System Events\" to tell (first application process whose bundle identifier is \"{bundle}\") to keystroke \"{key}\" using command down"
    );
    if bundle.is_empty() || osascript_code(&to_process).is_err() {
        let _ = osascript(&format!("tell application \"System Events\" to keystroke \"{key}\" using command down"));
    }
}

fn rtf_escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\n' => out.push_str("\\par\n"),
            '\r' => {}
            c if (c as u32) < 128 => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{}?", *u as i16));
                }
            }
        }
    }
    out
}

/// Rich text of `text` set in `font` without unwanted leading or trailing spaces.
pub fn rtf_for(text: &str, font: &str) -> String {
    format!("{{\\rtf1\\ansi\\ansicpg1252{{\\fonttbl{{\\f0\\fnil\\fcharset0 {font};}}}}\\uc0\\f0{{{}}}}}", rtf_escape(text))
}

/// Copied rich text with every font in its font table replaced by `font` (sizes, colours and the
/// rest of the formatting stay as they were).
pub fn retarget_rtf(rtf: &str, font: &str) -> Option<String> {
    let start = rtf.find("{\\fonttbl")?;
    let bytes = rtf.as_bytes();
    let (mut depth, mut end, mut i) = (0i32, None, start);
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 1,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    let end = end?;
    let table = &rtf[start..=end];
    // Every font number in the table ("\f0", "\f12"…), whether or not each font has its own braces.
    let mut ids: Vec<String> = Vec::new();
    let mut rest = table;
    while let Some(p) = rest.find("\\f") {
        let digits: String = rest[p + 2..].chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && !ids.contains(&digits) {
            ids.push(digits.clone());
        }
        rest = &rest[p + 2..];
    }
    if ids.is_empty() {
        ids.push("0".into());
    }
    let new_table: String =
        String::from("{\\fonttbl") + &ids.iter().map(|id| format!("{{\\f{id}\\fnil\\fcharset0 {font};}}")).collect::<String>() + "}";
    Some(format!("{}{}{}", &rtf[..start], new_table, &rtf[end + 1..]))
}

/// Changes the selected text in the app to `font` (runs off the main thread: it waits for the app).
/// How the last font change reached the app: 1 its own AppleScript, 2 VectorCraft's font search,
/// 3 DesignCraft's command line, 4 copy → swap → paste. Glyph insertion uses it to know whether the
/// app still has the glyph selected afterwards (a paste leaves the cursor after it).
static LAST_ROUTE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn route(r: u8) {
    LAST_ROUTE.store(r, std::sync::atomic::Ordering::Relaxed);
}

fn apply_in_app(bundle: &str, kind: Option<Docs>, font: &str, script: Option<&str>) -> Result<Option<String>, bool> {
    route(0);
    if !trusted_or_ask() {
        return Err(true);
    }
    let _ = osascript(&format!("tell application id \"{bundle}\" to activate"));
    // RightFont's paste template: activate, then "delay 0.1" (string 0x10028b813). Apps that take
    // longer to come to the front (LibreOffice, ONLYOFFICE, Electron and Java apps) would get the
    // copy keystroke while FontCraft is still in front, so FontCraft waits until the app is the
    // frontmost one (up to 2 s) and gives its window a moment to take the keyboard.
    std::thread::sleep(Duration::from_millis(100));
    wait_front(bundle);
    let doc = kind.and_then(|k| osascript(&front_doc_script(bundle, k)));
    // Apps RightFont talks to directly get its own AppleScript (strings 0x10028b730–0x10028d4fa).
    if let Some(s) = script {
        let s = s.replace("{bundle}", bundle).replace("{font}", font);
        route(1);
        if osascript(&s).is_some() {
            return Ok(doc);
        }
        // If AppleScript fails (e.g. text box container selected or script mismatch), fall through to Route 4!
    }
    // VectorCraft: its own "Highlight Font Name" (Cmd+Alt+Shift+F, command type.highlightFontName)
    // opens the Character panel's font family menu with the search field focused; the family is
    // typed and Return applies it to the selected type, keeping its size.
    if bundle == "ai.storyteller.vectorcraft" {
        let typed = font.replace('\\', "\\\\").replace('"', "\\\"");
        let ok = osascript(&format!(
            "tell application \"System Events\"\n\
             keystroke \"f\" using {{command down, option down, shift down}}\n\
             delay 0.4\n\
             keystroke \"{typed}\"\n\
             delay 0.5\n\
             key code 36\n\
             end tell"
        ));
        route(2);
        return if ok.is_some() { Ok(doc) } else { Err(false) };
    }
    // DesignCraft: its type.char command sets the selected text's font family (size kept), sent
    // through designcraft-cli to the running app's control port (DESIGNCRAFT_CONTROL_PORT, else
    // 7979) when DesignCraft was started with it; otherwise the copy → swap → paste way below.
    if bundle == "ai.storyteller.designcraft" {
        let port = std::env::var("DESIGNCRAFT_CONTROL_PORT").unwrap_or_else(|_| "7979".into());
        let cli = std::path::Path::new("/Applications/DesignCraft.app/Contents/MacOS/designcraft-cli");
        if cli.exists() {
            let args = serde_json::json!({ "attrs": { "fontFamily": font } }).to_string();
            let ok = crate::paths::command(cli.to_str().unwrap_or_default()).args(["app", "--port", &port, "type.char", &args]).output().is_ok_and(|o| o.status.success());
            if ok {
                route(3);
                return Ok(doc);
            }
        }
    }
    // Everything else: copy, swap the font, paste.
    route(4);
    let (saved_rtf, saved_text) = pb_get();
    let saved_html = pb_get_html();
    let marker = "\u{2063}fontcraft\u{2063}";
    pb_set(None, marker);
    keystroke(bundle, "c");
    // RightFont waits 0.2 s after copying (string 0x10028b730); slower apps (LibreOffice,
    // ONLYOFFICE) put the copy on the clipboard later, so FontCraft waits until it is there, up to 2 s.
    let mut waited = 0;
    while pb_get().1 == marker && waited < 2000 {
        std::thread::sleep(Duration::from_millis(100));
        waited += 100;
    }
    // A moment more for the rich versions, which some apps add after the plain text.
    std::thread::sleep(Duration::from_millis(150));
    let (rtf, plain) = pb_get();
    let html = pb_get_html();
    let rtf = rtf.filter(|r| r.starts_with("{\\rtf"));
    if plain == marker || (plain.is_empty() && rtf.is_none() && html.is_none()) {
        pb_set_all(saved_rtf.as_deref(), saved_html.as_deref(), &saved_text);
        return Err(false);
    }
    // The text keeps its size: the app's own rich text is changed (only its fonts). An RTF without
    // sizes (ONLYOFFICE, browsers) loses to the app's HTML, which keeps them.
    let rtf_has_size = rtf.as_deref().is_some_and(|r| r.contains("\\fs"));
    match (&rtf, &html) {
        (_, Some(h)) if !rtf_has_size => {
            let styled = retarget_html(h, font);
            pb_set_all(None, Some(&styled), &plain);
        }
        (Some(r), _) => {
            let styled = retarget_rtf(r, font).unwrap_or_else(|| rtf_for(&plain, font));
            pb_set_all(Some(&styled), None, &plain);
        }
        _ => pb_set_all(Some(&rtf_for(&plain, font)), None, &plain),
    }
    keystroke(bundle, "v");
    // The app reads the clipboard when it pastes, which can take a moment: the person's clipboard
    // comes back only after that.
    std::thread::sleep(Duration::from_millis(1200));
    pb_set_all(saved_rtf.as_deref(), saved_html.as_deref(), &saved_text);
    Ok(doc)
}

/// HTML with every font-family (CSS) and <font face> set to `font`; text without any gets a span.
pub fn retarget_html(html: &str, font: &str) -> String {
    let fam = format!("'{}'", font.replace('\'', ""));
    let mut out = String::with_capacity(html.len() + 64);
    let mut rest = html;
    let mut changed = false;
    while let Some(i) = rest.find("font-family:") {
        out.push_str(&rest[..i + "font-family:".len()]);
        rest = &rest[i + "font-family:".len()..];
        // The value runs to ';' or the end of the attribute ('"').
        let end = rest.find([';', '"']).unwrap_or(rest.len());
        out.push_str(&fam);
        rest = &rest[end..];
        changed = true;
    }
    out.push_str(rest);
    let mut out2 = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(i) = rest.find("face=\"") {
        out2.push_str(&rest[..i + 6]);
        rest = &rest[i + 6..];
        let end = rest.find('"').unwrap_or(rest.len());
        out2.push_str(font);
        rest = &rest[end..];
        changed = true;
    }
    out2.push_str(rest);
    if changed {
        return out2;
    }
    // No fonts named: the copied content is wrapped in a span with the font.
    match (out2.find("<body"), out2.rfind("</body>")) {
        (Some(b), Some(e)) => {
            let open = out2[b..].find('>').map(|x| b + x + 1).unwrap_or(b);
            format!("{}<span style=\"font-family:{fam}\">{}</span>{}", &out2[..open], &out2[open..e], &out2[e..])
        }
        _ => format!("<span style=\"font-family:{fam}\">{out2}</span>"),
    }
}

/// Puts `text` in `font` into the app's document at the cursor.
/// Puts `text` in `font` into the app's document at the cursor, keeping the size and style the app
/// is using there: the text is pasted as plain text (so it takes the app's own formatting), selected
/// (Shift+Left once per character), then changed to the font the same way "use font" does (the app's
/// script, or copy → swap the font → paste, which keeps the size). Apps that only take fonts from
/// the system just get the plain text.
/// Types `text` into the app in front as keyboard events carrying the characters themselves
/// (CGEventKeyboardSetUnicodeString, at most 20 UTF-16 units per event), whatever the keyboard
/// layout. False where this isn't available (then the text is pasted).
fn type_text(text: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::c_void;
        #[link(name = "ApplicationServices", kind = "framework")]
        unsafe extern "C" {
            fn CGEventCreateKeyboardEvent(source: *const c_void, keycode: u16, keydown: bool) -> *mut c_void;
            fn CGEventKeyboardSetUnicodeString(event: *mut c_void, len: usize, chars: *const u16);
            fn CGEventPost(tap: u32, event: *mut c_void);
            fn CFRelease(p: *const c_void);
        }
        let units: Vec<u16> = text.encode_utf16().collect();
        for chunk in units.chunks(20) {
            for down in [true, false] {
                let ev = unsafe { CGEventCreateKeyboardEvent(std::ptr::null(), 0, down) };
                if ev.is_null() {
                    return false;
                }
                unsafe {
                    CGEventKeyboardSetUnicodeString(ev, chunk.len(), chunk.as_ptr());
                    // kCGHIDEventTap: as if typed on the keyboard.
                    CGEventPost(0, ev);
                    CFRelease(ev);
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        true
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        false
    }
}

fn insert_in_app(bundle: &str, kind: Option<Docs>, text: &str, font: &str, script: Option<&str>, set_font: bool) -> Result<Option<String>, bool> {
    if !trusted_or_ask() {
        return Err(true);
    }
    let _ = osascript(&format!("tell application id \"{bundle}\" to activate"));
    std::thread::sleep(Duration::from_millis(300));
    wait_front(bundle);
    let doc = kind.and_then(|k| osascript(&front_doc_script(bundle, k)));
    // A glyph goes in at the cursor and never replaces text that is selected: when the app reports
    // a selection (its focused field's AXSelectedTextRange), the caret is first moved to the
    // selection's end with the Right arrow (key code 124).
    let mut before = ax_caret();
    if before.is_some_and(|c| c.len > 0) {
        let _ = osascript("tell application \"System Events\" to key code 124");
        std::thread::sleep(Duration::from_millis(80));
        before = ax_caret();
    }
    // The character is typed (a keyboard event carrying it), not pasted: pasting lets the app's
    // smart paste add a space before and after it; typing puts in exactly the character.
    if !type_text(text) {
        let (saved_rtf, saved_text) = pb_get();
        pb_set(None, text);
        keystroke(bundle, "v");
        std::thread::sleep(Duration::from_millis(300));
        pb_set(saved_rtf.as_deref(), &saved_text);
    }
    std::thread::sleep(Duration::from_millis(120));
    let n = text.chars().count().max(1);
    if set_font {
        // Select what was just typed: one Shift+Left (key code 123) per character.
        let _ = osascript(&format!(
            "tell application \"System Events\" to repeat {n} times\nkey code 123 using shift down\nend repeat"
        ));
        std::thread::sleep(Duration::from_millis(150));
        let _ = apply_in_app(bundle, kind, font, script);
        std::thread::sleep(Duration::from_millis(150));
    }
    // The cursor ends right after the glyph, with nothing added around it.
    settle_after_insert(before, n, set_font);
    Ok(doc)
}

/// Where the cursor is in the focused text of the app in front, from Accessibility: its start
/// (0-based), the selected length and the text's length. None when the app doesn't say (text
/// drawn on a canvas, as in Affinity).
#[derive(Clone, Copy, Debug)]
struct Caret {
    start: i64,
    len: i64,
    total: i64,
}

fn ax_caret() -> Option<Caret> {
    // System Events gives AXSelectedTextRange as {first, last} (1-based; last = first − 1 when
    // nothing is selected), the form RightFont's TextEdit script reads (string 0x10028d1ab).
    let out = osascript(
        "tell application \"System Events\" to tell (first process whose frontmost is true)\n\
         try\n\
         set f to value of attribute \"AXFocusedUIElement\"\n\
         set r to value of attribute \"AXSelectedTextRange\" of f\n\
         set n to value of attribute \"AXNumberOfCharacters\" of f\n\
         return ((item 1 of r) as text) & \"|\" & ((item 2 of r) as text) & \"|\" & (n as text)\n\
         end try\n\
         end tell",
    )?;
    let v: Vec<i64> = out.trim().split('|').filter_map(|x| x.trim().parse().ok()).collect();
    match v[..] {
        [first, last, total] => Some(Caret { start: first - 1, len: (last - first + 1).max(0), total }),
        _ => None,
    }
}

/// `count` characters of the focused text from `from` (0-based), from Accessibility.
fn ax_text(from: i64, count: i64) -> Option<String> {
    if from < 0 || count <= 0 {
        return None;
    }
    osascript(&format!(
        "tell application \"System Events\" to tell (first process whose frontmost is true)\n\
         try\n\
         set f to value of attribute \"AXFocusedUIElement\"\n\
         set v to value of attribute \"AXValue\" of f\n\
         return text {} thru {} of v\n\
         end try\n\
         end tell",
        from + 1,
        from + count
    ))
    .map(|s| s.trim_end_matches('\n').to_string())
}

/// After a glyph was typed (and its font set), the cursor goes right after the glyph and any space
/// the app added beside it is taken back out. With Accessibility the text is measured before and
/// after; without it, a selection left by the app's own script is closed with the Right arrow, and
/// after a paste (which already leaves the cursor after the glyph) no key is pressed.
fn settle_after_insert(before: Option<Caret>, n: usize, set_font: bool) {
    let key = |code: u8| {
        let _ = osascript(&format!("tell application \"System Events\" to key code {code}"));
        std::thread::sleep(Duration::from_millis(60));
    };
    let pasted = LAST_ROUTE.load(std::sync::atomic::Ordering::Relaxed) == 4;
    let (Some(b), Some(mut a)) = (before, ax_caret()) else {
        if set_font && !pasted {
            key(124);
        }
        return;
    };
    let n = n as i64;
    // Still selected: closed at its end.
    if a.len > 0 {
        key(124);
        match ax_caret() {
            Some(c) => a = c,
            None => return,
        }
    }
    let glyph_end = b.start + n;
    let extra = a.total - (b.total + n);
    let blank = |t: Option<String>| t.is_some_and(|t| !t.is_empty() && t.chars().all(char::is_whitespace));
    if extra > 0 {
        if extra >= 2 && blank(ax_text(b.start, 1)) && blank(ax_text(a.start - 1, 1)) {
            key(51);
            (0..n).for_each(|_| key(123));
            key(51);
            (0..n).for_each(|_| key(124));
        } else if blank(ax_text(glyph_end, extra)) {
            // Added after the glyph: the cursor is past them (backspace) or before them (forward).
            if a.start == glyph_end + extra {
                (0..extra).for_each(|_| key(51));
            } else if a.start == glyph_end {
                (0..extra).for_each(|_| key(117));
            }
        } else if blank(ax_text(b.start, extra)) && a.start == glyph_end + extra {
            // Added in front of it: past the glyph, out, and back after it.
            (0..n).for_each(|_| key(123));
            (0..extra).for_each(|_| key(51));
            (0..n).for_each(|_| key(124));
        }
        return;
    }
    // Nothing added, but the cursor isn't right after the glyph: moved back there.
    if extra == 0 && a.start > glyph_end {
        (0..(a.start - glyph_end).min(4)).for_each(|_| key(123));
    } else if extra == 0 && a.start < glyph_end && a.start >= b.start {
        (0..(glyph_end - a.start).min(4)).for_each(|_| key(124));
    }
}

impl App {
    /// Finds the apps once and starts the watcher.
    pub(crate) fn autoact_start(&mut self) {
        if self.autoact.started {
            return;
        }
        self.autoact.started = true;
        self.autoact.apps = find_installed();
        self.autoact_share_apps();
        // Fonts other apps ask for are answered from the catalog (macOS; not while taking test pictures).
        self.publish_font_names();
        crate::fontreq::set_substitutes(self.missing.replaced.clone());
        if self.screenshot.is_none() {
            crate::fontreq::start();
        }
        let watch = self.autoact.watch.clone();
        let shared = self.autoact.shared_apps.clone();
        let mut round = 0u32;
        std::thread::spawn(move || loop {
            let apps: Vec<(String, PathBuf, Option<Docs>, Vec<String>)> = shared.lock().unwrap().clone();
            let keys: Vec<(String, Vec<String>)> = apps.iter().map(|a| (a.0.clone(), a.3.clone())).collect();
            let running = running_now(&keys);
            let front = front_bundle();
            let want: Vec<String> = watch.lock().unwrap().want_docs.iter().cloned().collect();
            let mut docs = HashMap::new();
            for b in want {
                if !running.contains(&b) {
                    continue;
                }
                if let Some((_, _, Some(kind), _)) = apps.iter().find(|a| a.0 == b) {
                    if let Some(list) = osascript(&doc_list_script(&b, *kind)) {
                        docs.insert(b.clone(), list.split(", ").map(String::from).collect());
                    }
                } else if let Some(list) = window_documents(&b) {
                    // Any other app: the documents of its open windows (their file names).
                    docs.insert(b.clone(), list);
                }
            }
            {
                let mut w = watch.lock().unwrap();
                if let Some(f) = front {
                    if apps.iter().any(|a| a.0 == f) {
                        // An app coming to the front again is checked again.
                        if w.front.as_ref() != Some(&f) {
                            if let Ok(mut s) = stopped_checking().lock() {
                                s.remove(&f);
                            }
                        }
                        w.front = Some(f);
                    }
                }
                // Systems where the app in front can't be asked: the last supported app seen opening.
                if !cfg!(target_os = "macos") && w.front.as_ref().is_none_or(|f| !running.contains(f)) {
                    w.front = running.iter().find(|b| !w.running.contains(*b)).or(running.iter().next()).cloned();
                }
                if w.front.as_ref().is_some_and(|f| !running.contains(f)) {
                    w.front = None;
                }
                w.running = running;
                w.docs = docs;
            }
            // Every third round (about 4.5 s): the document open in front, for Missing Fonts.
            round += 1;
            if round % 3 == 0 {
                let front = watch.lock().unwrap().front.clone();
                let doc = front.and_then(|b| {
                    let procs = apps.iter().find(|a| a.0 == b).map(|a| a.3.clone()).unwrap_or_default();
                    front_document_of(&b, &procs).map(|p| (p, b))
                });
                watch.lock().unwrap().front_doc = doc;
            }
            std::thread::sleep(Duration::from_millis(1500));
        });
    }

    /// Whether a font file is on because FontCraft turned it on for an app (Auto Activation).
    pub(crate) fn auto_on_now(&self, path: &std::path::Path) -> bool {
        self.autoact.records.iter().any(|r| r.files.iter().any(|f| f == path))
    }

    /// Forgets that a font was turned on for an app (the person switched it off by hand).
    pub(crate) fn forget_auto(&mut self, path: &std::path::Path) {
        for r in self.autoact.records.iter_mut() {
            r.files.retain(|f| f != path);
        }
        self.autoact.records.retain(|r| !r.files.is_empty());
    }

    /// Gives the watcher the current list of apps (key, path, documents, process names).
    fn autoact_share_apps(&mut self) {
        *self.autoact.shared_apps.lock().unwrap() =
            self.autoact.apps.iter().map(|a| (a.def.bundle.to_string(), a.path.clone(), a.def.docs, a.procs.clone())).collect();
    }

    /// Looks for installed apps again (apps added or removed since FontCraft started).
    pub(crate) fn autoact_refresh(&mut self) {
        let apps = find_installed();
        let same = apps.len() == self.autoact.apps.len() && apps.iter().zip(&self.autoact.apps).all(|(a, b)| a.path == b.path);
        if !same {
            self.autoact.apps = apps;
            self.autoact_share_apps();
        }
    }

    /// The app FontCraft would work with now: the last one in front that is open and switched on.
    pub(crate) fn target_app(&self) -> Option<usize> {
        let w = self.autoact.watch.lock().unwrap();
        let front = w.front.clone()?;
        if !w.running.contains(&front) || self.settings.apps_off.contains(&front) {
            return None;
        }
        self.autoact.apps.iter().position(|a| a.def.bundle == front)
    }

    pub(crate) fn app_texture(&mut self, ctx: &egui::Context, i: usize) -> Option<TextureHandle> {
        let a = self.autoact.apps.get_mut(i)?;
        if a.tex.is_none() {
            let img = a.icon.take()?;
            a.tex = Some(ctx.load_texture(format!("app-{}", a.def.bundle), img, TextureOptions::LINEAR));
        }
        a.tex.clone()
    }

    /// The font files to turn on for the selected font (or its whole family, if that option is on).
    pub(crate) fn files_for(&self, face: usize) -> Vec<PathBuf> {
        let fam = self.lib.families.iter().find(|f| f.faces.contains(&face));
        let faces: Vec<usize> = match fam {
            Some(f) if self.settings.act_family => f.faces.clone(),
            _ => vec![face],
        };
        let mut v: Vec<PathBuf> = faces.iter().map(|&i| self.lib.files[self.lib.faces[i].file].path.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    /// Turns on the font's files that FontCraft can switch (library and Google fonts).
    pub(crate) fn auto_on(&mut self, files: &[PathBuf]) -> Vec<PathBuf> {
        let switchable: HashSet<PathBuf> = self
            .lib
            .files
            .iter()
            .filter(|f| matches!(f.source, crate::fonts::Source::Library | crate::fonts::Source::Google))
            .map(|f| f.path.clone())
            .collect();
        let mut turned = Vec::new();
        for p in files {
            if switchable.contains(p) && !self.active.contains(p) && crate::activate::set(p, true) {
                turned.push(p.clone());
            }
        }
        turned
    }

    pub(crate) fn face_names(&self, face: usize, ps: bool) -> String {
        let f = &self.lib.faces[face];
        if ps {
            if let Ok(tf) = ttf_parser::Face::parse(&self.lib.files[f.file].data, f.index) {
                for n in tf.names() {
                    if n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME {
                        if let Some(s) = n.to_string() {
                            return s;
                        }
                    }
                }
            }
        }
        f.family.clone()
    }

    /// The selected font, used in the target app's selected text.
    pub(crate) fn apply_to_app(&mut self, app: usize, face: usize) {
        let def = self.autoact.apps[app].def;
        let files = self.files_for(face);
        let turned = self.auto_on(&files);
        let font = self.face_names(face, def.ps_names);
        let pending = self.autoact.pending.clone();
        let name = def.name.to_string();
        // Apps that can't take a font from FontCraft (and every app outside macOS, where there are
        // no AppleScript or keystrokes to send): the font is turned on, and the person picks it there.
        // Windows: the selection's font is changed with copy, swap and paste (winauto.rs), with the
        // family name Windows apps use.
        #[cfg(target_os = "windows")]
        if def.how() != How::Activate && !def.win.is_empty() {
            let family = self.lib.faces[face].family.clone();
            let (exes, bundle) = (def.win, def.bundle.to_string());
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(400));
                let outcome = match crate::winauto::apply_font(exes, &family) {
                    Ok(true) => Outcome::Applied(name, None),
                    Ok(false) => Outcome::NoSelection(name),
                    Err(_) => Outcome::Activated(name, family),
                };
                pending.lock().unwrap().push((turned, bundle, outcome));
            });
            return;
        }
        if !cfg!(target_os = "macos") || def.how() == How::Activate {
            let family = self.lib.faces[face].family.clone();
            pending.lock().unwrap().push((turned, def.bundle.to_string(), Outcome::Activated(name, family)));
            return;
        }
        let (bundle, kind, script) = (def.bundle.to_string(), def.docs, def.script);
        let reload = def.reload_menu().filter(|_| !turned.is_empty());
        std::thread::spawn(move || {
            // Apps that read fonts only when they open but can read them again (VectorCraft,
            // DesignCraft) do so first, so the font just turned on is in their list.
            if let Some(names) = reload {
                std::thread::sleep(Duration::from_millis(300));
                reload_fonts_menu(&bundle, names);
                std::thread::sleep(Duration::from_millis(800));
            }
            // Give apps a moment to see a newly turned-on font.
            std::thread::sleep(Duration::from_millis(250));
            let outcome = match apply_in_app(&bundle, kind, &font, script) {
                Ok(doc) => Outcome::Applied(name, doc),
                Err(true) => Outcome::NeedsAccess,
                Err(false) => Outcome::NoSelection(name),
            };
            pending.lock().unwrap().push((turned, bundle, outcome));
        });
    }

    /// A glyph from the Glyphs page, inserted into the target app's document.
    pub(crate) fn insert_glyph(&mut self, app: usize, face: usize, gid: u16) {
        let lang = self.settings.lang;
        let cp = self.face_data.as_ref().and_then(|d| d.cmap.get(&gid).copied());
        let Some(ch) = cp.and_then(char::from_u32) else {
            self.notice = Some(i18n::tr(lang, K::GlyphNoChar).to_string());
            return;
        };
        self.insert_char(app, face, ch);
    }

    /// A character, in the font of `face`, put into the target app's document.
    pub(crate) fn insert_char(&mut self, app: usize, face: usize, ch: char) {
        let lang = self.settings.lang;
        let def = self.autoact.apps[app].def;
        let files = self.files_for(face);
        let turned = self.auto_on(&files);
        let font = self.face_names(face, def.ps_names);
        let pending = self.autoact.pending.clone();
        let name = def.name.to_string();
        // Windows and Linux (X11): the glyph is typed into the app's window (winauto.rs), and on
        // Windows its font set with copy, swap and paste; when the app's window isn't found (or on
        // Wayland), it goes on the clipboard, ready to paste.
        #[cfg(any(target_os = "windows", all(target_os = "linux", not(target_os = "android"))))]
        {
            let exes: &'static [&'static str] = if cfg!(target_os = "windows") { def.win } else { def.linux };
            if !exes.is_empty() {
                let family = self.lib.faces[face].family.clone();
                let bundle = def.bundle.to_string();
                let set_font = def.how() != How::Activate;
                let text = ch.to_string();
                self.clip = Some(text.clone());
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(250));
                    let typed = crate::winauto::insert(exes, &text);
                    #[cfg(target_os = "windows")]
                    if typed && set_font {
                        std::thread::sleep(Duration::from_millis(150));
                        crate::winauto::select_back(text.encode_utf16().count());
                        let _ = crate::winauto::apply_font(exes, &family);
                        crate::winauto::caret_right();
                    }
                    let _ = (set_font, &family);
                    let outcome = if typed { Outcome::Inserted(name) } else { Outcome::Activated(name, String::new()) };
                    pending.lock().unwrap().push((turned, bundle, outcome));
                });
                return;
            }
        }
        // Elsewhere: the character goes on the clipboard, ready to paste in the app.
        if !cfg!(target_os = "macos") {
            self.clip = Some(ch.to_string());
            self.notice = Some(i18n::tr(lang, K::CopiedForApp).replace("{c}", &ch.to_string()).replace("{app}", &name));
            pending.lock().unwrap().push((turned, def.bundle.to_string(), Outcome::Activated(name, String::new())));
            return;
        }
        let (bundle, kind, script) = (def.bundle.to_string(), def.docs, def.script);
        let set_font = def.how() != How::Activate;
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(250));
            let outcome = match insert_in_app(&bundle, kind, &ch.to_string(), &font, script, set_font) {
                Ok(_) => Outcome::Inserted(name),
                Err(true) => Outcome::NeedsAccess,
                Err(false) => Outcome::NoSelection(name),
            };
            pending.lock().unwrap().push((turned, bundle, outcome));
        });
    }

    /// Each frame: report finished work, and turn fonts off when their document or app has closed.
    pub(crate) fn autoact_tick(&mut self, ctx: &egui::Context) {
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        let done: Vec<(Vec<PathBuf>, String, Outcome)> = std::mem::take(&mut *self.autoact.pending.lock().unwrap());
        for (files, bundle, outcome) in done {
            let mut doc = match &outcome {
                Outcome::Applied(_, d) => d.clone(),
                _ => None,
            };
            // Apps without document scripting: the document open in front in that app, by its file
            // name (macOS AXDocument), so the font goes off when that document closes.
            if doc.is_none() && !files.is_empty() && cfg!(target_os = "macos") {
                let front = self.autoact.front_document().filter(|(_, b)| b == &bundle).map(|(p, _)| p);
                let front = front.or_else(|| front_document_of(&bundle, &[]));
                doc = front.and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));
            }
            if !files.is_empty() {
                if doc.is_some() {
                    self.autoact.watch.lock().unwrap().want_docs.insert(bundle.clone());
                }
                self.autoact.records.push(Record { files, bundle, doc });
            }
            let silent = self.settings.silent;
            self.notice = match outcome {
                Outcome::Applied(app, _) if !silent => Some(t(K::AppliedIn).replace("{app}", &app)),
                Outcome::Activated(app, font) => Some(t(K::ActivatedFor).replace("{app}", &app).replace("{font}", &font)),
                Outcome::Inserted(app) if !silent => Some(t(K::InsertedIn).replace("{app}", &app)),
                Outcome::NoSelection(app) => Some(t(K::SelectTextFirst).replace("{app}", &app)),
                Outcome::Refreshed(app) if !silent => Some(t(K::FontListRefreshed).replace("{app}", &app)),
                Outcome::Reopen(app) => Some(t(K::ReopenToSee).replace("{apps}", &app)),
                Outcome::NeedsAccess => {
                    // macOS has already shown its own request (trusted_or_ask); FontCraft only says
                    // why nothing happened, and opens no settings by itself.
                    Some(t(K::NeedsAccess).to_string())
                }
                _ => None,
            };
        }
        // Fonts other apps asked the system for (macOS): turned on for them, or missing.
        for a in crate::fontreq::take_answers() {
            match a {
                crate::fontreq::Answer::Activated(files, bundle, app, font) => {
                    if !files.is_empty() {
                        self.autoact.records.push(Record { files, bundle, doc: None });
                    }
                    if !self.settings.silent {
                        self.notice = Some(t(K::ActivatedFor).replace("{app}", &app).replace("{font}", &font));
                    }
                }
                crate::fontreq::Answer::Missing(font, app) => self.add_missing(font, app),
            }
        }
        if !self.autoact.records.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(1500));
        }
        // Turn off fonts whose app quit or whose document closed.
        let (running, docs) = {
            let w = self.autoact.watch.lock().unwrap();
            (w.running.clone(), w.docs.clone())
        };
        let watched: HashSet<&str> = self.autoact.apps.iter().map(|a| a.def.bundle).collect();
        let mut keep = Vec::new();
        let mut off: Vec<PathBuf> = Vec::new();
        for r in std::mem::take(&mut self.autoact.records) {
            // Apps outside FontCraft's list (fonts they asked the system for) are checked directly.
            let app_gone = if watched.contains(r.bundle.as_str()) { !running.contains(&r.bundle) } else { !app_running(&r.bundle) };
            let doc_gone = match (&r.doc, docs.get(&r.bundle)) {
                (Some(d), Some(list)) => !list.contains(d),
                _ => false,
            };
            if app_gone || doc_gone {
                off.extend(r.files);
            } else {
                keep.push(r);
            }
        }
        let still_needed: HashSet<PathBuf> = keep.iter().flat_map(|r| r.files.iter().cloned()).collect();
        for p in off {
            if !still_needed.contains(&p) && !self.active.contains(&p) {
                crate::activate::set(&p, false);
            }
        }
        self.autoact.records = keep;
        self.tell_apps_fonts_on();
    }

    /// Fonts turned on since last frame (by hand, from a menu or for an app): open apps that read
    /// fonts only when they start are asked to read them again with their own menu command, or the
    /// person is told to reopen them.
    fn tell_apps_fonts_on(&mut self) {
        let Some(seen) = self.autoact.seen_on.as_mut() else {
            let mut now: HashSet<PathBuf> = self.active.iter().cloned().collect();
            now.extend(self.autoact.records.iter().flat_map(|r| r.files.iter().cloned()));
            self.autoact.seen_on = Some(now);
            return;
        };
        let mut fresh = false;
        for p in self.active.iter().chain(self.autoact.records.iter().flat_map(|r| r.files.iter())) {
            if seen.insert(p.clone()) {
                fresh = true;
            }
        }
        // Fonts turned off are forgotten, so turning them on again counts again.
        let on: HashSet<&PathBuf> = self.active.iter().chain(self.autoact.records.iter().flat_map(|r| r.files.iter())).collect();
        seen.retain(|p| on.contains(p));
        if !fresh {
            return;
        }
        let running = self.autoact.watch.lock().unwrap().running.clone();
        let mut reopen: Vec<&'static str> = Vec::new();
        for a in &self.autoact.apps {
            let def = a.def;
            if !running.contains(def.bundle) || self.settings.apps_off.iter().any(|b| b == def.bundle) || reopen.contains(&def.name) {
                continue;
            }
            if let Some(names) = def.reload_menu() {
                let (bundle, name, pending) = (def.bundle.to_string(), def.name.to_string(), self.autoact.pending.clone());
                std::thread::spawn(move || {
                    // A moment for the system to finish registering the font.
                    std::thread::sleep(Duration::from_millis(300));
                    let outcome = if !trusted() {
                        Outcome::Reopen(name)
                    } else if reload_fonts_menu(&bundle, names) {
                        Outcome::Refreshed(name)
                    } else {
                        Outcome::Reopen(name)
                    };
                    pending.lock().unwrap().push((Vec::new(), bundle, outcome));
                });
            } else if def.needs_reopen() {
                reopen.push(def.name);
            }
        }
        if !reopen.is_empty() {
            let lang = self.settings.lang;
            let msg = i18n::tr(lang, K::ReopenToSee).replace("{apps}", &reopen.join(", "));
            self.notice = Some(match self.notice.take() {
                Some(n) if !n.is_empty() => format!("{n} {msg}"),
                _ => msg,
            });
        }
    }

    /// When FontCraft quits: turn off auto- and/or manually activated fonts, as the options say.
    pub(crate) fn autoact_quit(&mut self) {
        // Fonts activated temporarily always go off when FontCraft quits (RightFont's
        // deactivateTemporarilyActivatedFonts, 0x100072ea0), unless also switched on for good.
        for p in std::mem::take(&mut self.temp_active) {
            if !self.active.contains(&p) {
                crate::activate::set(&p, false);
            }
        }
        // Nothing FontCraft turned on stays on in the system after it quits (the person's rule):
        // fonts turned on for apps and documents go off, and so do the ones kept on by hand; those
        // stay in FontCraft's list (unless "turn off when FontCraft quits" clears it) and are turned
        // on again the next time FontCraft opens.
        for r in std::mem::take(&mut self.autoact.records) {
            for p in r.files {
                crate::activate::set(&p, false);
            }
        }
        if self.settings.deact_manual_on_quit {
            for p in std::mem::take(&mut self.active) {
                crate::activate::set(&p, false);
            }
            self.save_active();
        } else {
            for p in &self.active {
                crate::activate::set(p, false);
            }
        }
    }

    /// The toolbar button with the target app's icon: click to use the selected font there;
    /// drop a glyph on it to insert the glyph.
    pub(crate) fn target_button(&mut self, ui: &mut egui::Ui) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let Some(app) = self.target_app() else {
            self.autoact.target_rect = None;
            return;
        };
        let face = match self.selected {
            Some(crate::Row::Family(f)) => Some(self.lib.families[f].default_face),
            Some(crate::Row::Face(_, face)) => Some(face),
            None => None,
        };
        let tex = self.app_texture(ui.ctx(), app);
        let (r, resp) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
        self.autoact.target_rect = Some(r);
        let dropping = self.autoact.dragging.is_some() && ui.rect_contains_pointer(r);
        if resp.hovered() || dropping {
            ui.painter().rect_filled(r, 6.0, if dropping { pal.accent_soft } else { pal.hover });
        }
        draw_app_icon(ui.painter(), Rect::from_center_size(r.center(), Vec2::splat(ICON_PT)), tex.as_ref(), self.autoact.apps[app].def.name, pal.accent);
        let name = self.autoact.apps[app].def.name;
        let tip = i18n::tr(lang, K::UseIn).replace("{app}", name);
        if resp.on_hover_text(tip).clicked() {
            if let Some(face) = face {
                self.apply_to_app(app, face);
            }
        }
    }

    /// Auto Activation settings page: the installed apps in a scrolling box on the left (each with
    /// its icon and a check box), the options on the right.
    pub(crate) fn autoact_settings(&mut self, ui: &mut egui::Ui) {
        let pal = self.pal;
        let lang = self.settings.lang;
        let t = move |k: K| i18n::tr(lang, k);
        self.autoact_start();
        // Opening this page looks for apps again (installed or removed since FontCraft started).
        if !self.autoact.settings_seen {
            self.autoact.settings_seen = true;
            self.autoact_refresh();
        }
        ui.add(egui::Label::new(egui::RichText::new(t(K::AutoActHint)).size(12.5).color(pal.muted)).wrap());
        ui.add_space(6.0);
        if cfg!(target_os = "macos") && !trusted() {
            egui::Frame::NONE.fill(pal.card).corner_radius(8).inner_margin(Margin::same(8)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(t(K::NeedsAccess)).size(12.5).color(pal.warn));
                    if ui.button(t(K::OpenAccess)).clicked() {
                        crate::fcl::open_security(crate::fcl::Pane::Accessibility);
                    }
                });
            });
            ui.add_space(6.0);
        }
        let running = self.autoact.watch.lock().unwrap().running.clone();
        let total_w = ui.available_width();
        let list_w = (total_w * 0.52).max(280.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(list_w);
                ui.label(egui::RichText::new(t(K::InstalledApps)).strong().color(pal.ink));
                ui.add_space(4.0);
                egui::Frame::NONE
                    .fill(pal.card)
                    .stroke(Stroke::new(1.0_f32, pal.line))
                    .corner_radius(8)
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().id_salt("autoact-apps").max_height(300.0).min_scrolled_height(300.0).auto_shrink([false, false]).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            if self.autoact.apps.is_empty() {
                                ui.label(egui::RichText::new(t(K::NoAppsFound)).size(12.5).color(pal.muted));
                            }
                            for i in 0..self.autoact.apps.len() {
                                let tex = self.app_texture(ui.ctx(), i);
                                let bundle = self.autoact.apps[i].def.bundle.to_string();
                                let name = self.autoact.apps[i].def.name;
                                let how = if cfg!(target_os = "macos") { self.autoact.apps[i].def.how() } else { How::Activate };
                                ui.horizontal(|ui| {
                                    let mut on = !self.settings.apps_off.contains(&bundle);
                                    if crate::oslook::check(ui, &mut on, "").changed() {
                                        if on {
                                            self.settings.apps_off.retain(|b| b != &bundle);
                                        } else {
                                            self.settings.apps_off.push(bundle.clone());
                                        }
                                    }
                                    paint_app_icon(ui, tex.as_ref(), name, ICON_PT, pal.accent);
                                    ui.label(egui::RichText::new(name).color(pal.ink));
                                    if let Some(v) = &self.autoact.apps[i].version {
                                        ui.label(egui::RichText::new(v).size(11.5).color(pal.muted));
                                    }
                                    if how == How::Activate {
                                        ui.label(egui::RichText::new(t(K::OnlyActivates)).size(11.5).color(pal.muted))
                                            .on_hover_text(t(K::OnlyActivatesTip));
                                    }
                                    if running.contains(&bundle) {
                                        let (d, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                                        ui.painter().circle_filled(d.center(), 3.5, crate::libs::LIME);
                                    }
                                });
                            }
                        });
                    });
            });
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(t(K::ActivationOptions)).strong().color(pal.ink));
                crate::oslook::check(ui, &mut self.settings.act_family, t(K::ActivateFamily));
                crate::oslook::check(ui, &mut self.settings.silent, t(K::SilentMode));
                ui.add_space(10.0);
                ui.label(egui::RichText::new(t(K::DeactivationOptions)).strong().color(pal.ink));
                ui.label(egui::RichText::new(t(K::WhenQuits)).size(12.5).color(pal.muted));
                // Always: nothing FontCraft turned on for apps stays on after it quits.
                ui.add_enabled_ui(false, |ui| {
                    let mut always = true;
                    crate::oslook::check(ui, &mut always, t(K::AutoActivatedFonts));
                });
                crate::oslook::check(ui, &mut self.settings.deact_manual_on_quit, t(K::ManualFonts));
            });
        });
    }
}

/// App icons are shown between 12 and 24 points; FontCraft uses 24 (the picture is read at 64 px,
/// so it stays sharp on high-resolution screens and keeps its proportions).
pub const ICON_PT: f32 = 24.0;

/// An app's icon in a square of `size` points (12–24), never stretched; its initial on a disc if it has none.
pub fn paint_app_icon(ui: &mut egui::Ui, tex: Option<&TextureHandle>, name: &str, size: f32, accent: Color32) -> Rect {
    let size = size.clamp(12.0, 24.0);
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    draw_app_icon(ui.painter(), r, tex, name, accent);
    r
}

fn app_badge_style(name: &str, accent: Color32) -> (Color32, &'static str, Color32) {
    let n = name.to_ascii_lowercase();
    if n.contains("photoshop") {
        (Color32::from_rgb(0, 30, 54), "Ps", Color32::from_rgb(49, 168, 255))
    } else if n.contains("illustrator") {
        (Color32::from_rgb(51, 0, 0), "Ai", Color32::from_rgb(255, 154, 0))
    } else if n.contains("indesign") {
        (Color32::from_rgb(73, 2, 31), "Id", Color32::from_rgb(255, 51, 102))
    } else if n.contains("incopy") {
        (Color32::from_rgb(50, 10, 40), "Ic", Color32::from_rgb(255, 100, 150))
    } else if n.contains("after effects") {
        (Color32::from_rgb(0, 0, 90), "Ae", Color32::from_rgb(153, 153, 255))
    } else if n.contains("xd") {
        (Color32::from_rgb(71, 1, 55), "Xd", Color32::from_rgb(255, 97, 246))
    } else if n.contains("express") {
        (Color32::from_rgb(38, 0, 38), "Ex", Color32::from_rgb(204, 41, 250))
    } else if n.contains("word") {
        (Color32::from_rgb(24, 90, 189), "W", Color32::WHITE)
    } else if n.contains("powerpoint") {
        (Color32::from_rgb(196, 62, 28), "P", Color32::WHITE)
    } else if n.contains("excel") {
        (Color32::from_rgb(16, 124, 65), "X", Color32::WHITE)
    } else if n.contains("publisher") && !n.contains("affinity") {
        (Color32::from_rgb(7, 117, 104), "Pub", Color32::WHITE)
    } else if n.contains("designer") && n.contains("microsoft") {
        (Color32::from_rgb(80, 55, 149), "Des", Color32::WHITE)
    } else if n.contains("visio") {
        (Color32::from_rgb(57, 85, 163), "V", Color32::WHITE)
    } else if n.contains("coreldraw") {
        (Color32::from_rgb(0, 132, 61), "CD", Color32::WHITE)
    } else if n.contains("photo-paint") || n.contains("photopaint") {
        (Color32::from_rgb(183, 18, 52), "PP", Color32::WHITE)
    } else if n.contains("figma") {
        (Color32::from_rgb(10, 207, 131), "Fg", Color32::WHITE)
    } else if n.contains("canva") {
        (Color32::from_rgb(0, 196, 204), "Cv", Color32::WHITE)
    } else if n.contains("sketch") {
        (Color32::from_rgb(253, 179, 0), "Sk", Color32::BLACK)
    } else if n.contains("affinity designer") {
        (Color32::from_rgb(27, 114, 186), "Ad", Color32::WHITE)
    } else if n.contains("affinity photo") {
        (Color32::from_rgb(127, 63, 152), "Ap", Color32::WHITE)
    } else if n.contains("affinity publisher") {
        (Color32::from_rgb(212, 32, 54), "Pub", Color32::WHITE)
    } else if n.contains("affinity") {
        (Color32::from_rgb(33, 150, 243), "Aff", Color32::WHITE)
    } else if n.contains("vectorcraft") {
        (Color32::from_rgb(108, 92, 231), "VC", Color32::WHITE)
    } else if n.contains("designcraft") {
        (Color32::from_rgb(9, 132, 227), "DC", Color32::WHITE)
    } else if n.contains("photocraft") {
        (Color32::from_rgb(0, 184, 148), "PC", Color32::WHITE)
    } else if n.contains("anicraft") {
        (Color32::from_rgb(232, 67, 147), "AC", Color32::WHITE)
    } else if n.contains("gimp") {
        (Color32::from_rgb(114, 92, 71), "Gi", Color32::WHITE)
    } else if n.contains("inkscape") {
        (Color32::from_rgb(30, 30, 30), "Ink", Color32::WHITE)
    } else if n.contains("krita") {
        (Color32::from_rgb(51, 117, 255), "Kr", Color32::WHITE)
    } else if n.contains("scribus") {
        (Color32::from_rgb(31, 107, 153), "Sc", Color32::WHITE)
    } else if n.contains("boxy") {
        (Color32::from_rgb(16, 163, 127), "Bx", Color32::WHITE)
    } else if n.contains("libreoffice") {
        (Color32::from_rgb(24, 163, 3), "LO", Color32::WHITE)
    } else if n.contains("onlyoffice") {
        (Color32::from_rgb(255, 111, 61), "OO", Color32::WHITE)
    } else if n.contains("pages") {
        (Color32::from_rgb(255, 149, 0), "Pg", Color32::WHITE)
    } else if n.contains("keynote") {
        (Color32::from_rgb(0, 122, 255), "Kn", Color32::WHITE)
    } else if n.contains("numbers") {
        (Color32::from_rgb(52, 199, 89), "Nm", Color32::WHITE)
    } else if n.contains("pixelmator") {
        (Color32::from_rgb(255, 59, 48), "Px", Color32::WHITE)
    } else {
        (accent, "App", Color32::WHITE)
    }
}

fn draw_app_icon(p: &egui::Painter, r: Rect, tex: Option<&TextureHandle>, name: &str, accent: Color32) {
    match tex {
        Some(t) => {
            let [w, h] = t.size();
            let k = (r.width() / w as f32).min(r.height() / h as f32);
            let fit = Rect::from_center_size(r.center(), Vec2::new(w as f32 * k, h as f32 * k));
            p.image(t.id(), fit, Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            let (bg, label, text_color) = app_badge_style(name, accent);
            let tile = r.shrink(1.0);
            p.rect_filled(tile, 4.0, bg);
            p.rect_stroke(tile, 4.0, Stroke::new(1.0, Color32::from_black_alpha(40)), egui::StrokeKind::Inside);
            p.text(tile.center(), Align2::CENTER_CENTER, label, egui::FontId::proportional(tile.width() * 0.42), text_color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swaps_every_font_and_keeps_formatting() {
        let rtf = "{\\rtf1\\ansi{\\fonttbl\\f0\\fswiss\\fcharset0 Helvetica;\\f1\\fnil Times-Bold;}{\\colortbl;\\red255\\green0\\blue0;}\\f0\\fs48 \\cf1 Hola {\\f1 mundo}}";
        let out = retarget_rtf(rtf, "Lobster-Regular").unwrap();
        assert!(!out.contains("Helvetica") && !out.contains("Times-Bold"));
        assert!(out.contains("Lobster-Regular"));
        assert!(out.contains("\\fs48") && out.contains("\\cf1") && out.contains("\\colortbl"));
        assert!(out.contains("{\\f0\\fnil\\fcharset0 Lobster-Regular;}") && out.contains("{\\f1\\fnil\\fcharset0 Lobster-Regular;}"));
    }

    #[test]
    fn grouped_font_table_entries() {
        let rtf = "{\\rtf1{\\fonttbl{\\f0\\fnil Arial;}{\\f2\\froman{\\*\\panose 02020603050405020304}Times;}}\\f2 x}";
        let out = retarget_rtf(rtf, "Inter").unwrap();
        assert!(out.contains("{\\f0\\fnil\\fcharset0 Inter;}") && out.contains("{\\f2\\fnil\\fcharset0 Inter;}"));
        assert!(out.ends_with("\\f2 x}"));
    }

    #[test]
    fn plain_text_and_non_ascii() {
        let out = rtf_for("Ñandú {1}\n€", "Lobster");
        assert!(out.contains("\\u209?") && out.contains("\\{1\\}") && out.contains("\\par") && out.contains("\\u8364?"));
    }
}

/// Developer aid (`--test-clipboard`): puts rich text on the clipboard, reads it back, restores what was there.
pub fn clipboard_selftest() -> bool {
    let (old_rtf, old_text) = pb_get();
    let rtf = rtf_for("Prueba", "Lobster-Regular");
    pb_set(Some(&rtf), "Prueba");
    let (back_rtf, back_text) = pb_get();
    let ok = back_rtf.as_deref().is_some_and(|r| r.contains("Lobster-Regular")) && back_text == "Prueba";
    let swapped = back_rtf.as_deref().and_then(|r| retarget_rtf(r, "Inter")).is_some_and(|r| r.contains("Inter") && !r.contains("Lobster"));
    pb_set(old_rtf.as_deref(), &old_text);
    println!("rich text round trip: {ok}, font swapped: {swapped}");
    ok && swapped
}
