//! Dragging a glyph out of FontCraft into another app (macOS): a real system drag session that
//! carries the character as plain text and as rich text in its font, so dropping it into a text
//! document puts the character there in that font. Anything AppKit refuses is caught (no crash).

#[cfg(target_os = "macos")]
mod mac {
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::{AnyObject, NSObject};
    use objc2::{class, define_class, msg_send, AllocAnyThread};
    use objc2_foundation::{NSData, NSPoint, NSRect, NSSize, NSString};

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; no Drop.
        #[unsafe(super(NSObject))]
        #[name = "FontCraftGlyphDragSource"]
        struct DragSource;

        impl DragSource {
            /// NSDraggingSource: the glyph is copied into the other app.
            #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
            fn operation_mask(&self, _session: *mut AnyObject, _context: isize) -> usize {
                1 // NSDragOperationCopy
            }
        }
    );

    /// `[[class alloc] init]`.
    unsafe fn new_object(cls: &objc2::runtime::AnyClass) -> Retained<AnyObject> {
        let a: Allocated<AnyObject> = unsafe { msg_send![cls, alloc] };
        unsafe { msg_send![a, init] }
    }

    /// Starts the drag from `view` (the window's content view) at `pos` (points from its top-left).
    pub fn start(view: *mut AnyObject, text: &str, rtf: Option<&str>, png: Option<Vec<u8>>, file: Option<&std::path::Path>, pos: (f64, f64)) -> bool {
        if view.is_null() {
            return false;
        }
        let text = text.to_string();
        let rtf = rtf.map(String::from);
        let file_url = file.map(|f| format!("file://{}", f.to_string_lossy().replace(' ', "%20")));
        let run = move || unsafe {
            let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
            let event: *mut AnyObject = msg_send![app, currentEvent];
            if event.is_null() {
                return false;
            }
            let item: Retained<AnyObject> = new_object(class!(NSPasteboardItem));
            let plain_type = NSString::from_str("public.utf8-plain-text");
            let _: bool = msg_send![&item, setString: &*NSString::from_str(&text), forType: &*plain_type];
            if let Some(r) = &rtf {
                let rtf_type = NSString::from_str("public.rtf");
                let _: bool = msg_send![&item, setData: &*NSData::with_bytes(r.as_bytes()), forType: &*rtf_type];
            }
            if let Some(b) = &png {
                let png_type = NSString::from_str("public.png");
                let _: bool = msg_send![&item, setData: &*NSData::with_bytes(b), forType: &*png_type];
            }
            if let Some(u) = &file_url {
                let url_type = NSString::from_str("public.file-url");
                let _: bool = msg_send![&item, setString: &*NSString::from_str(u), forType: &*url_type];
            }
            let alloc_drag: Allocated<AnyObject> = msg_send![class!(NSDraggingItem), alloc];
            let dragging: Retained<AnyObject> = msg_send![alloc_drag, initWithPasteboardWriter: &*item];
            let image: Option<Retained<AnyObject>> = png.as_ref().and_then(|b| {
                let data = NSData::with_bytes(b);
                let alloc_img: Allocated<AnyObject> = msg_send![class!(NSImage), alloc];
                let img: Option<Retained<AnyObject>> = msg_send![alloc_img, initWithData: &*data];
                img
            });
            let bounds: NSRect = msg_send![view, bounds];
            let flipped: bool = msg_send![view, isFlipped];
            let y = if flipped { pos.1 } else { bounds.size.height - pos.1 };
            let frame = NSRect::new(NSPoint::new(pos.0 - 24.0, y - 24.0), NSSize::new(48.0, 48.0));
            let contents: *const AnyObject = image.as_deref().map_or(std::ptr::null(), |i| i as *const AnyObject);
            let _: () = msg_send![&dragging, setDraggingFrame: frame, contents: contents];
            let items: Retained<AnyObject> = msg_send![class!(NSArray), arrayWithObject: &*dragging];
            let source: Retained<DragSource> = msg_send![DragSource::alloc(), init];
            let session: *mut AnyObject = msg_send![view, beginDraggingSessionWithItems: &*items, event: event, source: &*source];
            // The source must live as long as the session; AppKit keeps the session, so keep it too.
            std::mem::forget(source);
            !session.is_null()
        };
        match objc2::exception::catch(std::panic::AssertUnwindSafe(run)) {
            Ok(ok) => ok,
            Err(_) => false,
        }
    }
}

/// Starts dragging `text` (with rich text in its font) out of the window; false if the system refused.
#[allow(unused_variables)]
pub fn start(view: *mut std::ffi::c_void, text: &str, rtf: Option<&str>, png: Option<Vec<u8>>, file: Option<&std::path::Path>, pos: (f64, f64)) -> bool {
    #[cfg(target_os = "macos")]
    {
        mac::start(view as *mut objc2::runtime::AnyObject, text, rtf, png, file, pos)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
