#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    // Set icon
    res.set_icon("assets/ico/canview.ico");
    // Set icon with ID
    res.set_icon_with_id("assets/ico/canview.ico", 1);
    // Compile the resource
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile resources: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Do nothing on non-Windows platforms
}
