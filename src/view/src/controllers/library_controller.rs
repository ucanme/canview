//! Library management controller
//!
//! Handles business logic for library and version management.

use crate::app::CanViewApp;
use crate::ChannelType;
use gpui::Context;

/// Create a new library
pub fn create_library(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    if app.new_library_name.trim().is_empty() {
        app.status_msg = "Library name cannot be empty".into();
        cx.notify();
        return;
    }

    match app.library_manager.create_library(
        app.new_library_name.clone(),
        ChannelType::CAN, // Default to CAN for now
    ) {
        Ok(library) => {
            eprintln!("✅ Library created successfully: {}", library.name);

            // Sync to app_config for persistence
            app.app_config.libraries = app.library_manager.libraries().to_vec();

            // Save config to file
            crate::controllers::config_controller::save_config(app, cx);

            app.status_msg = format!("Library '{}' created", app.new_library_name).into();
            app.new_library_name.clear();
            app.show_library_dialog = false;
            cx.notify();
        }
        Err(e) => {
            eprintln!("❌ Error creating library: {}", e);
            app.status_msg = format!("Error creating library: {}", e).into();
            cx.notify();
        }
    }
}

/// Delete a library by ID
pub fn delete_library(app: &mut CanViewApp, library_id: &str, cx: &mut Context<CanViewApp>) {
    match app
        .library_manager
        .delete_library(library_id, &app.app_config.mappings)
    {
        Ok(_) => {
            app.status_msg = format!("Library deleted").into();
            if app.selected_library_id.as_ref() == Some(&library_id.to_string()) {
                app.selected_library_id = None;
            }
            cx.notify();
        }
        Err(e) => {
            app.status_msg = format!("Error deleting library: {}", e).into();
            cx.notify();
        }
    }
}

/// Add a version to a library
pub fn add_library_version(app: &mut CanViewApp, cx: &mut Context<CanViewApp>) {
    let library_id = match &app.selected_library_id {
        Some(id) => id.clone(),
        None => {
            app.status_msg = "No library selected".into();
            cx.notify();
            return;
        }
    };

    // Get version name from input if available
    let version_name = if let Some(input) = &app.version_name_input {
        input.read(cx).value().to_string()
    } else {
        app.new_version_name.clone()
    };

    if version_name.trim().is_empty() {
        app.status_msg = "Version name cannot be empty".into();
        cx.notify();
        return;
    }

    eprintln!(
        "📝 Adding version: '{}' to library: {}",
        version_name, library_id
    );

    // Hide the input dialog
    app.show_version_input = false;
    cx.notify();

    // Create version directly
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let version = crate::models::library::LibraryVersion::new(
        version_name.clone(),
        String::new(), // Empty path for now
        date,
    )
    .with_description(format!(
        "Created version '{}' (database file to be added)",
        version_name
    ));

    // Add version directly to library
    if let Some(library) = app.library_manager.find_library_mut(&library_id) {
        library.add_version(version.clone());
        eprintln!("✅ Version '{}' added successfully", version_name);

        // Sync to app_config for persistence
        app.app_config.libraries = app.library_manager.libraries().to_vec();

        // Save config to file
        crate::controllers::config_controller::save_config(app, cx);

        app.status_msg = format!(
            "Version '{}' created successfully. Use 'Add Database File' to attach a database.",
            version_name
        )
        .into();
        app.new_version_name.clear();
        cx.notify();
    } else {
        eprintln!("❌ Error: Library not found");
        app.status_msg = "Error: Library not found".into();
        cx.notify();
    }
}

/// Delete a version from a library
pub fn delete_library_version(
    app: &mut CanViewApp,
    library_id: &str,
    version_name: &str,
    cx: &mut Context<CanViewApp>,
) {
    match app.library_manager.remove_version(
        library_id,
        version_name,
        &app.app_config.mappings,
    ) {
        Ok(_) => {
            app.status_msg = format!("Version '{}' deleted", version_name).into();
            cx.notify();
        }
        Err(e) => {
            app.status_msg = format!("Error deleting version: {}", e).into();
            cx.notify();
        }
    }
}

/// Load a library version into channels
pub fn load_library_version(
    app: &mut CanViewApp,
    library_id: &str,
    version_name: &str,
    cx: &mut Context<CanViewApp>,
) {
    let _ = internal_load_library_version(app, 1, library_id, version_name);
    cx.notify();
}

/// Apply a library version to channel mappings
pub fn apply_version_to_mappings(
    app: &mut CanViewApp,
    library_id: &str,
    version_name: &str,
    cx: &mut Context<CanViewApp>,
) {
    eprintln!(
        "🖱️ Applying version {} of {} to mappings",
        version_name, library_id
    );

    let library = match app.library_manager.find_library(library_id) {
        Some(lib) => lib,
        None => {
            app.status_msg = "Library not found".into();
            cx.notify();
            return;
        }
    };

    let version = match library.get_version(version_name) {
        Some(ver) => ver,
        None => {
            app.status_msg = "Version not found".into();
            cx.notify();
            return;
        }
    };

    // Update mappings
    let count = {
        let mut updated = 0;
        for channel_db in &version.channel_databases {
            if let Some(mapping) = app
                .app_config
                .mappings
                .iter_mut()
                .find(|m| m.channel_id == channel_db.channel_id)
            {
                mapping.library_id = Some(library_id.to_string());
                mapping.version_name = Some(version_name.to_string());
                mapping.channel_type = library.channel_type;
                updated += 1;
            } else {
                app.app_config
                    .mappings
                    .push(crate::models::ChannelMapping {
                        channel_id: channel_db.channel_id,
                        channel_type: library.channel_type,
                        library_id: Some(library_id.to_string()),
                        version_name: Some(version_name.to_string()),
                        path: String::new(),
                        description: String::new(),
                    });
                }
        }
        updated
    };

    // Load into memory
    let _ = internal_load_library_version(app, 1, library_id, version_name);

    // Save config
    crate::controllers::config_controller::save_config(app, cx);

    app.status_msg = format!("✅ Applied version {} to {} channels", version_name, count).into();
    cx.notify();
}

/// Internal method to load a library version without GPUI context
pub fn internal_load_library_version(
    app: &mut CanViewApp,
    default_channel_id: u16,
    library_id: &str,
    version_name: &str,
) -> Result<(), String> {
    eprintln!(
        "DEBUG: Internal load library version: lib={}, ver={}, ch={}",
        library_id, version_name, default_channel_id
    );

    let library = app
        .library_manager
        .find_library(library_id)
        .ok_or("Library not found")?;

    let version = library.get_version(version_name).ok_or("Version not found")?;

    // Load the database for each channel in the version
    let channel_dbs = &version.channel_databases;

    if channel_dbs.is_empty() {
        // Use the default path (backward compatibility)
        let path = &version.path;

        // Check if path is empty
        if path.trim().is_empty() {
            let msg = format!(
                "❌ Database path is empty for version '{}'. Please add a database file in the Library view.",
                version_name
            );
            app.status_msg = msg.clone().into();
            eprintln!("ERROR: Empty database path for version '{}'", version_name);
            return Err(msg);
        }

        // Check if file exists
        if !std::path::Path::new(path).exists() {
            let msg = format!(
                "❌ Database file not found: {}. Please check the file path in Library view.",
                path
            );
            app.status_msg = msg.clone().into();
            eprintln!("ERROR: Database file not found: {}", path);
            return Err(msg);
        }

        match app
            .library_manager
            .load_database(path, library.channel_type)
        {
            Ok(database) => {
                match database {
                    crate::library::Database::Dbc(dbc) => {
                        eprintln!("DEBUG: Inserting DBC into channel {}", default_channel_id);
                        app.dbc_channels.insert(default_channel_id, dbc);
                    }
                    crate::library::Database::Ldf(ldf) => {
                        eprintln!("DEBUG: Inserting LDF into channel {}", default_channel_id);
                        app.ldf_channels.insert(default_channel_id, ldf);
                    }
                }
                app.status_msg =
                    format!("✅ Loaded version {} of {}", version_name, library.name).into();
            }
            Err(e) => {
                let msg = format!("❌ Error loading database: {}", e);
                app.status_msg = msg.clone().into();
                eprintln!("ERROR: Failed to load database from '{}': {}", path, e);
                return Err(msg);
            }
        }
    } else {
        // Load all configured channels
        for channel_db in channel_dbs {
            // Check if path is empty
            if channel_db.database_path.trim().is_empty() {
                eprintln!(
                    "ERROR: Empty database path for channel {}",
                    channel_db.channel_id
                );
                continue;
            }

            // Check if file exists
            if !std::path::Path::new(&channel_db.database_path).exists() {
                eprintln!(
                    "ERROR: Database file not found for channel {}: {}",
                    channel_db.channel_id, channel_db.database_path
                );
                continue;
            }

            match app
                .library_manager
                .load_database(&channel_db.database_path, library.channel_type)
            {
                Ok(database) => match database {
                    crate::library::Database::Dbc(dbc) => {
                        eprintln!(
                            "DEBUG: Inserting DBC into channel {}",
                            channel_db.channel_id
                        );
                        app.dbc_channels.insert(channel_db.channel_id, dbc);
                    }
                    crate::library::Database::Ldf(ldf) => {
                        eprintln!(
                            "DEBUG: Inserting LDF into channel {}",
                            channel_db.channel_id
                        );
                        app.ldf_channels.insert(channel_db.channel_id, ldf);
                    }
                },
                Err(e) => {
                    eprintln!(
                        "ERROR: Failed to load database for channel {} from '{}': {}",
                        channel_db.channel_id, channel_db.database_path, e
                    );
                }
            }
        }
        app.status_msg = format!(
            "Loaded version {} of {} ({} channels)",
            version_name,
            library.name,
            channel_dbs.len()
        )
        .into();
    }

    eprintln!(
        "DEBUG: Current DBC channels: {:?}",
        app.dbc_channels.keys()
    );
    eprintln!(
        "DEBUG: Current LDF channels: {:?}",
        app.ldf_channels.keys()
    );

    Ok(())
}
