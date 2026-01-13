#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    // Set icon
    res.set_icon("../../assets/ico/canview.ico");
    // Compile the resource
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile resources: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Do nothing on non-Windows platforms
}
