// Gives the Windows .exe its icon and version details. Does nothing on other systems.
fn main() {
    println!("cargo:rerun-if-changed=packaging/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("packaging/icon.ico");
        res.set("ProductName", "BeatCraft");
        res.set("FileDescription", "BeatCraft music studio");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed the Windows icon: {e}");
        }
    }
}
