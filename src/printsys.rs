//! The system's own print dialog on Windows and Linux, printing FontCraft's laid-out pages
//! (grey pictures at 2 pixels per point) directly, so the person picks the printer, copies and
//! pages there, as macOS's print panel does:
//! - Windows: the common Print dialog (comdlg32 PrintDlgW, which returns the printer's device
//!   context), then each page drawn with StretchDIBits between StartDoc/EndDoc.
//! - Linux: GTK's print dialog (GtkPrintOperation, the one GNOME, Xfce, Cinnamon and MATE apps
//!   show), drawing each page with cairo; it runs on a GTK thread of its own. When GTK can't start
//!   (no display), the PDF opens in the default viewer instead.

/// One page: width and height in pixels, grey values (0 black … 255 white), row after row.
pub struct Sheet {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

/// Shows the print dialog and prints `sheets` on `paper_w` × `paper_h` points. Returns Ok when the
/// dialog was shown (the person may still cancel), Err with the reason otherwise.
#[cfg(target_os = "windows")]
pub fn print(sheets: Vec<Sheet>, paper_w: f32, paper_h: f32, title: &str) -> Result<(), String> {
    win::print(&sheets, paper_w, paper_h, title)
}

#[cfg(all(target_os = "linux", not(target_os = "android")))]
pub fn print(sheets: Vec<Sheet>, paper_w: f32, paper_h: f32, title: &str) -> Result<(), String> {
    gtk_print::print(sheets, paper_w, paper_h, title)
}

#[cfg(target_os = "windows")]
mod win {
    use super::Sheet;
    use std::ffi::c_void;

    #[repr(C)]
    struct PrintDlgW {
        l_struct_size: u32,
        hwnd_owner: isize,
        h_dev_mode: isize,
        h_dev_names: isize,
        h_dc: isize,
        flags: u32,
        n_from_page: u16,
        n_to_page: u16,
        n_min_page: u16,
        n_max_page: u16,
        n_copies: u16,
        h_instance: isize,
        l_cust_data: isize,
        lpfn_print_hook: isize,
        lpfn_setup_hook: isize,
        lp_print_template_name: *const u16,
        lp_setup_template_name: *const u16,
        h_print_template: isize,
        h_setup_template: isize,
    }

    #[repr(C)]
    struct DocInfoW {
        cb_size: i32,
        lpsz_doc_name: *const u16,
        lpsz_output: *const u16,
        lpsz_datatype: *const u16,
        fw_type: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        bi_size: u32,
        bi_width: i32,
        bi_height: i32,
        bi_planes: u16,
        bi_bit_count: u16,
        bi_compression: u32,
        bi_size_image: u32,
        bi_x_pels_per_meter: i32,
        bi_y_pels_per_meter: i32,
        bi_clr_used: u32,
        bi_clr_important: u32,
        colors: [u32; 256],
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn PrintDlgW(pd: *mut PrintDlgW) -> i32;
    }
    #[link(name = "gdi32")]
    extern "system" {
        fn StartDocW(hdc: isize, di: *const DocInfoW) -> i32;
        fn StartPage(hdc: isize) -> i32;
        fn EndPage(hdc: isize) -> i32;
        fn EndDoc(hdc: isize) -> i32;
        fn DeleteDC(hdc: isize) -> i32;
        fn GetDeviceCaps(hdc: isize, index: i32) -> i32;
        fn StretchDIBits(hdc: isize, xd: i32, yd: i32, wd: i32, hd: i32, xs: i32, ys: i32, ws: i32, hs: i32, bits: *const c_void, bmi: *const BitmapInfo, usage: u32, rop: u32) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalFree(h: isize) -> isize;
    }
    #[link(name = "user32")]
    extern "system" {
        fn GetActiveWindow() -> isize;
    }

    const PD_RETURNDC: u32 = 0x100;
    const PD_NOSELECTION: u32 = 0x4;
    const PD_PAGENUMS: u32 = 0x2;
    const PD_USEDEVMODECOPIESANDCOLLATE: u32 = 0x40000;
    const LOGPIXELSX: i32 = 88;
    const LOGPIXELSY: i32 = 90;
    const PHYSICALOFFSETX: i32 = 112;
    const PHYSICALOFFSETY: i32 = 113;
    const SRCCOPY: u32 = 0x00CC_0020;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn print(sheets: &[Sheet], paper_w: f32, _paper_h: f32, title: &str) -> Result<(), String> {
        let n = sheets.len().clamp(1, u16::MAX as usize) as u16;
        let mut pd = PrintDlgW {
            l_struct_size: std::mem::size_of::<PrintDlgW>() as u32,
            hwnd_owner: unsafe { GetActiveWindow() },
            h_dev_mode: 0,
            h_dev_names: 0,
            h_dc: 0,
            flags: PD_RETURNDC | PD_NOSELECTION | PD_USEDEVMODECOPIESANDCOLLATE,
            n_from_page: 1,
            n_to_page: n,
            n_min_page: 1,
            n_max_page: n,
            n_copies: 1,
            h_instance: 0,
            l_cust_data: 0,
            lpfn_print_hook: 0,
            lpfn_setup_hook: 0,
            lp_print_template_name: std::ptr::null(),
            lp_setup_template_name: std::ptr::null(),
            h_print_template: 0,
            h_setup_template: 0,
        };
        // Cancelled (or no printer): nothing more to do.
        if unsafe { PrintDlgW(&mut pd) } == 0 || pd.h_dc == 0 {
            return Ok(());
        }
        let hdc = pd.h_dc;
        let (from, to) = if pd.flags & PD_PAGENUMS != 0 { (pd.n_from_page.max(1) as usize, pd.n_to_page.max(pd.n_from_page) as usize) } else { (1, sheets.len()) };
        let name = wide(title);
        let di = DocInfoW { cb_size: std::mem::size_of::<DocInfoW>() as i32, lpsz_doc_name: name.as_ptr(), lpsz_output: std::ptr::null(), lpsz_datatype: std::ptr::null(), fw_type: 0 };
        let mut ok = unsafe { StartDocW(hdc, &di) } > 0;
        if ok {
            let (dpi_x, dpi_y) = unsafe { (GetDeviceCaps(hdc, LOGPIXELSX) as f32, GetDeviceCaps(hdc, LOGPIXELSY) as f32) };
            let (off_x, off_y) = unsafe { (GetDeviceCaps(hdc, PHYSICALOFFSETX), GetDeviceCaps(hdc, PHYSICALOFFSETY)) };
            let mut bmi = BitmapInfo {
                bi_size: 40,
                bi_width: 0,
                bi_height: 0,
                bi_planes: 1,
                bi_bit_count: 8,
                bi_compression: 0,
                bi_size_image: 0,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 256,
                bi_clr_important: 256,
                colors: [0; 256],
            };
            for (i, c) in bmi.colors.iter_mut().enumerate() {
                let g = i as u32;
                *c = (g << 16) | (g << 8) | g;
            }
            for s in sheets.iter().take(to).skip(from - 1) {
                // Rows padded to 4 bytes, as DIBs are; top-down (negative height).
                let stride = s.w.div_ceil(4) * 4;
                let mut bits = vec![255u8; stride * s.h];
                for y in 0..s.h {
                    bits[y * stride..y * stride + s.w].copy_from_slice(&s.px[y * s.w..(y + 1) * s.w]);
                }
                bmi.bi_width = s.w as i32;
                bmi.bi_height = -(s.h as i32);
                // The page's own size on paper: its pixels are 2 per point, 72 points an inch.
                let k = paper_w / s.w as f32;
                let (wd, hd) = ((s.w as f32 * k / 72.0 * dpi_x).round() as i32, (s.h as f32 * k / 72.0 * dpi_y).round() as i32);
                unsafe {
                    if StartPage(hdc) <= 0 {
                        ok = false;
                        break;
                    }
                    StretchDIBits(hdc, -off_x, -off_y, wd, hd, 0, 0, s.w as i32, s.h as i32, bits.as_ptr() as *const c_void, &bmi, 0, SRCCOPY);
                    if EndPage(hdc) <= 0 {
                        ok = false;
                        break;
                    }
                }
            }
            unsafe { EndDoc(hdc) };
        }
        unsafe {
            DeleteDC(hdc);
            if pd.h_dev_mode != 0 {
                GlobalFree(pd.h_dev_mode);
            }
            if pd.h_dev_names != 0 {
                GlobalFree(pd.h_dev_names);
            }
        }
        if ok { Ok(()) } else { Err("the printer refused the page".into()) }
    }
}

#[cfg(all(target_os = "linux", not(target_os = "android")))]
mod gtk_print {
    use super::Sheet;
    use std::sync::mpsc::{channel, Sender};
    use std::sync::{Mutex, OnceLock};

    struct Job {
        sheets: Vec<Sheet>,
        w: f32,
        h: f32,
        title: String,
        done: Sender<Result<(), String>>,
    }

    /// The one thread GTK runs on (GTK may only ever be used from the thread that started it).
    fn worker() -> Option<&'static Mutex<Sender<Job>>> {
        static W: OnceLock<Option<Mutex<Sender<Job>>>> = OnceLock::new();
        W.get_or_init(|| {
            let (tx, rx) = channel::<Job>();
            let (ready_tx, ready_rx) = channel::<bool>();
            std::thread::spawn(move || {
                let ok = gtk::init().is_ok();
                let _ = ready_tx.send(ok);
                if !ok {
                    return;
                }
                while let Ok(job) = rx.recv() {
                    let r = run(job.sheets, job.w, job.h, &job.title);
                    let _ = job.done.send(r);
                }
            });
            if ready_rx.recv().unwrap_or(false) { Some(Mutex::new(tx)) } else { None }
        })
        .as_ref()
    }

    fn run(sheets: Vec<Sheet>, w: f32, h: f32, title: &str) -> Result<(), String> {
        use gtk::prelude::*;
        let op = gtk::PrintOperation::new();
        op.set_job_name(title);
        op.set_n_pages(sheets.len().max(1) as i32);
        op.set_use_full_page(true);
        op.set_unit(gtk::Unit::Points);
        let setup = gtk::PageSetup::new();
        let size = gtk::PaperSize::new_custom("fontcraft", title, w as f64, h as f64, gtk::Unit::Points);
        setup.set_paper_size(&size);
        op.set_default_page_setup(Some(&setup));
        op.connect_draw_page(move |_, ctx, n| {
            let Some(s) = sheets.get(n as usize) else { return };
            let Some(cr) = ctx.cairo_context() else { return };
            // Each page as an alpha mask (dark where the page is dark), in black on the white sheet.
            let stride = gtk::cairo::Format::A8.stride_for_width(s.w as u32).unwrap_or(s.w as i32) as usize;
            let mut data = vec![0u8; stride * s.h];
            for y in 0..s.h {
                for x in 0..s.w {
                    data[y * stride + x] = 255 - s.px[y * s.w + x];
                }
            }
            let Ok(surf) = gtk::cairo::ImageSurface::create_for_data(data, gtk::cairo::Format::A8, s.w as i32, s.h as i32, stride as i32) else { return };
            let k = ctx.width() / s.w as f64;
            cr.scale(k, k);
            cr.set_source_rgb(0.0, 0.0, 0.0);
            let _ = cr.mask_surface(&surf, 0.0, 0.0);
        });
        op.run(gtk::PrintOperationAction::PrintDialog, None::<&gtk::Window>).map(|_| ()).map_err(|e| e.to_string())
    }

    pub fn print(sheets: Vec<Sheet>, w: f32, h: f32, title: &str) -> Result<(), String> {
        let Some(tx) = worker() else { return Err("GTK".into()) };
        let (done, wait) = channel();
        tx.lock().map_err(|e| e.to_string())?.send(Job { sheets, w, h, title: title.to_string(), done }).map_err(|e| e.to_string())?;
        // The dialog runs on GTK's thread; FontCraft goes on drawing meanwhile.
        std::thread::spawn(move || {
            if let Ok(Err(e)) = wait.recv() {
                eprintln!("print: {e}");
            }
        });
        Ok(())
    }
}
