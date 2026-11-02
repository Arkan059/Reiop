fn main() {
    if std::path::Path::new("icon.ico").exists() {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("ProductName", "Reiop");
        res.set("FileDescription", "Windows optimization and diagnostic CLI");
        res.set("LegalCopyright", "arkanquixote");
        let _ = res.compile();
    }
}
