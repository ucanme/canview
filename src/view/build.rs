#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();

    // 使用绝对路径设置图标
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let icon_path = manifest_dir.join("../../assets/ico/can-viewer.ico");

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

    emit_version();
}

#[cfg(target_os = "macos")]
fn main() {
    // Emit rerun-if-changed for the icon so cargo rebuilds when it changes.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let icon_path = manifest_dir.join("../../assets/ico/can-viewer.icns");
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

    emit_version();
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn main() {
    // Linux/other: no platform-specific build steps
    emit_version();
}

/// Emit CAN_VIEWER_VERSION env var from `git describe --tags --always --dirty`.
/// Falls back to CARGO_PKG_VERSION when git is unavailable or no tags exist.
fn emit_version() {
    // Re-run when HEAD moves so the version stays accurate.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/tags");

    let version = std::process::Command::new("git")
        .args(["describe", "--tags", "--always"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            // Fall back to Cargo package version when git is unavailable or no tags.
            env!("CARGO_PKG_VERSION").to_string()
        });

    println!("cargo:rustc-env=CAN_VIEWER_VERSION={}", version);
}
