//! The host system's own menus for FontCraft's right-click menus: macOS NSMenu and Windows' Win32
//! pop-up menu, through muda (the menu library the menu bar icon uses). They look, behave and draw
//! their arrows, separators and colours exactly as the system does; the items carry FontCraft's own
//! icons (the SVGs in assets/icons/ui, the same as the sidebar's), line icons as template images
//! that the system tints like its own, the activation asterisk in its colour.
//! The chosen item comes back as a menu event; `take` hands it to the menu that asked.
//! Linux (whose native menus need a GTK window FontCraft doesn't have) and Android keep
//! FontCraft's own menus.

/// One entry of a native menu (built everywhere; shown only where the system has native menus).
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub enum NItem {
    Item { id: String, label: String, enabled: bool, icon: Option<MIcon> },
    Sep,
    Sub { label: String, icon: Option<MIcon>, items: Vec<NItem> },
}

/// One of FontCraft's icons (uiicons) for a menu item: a line icon the system tints (template), or
/// one kept in its colour.
#[derive(Clone, Copy)]
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
pub struct MIcon {
    pub name: &'static str,
    pub color: Option<[u8; 3]>,
}

impl MIcon {
    /// A line icon, tinted by the system like its own menu images.
    pub const fn line(name: &'static str) -> Self {
        MIcon { name, color: None }
    }
    /// An icon kept in its own colour (the activation asterisk).
    pub const fn colored(name: &'static str, rgb: [u8; 3]) -> Self {
        MIcon { name, color: Some(rgb) }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod imp {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use tray_icon::menu::{ContextMenu, Icon, IconMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

    thread_local! {
        /// The menu on screen (kept alive until it is answered) and its owner.
        static OPEN: RefCell<Option<(String, Menu)>> = const { RefCell::new(None) };
        /// When the last menu opened.
        static OPENED: std::cell::Cell<Option<std::time::Instant>> = const { std::cell::Cell::new(None) };
        /// Owner → chosen item id.
        static CHOSEN: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    }

    /// The window the menus open over: macOS its NSView, Windows its HWND.
    static WINDOW: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    pub fn set_window(handle: usize) {
        WINDOW.store(handle, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn available() -> bool {
        WINDOW.load(std::sync::atomic::Ordering::Relaxed) != 0
    }

    /// Menu icon size: macOS menu images are 16 points.
    const PX: u32 = 16;

    /// The icon's picture, drawn from its SVG once and kept (menus open many times).
    fn image(icon: MIcon) -> Option<Icon> {
        use std::collections::HashMap;
        use std::sync::Mutex;
        static CACHE: Mutex<Option<HashMap<(&'static str, [u8; 3]), Vec<u8>>>> = Mutex::new(None);
        let key = (icon.name, icon.color.unwrap_or([0, 0, 0]));
        let mut cache = CACHE.lock().ok()?;
        let map = cache.get_or_insert_with(HashMap::new);
        if !map.contains_key(&key) {
            map.insert(key, crate::uiicons::rgba(key.0, key.1, PX)?);
        }
        Icon::from_rgba(map.get(&key)?.clone(), PX, PX).ok()
    }

    fn build(owner: &str, items: Vec<NItem>, add: &dyn Fn(&dyn IsMenuItem)) {
        for it in items {
            match it {
                NItem::Item { id, label, enabled, icon } => {
                    let mid = format!("{owner}\u{1f}{id}");
                    match icon {
                        Some(ic) => {
                            let item = IconMenuItem::with_id(mid, label, enabled, None, None);
                            set_item_icon(&item, ic);
                            add(&item);
                        }
                        None => add(&MenuItem::with_id(mid, label, enabled, None)),
                    }
                }
                NItem::Sep => add(&PredefinedMenuItem::separator()),
                NItem::Sub { label, icon, items } => {
                    let sub = Submenu::new(label, true);
                    if let Some(ic) = icon {
                        #[cfg(target_os = "macos")]
                        if ic.color.is_none() {
                            sub.set_icon_templated(image(ic));
                        } else {
                            sub.set_icon(image(ic));
                        }
                        #[cfg(not(target_os = "macos"))]
                        sub.set_icon(image(ic));
                    }
                    build(owner, items, &|i| {
                        let _ = sub.append(i);
                    });
                    add(&sub);
                }
            }
        }
    }

    fn set_item_icon(item: &IconMenuItem, ic: MIcon) {
        #[cfg(target_os = "macos")]
        if ic.color.is_none() {
            item.set_icon_templated(image(ic));
            return;
        }
        item.set_icon(image(ic));
    }

    /// Builds the menu without showing it, for timing (FONTCRAFT_MENU_BENCH).
    pub fn build_only(owner: &str, items: Vec<NItem>) {
        let menu = Menu::new();
        build(owner, items, &|i| {
            let _ = menu.append(i);
        });
    }

    pub fn popup(owner: &str, items: Vec<NItem>) -> bool {
        let handle = WINDOW.load(std::sync::atomic::Ordering::Relaxed);
        if handle == 0 {
            return false;
        }
        let menu = Menu::new();
        build(owner, items, &|i| {
            let _ = menu.append(i);
        });
        // The menu opens at the pointer, like any context menu of the system.
        unsafe {
            #[cfg(target_os = "macos")]
            menu.show_context_menu_for_nsview(handle as *const std::ffi::c_void, None);
            #[cfg(target_os = "windows")]
            {
                menu.attach_menu_subclass_for_hwnd(handle as isize);
                menu.show_context_menu_for_hwnd(handle as isize, None);
            }
        }
        OPEN.with(|o| *o.borrow_mut() = Some((owner.to_string(), menu)));
        OPENED.with(|t| t.set(Some(std::time::Instant::now())));
        true
    }

    pub fn take(owner: &str) -> Option<String> {
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if let Some((o, id)) = ev.id.0.split_once('\u{1f}') {
                CHOSEN.with(|c| c.borrow_mut().insert(o.to_string(), id.to_string()));
                // Answered: the menu can go (Windows: its window hook with it).
                OPEN.with(|m| {
                    #[cfg(target_os = "windows")]
                    if let Some((_, menu)) = m.borrow().as_ref() {
                        unsafe { menu.detach_menu_subclass_from_hwnd(WINDOW.load(std::sync::atomic::Ordering::Relaxed) as isize) };
                    }
                    *m.borrow_mut() = None;
                });
            }
        }
        CHOSEN.with(|c| c.borrow_mut().remove(owner))
    }

    pub fn waiting() -> bool {
        // Only for a moment: a menu closed without a choice sends nothing.
        OPEN.with(|m| m.borrow().is_some()) && OPENED.with(|t| t.get()).is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(1))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp {
    use super::*;
    pub fn set_window(_handle: usize) {}
    pub fn available() -> bool {
        false
    }
    pub fn popup(_owner: &str, _items: Vec<NItem>) -> bool {
        false
    }
    pub fn take(_owner: &str) -> Option<String> {
        None
    }
    pub fn waiting() -> bool {
        false
    }
}

/// Remembers the window native menus open over (macOS NSView, Windows HWND).
pub fn set_window(handle: usize) {
    imp::set_window(handle)
}

/// Whether right-click menus are the system's own here.
pub fn available() -> bool {
    imp::available()
}

/// Shows a native menu at the pointer for `owner`; false where the system has none to offer
/// (then FontCraft's own menu is used).
pub fn popup(owner: &str, items: Vec<NItem>) -> bool {
    imp::popup(owner, items)
}

/// The item chosen in `owner`'s native menu, once.
pub fn take(owner: &str) -> Option<String> {
    imp::take(owner)
}

/// Builds a native menu without showing it (timing in test runs).
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub fn build_only(owner: &str, items: Vec<NItem>) {
    imp::build_only(owner, items)
}

/// Whether a native menu is still waiting for its answer (Windows hands it over a moment after
/// the menu closes; macOS before `popup` returns).
pub fn waiting() -> bool {
    imp::waiting()
}
