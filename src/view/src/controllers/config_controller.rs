//! Configuration management controller
//!
//! Handles business logic for application configuration management.

use crate::app::CanViewApp;
use crate::AppConfig;
use gpui::Context;
use std::path::PathBuf;

/// Load configuration at startup
pub fn load_startup_config(app: &mut CanViewApp) {
    let exe_config = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("multi_channel_config.json")));
    let path = exe_config
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("multi_channel_config.json"));
    if path.exists() {
        app.status_msg = "Found saved config, loading...".into();
        if let Ok(content) = std::fs::read_to_string(&path) {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    // Save configuration
                    app.app_config = config.clone();
                    app.config_dir = Some(
                        path.parent()
                            .unwrap_or(std::path::Path::new("../../../../.."))
                            .to_path_buf(),
                    );
                    app.config_file_path = Some(path);

                    // Load signal libraries
                    if !config.libraries.is_empty() {
                        eprintln!("📚 Loading signal library configuration...");
                        eprintln!("  Found {} signal libraries", config.libraries.len());

                        // Load libraries into library_manager
                        app.library_manager =
                            crate::app::LibraryManager::from_libraries(config.libraries.clone());

                        // Statistics
                        let total_versions: usize = app
                            .library_manager
                            .libraries()
                            .iter()
                            .map(|lib| lib.versions.len())
                            .sum();
                        let total_channels: usize = app
                            .library_manager
                            .libraries()
                            .iter()
                            .flat_map(|lib| &lib.versions)
                            .map(|ver| ver.channel_databases.len())
                            .sum();

                        eprintln!("  ✅ Loading complete:");
                        eprintln!("     - {} libraries", app.library_manager.libraries().len());
                        eprintln!("     - {} versions", total_versions);
                        eprintln!("     - {} channels", total_channels);

                        // Display library list
                        for library in app.library_manager.libraries() {
                            eprintln!(
                                "     📦 {}: {} versions",
                                library.name,
                                library.versions.len()
                            );
                        }

                        app.status_msg = format!(
                            "Configuration loaded: {} libraries, {} versions, {} channels",
                            app.library_manager.libraries().len(),
                            total_versions,
                            total_channels
                        )
                        .into();

                        // Auto-load active version for each channel
                        for mapping in &config.mappings {
                            if let (Some(lib_id), Some(ver_name)) =
                                (&mapping.library_id, &mapping.version_name)
                            {
                                eprintln!(
                                    "  🔄 Auto-loading channel {} library {} version {}",
                                    mapping.channel_id, lib_id, ver_name
                                );
                                // Load the library version
                                let _ = crate::controllers::library_controller::internal_load_library_version(
                                    app,
                                    mapping.channel_id,
                                    lib_id,
                                    ver_name,
                                );
                            }
                        }
                    } else {
                        app.status_msg =
                            "Configuration loaded (no libraries configured).".into();
                    }
                }
                Err(e) => {
                    app.status_msg =
                        format!("Config load error: {}. Using default config.", e).into();
                    // Initialize with empty config instead of failing
                    app.app_config = AppConfig::default();
                    eprintln!("❌ Configuration loading failed: {}", e);
                }
            }
        }
    } else {
        app.status_msg = "Ready - GPUI version initialized".into();
        eprintln!("ℹ️  Configuration file not found, using default configuration");
    }
}

/// Load configuration from file
pub fn load_config(app: &mut CanViewApp, _cx: &mut Context<CanViewApp>) {
    // TODO: Implement file dialog for config selection
    app.status_msg = "Load config - file dialog not yet implemented".into();
    eprintln!("⚠️  Load config: file dialog not yet implemented");
}

/// Save configuration to file
pub fn save_config(app: &CanViewApp, _cx: &mut Context<CanViewApp>) {
    let config_path = app.config_file_path.clone().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("multi_channel_config.json")))
            .unwrap_or_else(|| PathBuf::from("multi_channel_config.json"))
    });
    if let Ok(content) = serde_json::to_string_pretty(&app.app_config) {
        if std::fs::write(&config_path, content).is_ok() {
            eprintln!("✅ Configuration saved to: {}", config_path.display());
            return;
        }
    }
    eprintln!("❌ Failed to save configuration");
}

/// Import database file
pub fn import_database_file(app: &mut CanViewApp, _cx: &mut Context<CanViewApp>) {
    // TODO: Implement file dialog for database import
    app.status_msg = "Import database - file dialog not yet implemented".into();
    eprintln!("⚠️  Import database: file dialog not yet implemented");
}

/// Apply BLF file loading result
pub fn apply_blf_result(app: &mut CanViewApp, result: anyhow::Result<blf::BlfResult>) {
    // Delegate to the impl method, with no file name (controller path doesn't have it)
    app.apply_blf_result(result, None);
}
