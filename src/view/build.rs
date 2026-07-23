#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();

    // 使用绝对路径设置图标
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let icon_path = manifest_dir.join("../../assets/ico/canview.ico");

    // 转换为字符串
    let icon_path_str = icon_path.to_str().expect("Invalid icon path");

    println!("cargo:rerun-if-changed={}", icon_path.display());

    // 设置图标
    res.set_icon(icon_path_str);

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

#[cfg(target_os = "macos")]
fn main() {
    // Emit rerun-if-changed for the icon so cargo rebuilds when it changes.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let icon_path = manifest_dir.join("../../assets/ico/canview.icns");
    println!("cargo:rerun-if-changed={}", icon_path.display());

    // macOS: place the .icns inside the binary's Resources dir so the
    // system picks it up when the binary is wrapped in an .app bundle by
    // scripts/package-macos.sh. We also create a minimal Info.plist next
    // to the binary for `cargo run` users who don't run the packaging
    // script — but that won't make Dock show the icon by itself; only an
    // .app bundle does. See README for `cargo bundle` or scripts/.
    //
    // Link the Metal framework (required by gpui on macOS).
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn main() {
    // Linux/other: no platform-specific build steps
}
