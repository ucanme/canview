#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();

    // Set icon
    res.set_icon("../../assets/ico/canview.ico");

    // Compile the resource
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile resources: {}", e);
    }

    // 在 Release 模式下设置 Windows 子系统为 GUI（隐藏控制台）
    #[cfg(not(debug_assertions))]
    {
        println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg-bins=/ENTRY:mainCRTStartup");
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Do nothing on non-Windows platforms
}
