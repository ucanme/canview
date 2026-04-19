//! Server commands
//!
//! Command handlers for starting/stopping the sharing server and importing libraries.

use crate::app::CanViewApp;

impl CanViewApp {
    /// Start the sharing server
    pub fn start_share_server(&mut self) {
        if self.server_handle.is_some() {
            self.import_status = Some("Server is already running".into());
            return;
        }

        let libraries = self.library_manager.libraries().to_vec();
        let config_path = self
            .config_file_path
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("multi_channel_config.json"));

        match crate::server::start_server(libraries, config_path) {
            Ok(handle) => {
                let url = handle.share_url.clone();
                log::info!("Share server started: {}", url);
                self.import_status = Some(format!("Server started! URL copied."));
                self.server_handle = Some(handle);
                self.show_share_dialog = true;
            }
            Err(e) => {
                log::error!("Failed to start server: {}", e);
                self.import_status = Some(format!("Failed to start: {}", e));
            }
        }
    }

    /// Stop the sharing server
    pub fn stop_share_server(&mut self) {
        if let Some(mut handle) = self.server_handle.take() {
            handle.shutdown();
            log::info!("Share server stopped");
            self.import_status = Some("Server stopped".into());
            self.show_share_dialog = false;
        }
    }

    /// Get the current share URL (if server is running) — LAN IP or localhost
    pub fn share_url(&self) -> Option<&str> {
        self.server_handle.as_ref().map(|h| h.url())
    }

    /// Get the localhost URL (always accessible on this machine)
    pub fn local_share_url(&self) -> Option<&str> {
        self.server_handle.as_ref().map(|h| h.local_url())
    }

    /// Import libraries from a remote URL (runs async on a background thread)
    pub fn start_import(&mut self, url: String) {
        let existing = self.library_manager.libraries().to_vec();
        self.import_status = Some("Importing...".into());

        // Determine local directory for saving downloaded files
        let local_lib_dir = self
            .config_file_path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.join("libraries")))
            .unwrap_or_else(|| std::path::PathBuf::from("libraries"));

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::Builder::new()
            .name("canview-import".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime for import");

                let result = rt.block_on(crate::server::import_from_url(&url, &existing, Some(local_lib_dir)));
                let _ = tx.send(result);
            })
            .expect("Failed to spawn import thread");

        // Store the receiver so we can poll for results
        // We'll check this in the next render cycle
        self.pending_import = Some(rx);
    }

    /// Poll for import completion (call from render or timer)
    pub fn poll_import(&mut self, cx: &mut gpui::Context<Self>) -> bool {
        if let Some(ref rx) = self.pending_import {
            match rx.try_recv() {
                Ok(Ok(imported)) => {
                    let count = imported.len();
                    for lib in imported {
                        // Create library if it doesn't already exist
                        let _ = self.library_manager.create_library(
                            lib.name.clone(),
                            lib.channel_type,
                        );
                        // Upsert versions: update existing or add new
                        if let Some(lib_ref) = self.library_manager.find_library_mut(&lib.id) {
                            for version in lib.versions {
                                // Remove existing version with same name so we can replace it
                                lib_ref.remove_version(&version.name);
                                lib_ref.add_version(version);
                            }
                        }
                    }
                    // Persist the updated libraries
                    self.app_config.libraries = self.library_manager.libraries().to_vec();
                    self.save_config(cx);
                    self.import_status = Some(format!("Imported {} libraries", count));
                    self.pending_import = None;
                    self.show_import_dialog = false;
                    return true;
                }
                Ok(Err(e)) => {
                    self.import_status = Some(format!("Import failed: {}", e));
                    self.pending_import = None;
                    return true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still in progress
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.import_status = Some("Import thread disconnected".into());
                    self.pending_import = None;
                    return true;
                }
            }
        }
        false
    }
}
