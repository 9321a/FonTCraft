//! Windows: puts FontCraft's icon and name into fontcraft.exe (Explorer, taskbar, installer).
fn main() {
    println!("cargo:rerun-if-changed=assets/icons/fontcraft.ico");
    #[cfg(windows)]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/icons/fontcraft.ico");
            res.set("ProductName", "FontCraft");
            res.set("FileDescription", "FontCraft");
            res.set("CompanyName", "FontCraft");
            if let Err(e) = res.compile() {
                println!("cargo:warning=icon not embedded: {e}");
            }
        }
    }
}
