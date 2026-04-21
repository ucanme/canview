//! HTTP server for signal library sharing
//!
//! Provides HTTP API to share and import signal libraries between CANVIEW instances.

mod handlers;
mod routes;
mod token;

pub use token::TokenManager;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Shared state accessible by all HTTP handlers
pub struct SharedState {
    pub token_manager: TokenManager,
    pub config_path: std::path::PathBuf,
    pub libraries: Mutex<Vec<crate::models::SignalLibrary>>,
    /// Base URL (e.g. "http://192.168.1.100:8080") for constructing download URLs in API responses
    pub base_url: String,
}

/// Handle to a running server instance
pub struct ServerHandle {
    pub addr: SocketAddr,
    pub token: String,
    pub share_url: String,
    pub local_url: String,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ServerHandle {
    /// Get the full share URL (with token) — LAN IP if available, else localhost
    pub fn url(&self) -> &str {
        &self.share_url
    }

    /// Get the localhost URL (always works on the same machine)
    pub fn local_url(&self) -> &str {
        &self.local_url
    }

    /// Shut down the server
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Start the HTTP server on a random available port.
///
/// Returns a `ServerHandle` that contains the URL and can be used to stop the server.
pub fn start_server(
    libraries: Vec<crate::models::SignalLibrary>,
    config_path: std::path::PathBuf,
) -> Result<ServerHandle, String> {
    let token_manager = TokenManager::new();
    let token = token_manager.token().to_string();

    // Bind the port first (in the main thread) so we know the address before building SharedState
    let std_listener = std::net::TcpListener::bind("0.0.0.0:0")
        .map_err(|e| format!("Failed to bind server port: {}", e))?;
    // tokio::net::TcpListener::from_std requires the socket to be non-blocking
    std_listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;
    let addr = std_listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;

    // Try to get local IP for LAN sharing, prefer real private network IPs
    let ip = get_local_ip().unwrap_or_else(|| addr.ip());
    let lan_base = format!("http://{}:{}", ip, addr.port());
    let local_base = format!("http://127.0.0.1:{}", addr.port());
    let base_url = if is_preferred_lan_ip(&ip) { lan_base } else { local_base.clone() };

    let local_url = format!("{}/api/libraries?token={}", local_base, token);
    let share_url = format!("{}/api/libraries?token={}", base_url, token);

    let state = Arc::new(SharedState {
        token_manager,
        config_path,
        libraries: Mutex::new(libraries),
        base_url: base_url.clone(),
    });

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn server on a dedicated thread with its own tokio runtime
    std::thread::Builder::new()
        .name("canview-server".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for server");

            rt.block_on(async move {
                let app = routes::create_router(state);

                // Convert std listener to tokio listener inside the async context
                let listener = tokio::net::TcpListener::from_std(std_listener)
                    .expect("Failed to convert std listener to tokio listener");

                log::info!("CANVIEW server started on {}", addr);

                axum::serve(listener, app)
                    .with_graceful_shutdown(async {
                        let _ = shutdown_rx.await;
                        log::info!("CANVIEW server shutting down");
                    })
                    .await
                    .expect("Server error");
            });
        })
        .map_err(|e| format!("Failed to spawn server thread: {}", e))?;

    Ok(ServerHandle {
        addr,
        token,
        share_url,
        local_url,
        shutdown_tx: Some(shutdown_tx),
    })
}

/// Import libraries from a remote CANVIEW server URL
pub async fn import_from_url(
    url: &str,
    existing_libraries: &[crate::models::SignalLibrary],
    local_lib_dir: Option<std::path::PathBuf>,
) -> Result<Vec<crate::models::SignalLibrary>, String> {
    // Parse the URL to extract base and token
    let (base_url, token) = parse_share_url(url)?;

    let client = reqwest::Client::new();

    // Fetch library list
    let resp = client
        .get(format!("{}/api/libraries?token={}", base_url, token))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Server returned error: {}", resp.status()));
    }

    let remote_libs: Vec<crate::models::SignalLibrary> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Determine local directory to store downloaded files
    let save_dir = local_lib_dir.unwrap_or_else(|| std::path::PathBuf::from("libraries"));
    std::fs::create_dir_all(&save_dir)
        .map_err(|e| format!("Failed to create import directory '{}': {}", save_dir.display(), e))?;

    // Deduplicate names and download files
    let mut imported = Vec::new();
    for mut lib in remote_libs {
        let original_name = lib.name.clone();
        let original_id = lib.id.clone();
        let deduped_name = deduplicate_name(&original_name, existing_libraries, &imported);
        if deduped_name != original_name {
            lib.name = deduped_name.clone();
            lib.id = crate::library::generate_library_id(&deduped_name);
        }

        // Download database files for each version and persist local paths.
        for version in &mut lib.versions {
            let mut first_local_path: Option<String> = None;

            for channel_db in &mut version.channel_databases {
                let channel_dir_name = if channel_db.channel_name.trim().is_empty() {
                    format!("channel_{}", channel_db.channel_id)
                } else {
                    channel_db.channel_name.clone()
                };
                let download_url = if is_http_url(&channel_db.database_path) {
                    channel_db.database_path.clone()
                } else {
                    format!(
                        "{}/api/libraries/{}/versions/{}/files/{}?token={}",
                        base_url,
                        original_id,
                        urlencoding::encode(&version.name),
                        channel_db.channel_id,
                        token
                    )
                };

                let local_path = download_file_to_local_path(
                    &client,
                    &download_url,
                    &save_dir,
                    &crate::library::build_library_file_subdir(&lib.name, &version.name, &channel_dir_name),
                    &channel_db.database_path,
                    &format!("channel {} in version '{}'", channel_db.channel_id, version.name),
                )
                .await?;

                if first_local_path.is_none() {
                    first_local_path = Some(local_path.clone());
                }

                channel_db.database_path = local_path;
            }

            if let Some(first_local_path) = first_local_path {
                version.path = first_local_path;
            } else if is_http_url(&version.path) {
                version.path = download_file_to_local_path(
                    &client,
                    &version.path,
                    &save_dir,
                    &crate::library::build_library_file_subdir(&lib.name, &version.name, "default"),
                    &version.path,
                    &format!("version '{}'", version.name),
                )
                .await?;
            }
        }

        imported.push(lib);
    }

    Ok(imported)
}

/// Extract a filename from a download response (Content-Disposition or fallback to original path)
fn extract_filename_from_response(
    response: &reqwest::Response,
    fallback_path: &str,
) -> String {
    // Try Content-Disposition header first
    if let Some(cd) = response.headers().get(reqwest::header::CONTENT_DISPOSITION) {
        if let Ok(cd_str) = cd.to_str() {
            // Parse filename from: attachment; filename="foo.dbc"
            if let Some(fn_start) = cd_str.find("filename=\"") {
                let rest = &cd_str[fn_start + 10..];
                if let Some(fn_end) = rest.find('"') {
                    return rest[..fn_end].to_string();
                }
            }
        }
    }

    // Fallback: use the last component of the stored path
    std::path::Path::new(fallback_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("database.dbc")
        .to_string()
}

fn is_http_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

async fn download_file_to_local_path(
    client: &reqwest::Client,
    download_url: &str,
    save_dir: &std::path::Path,
    relative_dir: &std::path::Path,
    fallback_path: &str,
    context: &str,
) -> Result<String, String> {
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {} from '{}': {}", context, download_url, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned {} while downloading {} from '{}'",
            response.status(),
            context,
            download_url
        ));
    }

    let filename = extract_filename_from_response(&response, fallback_path);
    let target_dir = save_dir.join(relative_dir);
    std::fs::create_dir_all(&target_dir).map_err(|e| {
        format!(
            "Failed to create import subdirectory '{}' for {}: {}",
            target_dir.display(),
            context,
            e
        )
    })?;
    let local_path = target_dir.join(&filename);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read downloaded bytes for {}: {}", context, e))?;

    std::fs::write(&local_path, &bytes).map_err(|e| {
        format!(
            "Failed to save {} to '{}': {}",
            context,
            local_path.display(),
            e
        )
    })?;

    Ok(local_path.to_string_lossy().to_string())
}

/// Parse a share URL into (base_url, token)
fn parse_share_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim();

    // Extract token from query string
    if let Some(token_pos) = url.find("token=") {
        let token_start = token_pos + 6;
        let token = url[token_start..]
            .split('&')
            .next()
            .unwrap_or("")
            .to_string();

        // Extract base URL (everything before /api/)
        let base_url = if let Some(api_pos) = url.find("/api/") {
            url[..api_pos].to_string()
        } else {
            // Try to extract host:port
            let without_token = url[..token_pos].trim_end_matches('?').trim_end_matches('&');
            without_token.to_string()
        };

        if token.is_empty() {
            return Err("Token not found in URL".to_string());
        }

        Ok((base_url, token))
    } else {
        Err("Invalid share URL: missing token parameter".to_string())
    }
}

/// Generate a unique name by appending _v1, _v2, etc. if name conflicts exist
fn deduplicate_name(
    name: &str,
    existing: &[crate::models::SignalLibrary],
    importing: &[crate::models::SignalLibrary],
) -> String {
    let all_names: Vec<&str> = existing
        .iter()
        .chain(importing.iter())
        .map(|lib| lib.name.as_str())
        .collect();

    if !all_names.contains(&name) {
        return name.to_string();
    }

    let mut counter = 1;
    loop {
        let candidate = format!("{}_{}", name, counter);
        if !all_names.contains(&candidate.as_str()) {
            return candidate;
        }
        counter += 1;
    }
}

/// Get local IP address for LAN access
fn get_local_ip() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

/// Returns true if the IP is a standard private LAN address (not a virtual adapter range).
fn is_preferred_lan_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            // Standard private ranges: 10.x.x.x, 172.16-31.x.x, 192.168.x.x
            let is_private = octets[0] == 10
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168);
            // Exclude benchmark/virtual ranges: 198.18.0.0/15, 169.254.x.x (link-local)
            let is_virtual = (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 169 && octets[1] == 254);
            is_private && !is_virtual
        }
        std::net::IpAddr::V6(_) => false, // Prefer IPv4 LAN for sharing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_share_url() {
        let url = "http://192.168.1.100:8080/api/libraries?token=abc123";
        let (base, token) = parse_share_url(url).unwrap();
        assert_eq!(base, "http://192.168.1.100:8080");
        assert_eq!(token, "abc123");
    }

    #[test]
    fn test_deduplicate_name() {
        let existing = vec![
            crate::models::SignalLibrary::new("id1".into(), "MyLib".into(), crate::models::ChannelType::CAN),
            crate::models::SignalLibrary::new("id2".into(), "MyLib_1".into(), crate::models::ChannelType::CAN),
        ];

        assert_eq!(deduplicate_name("OtherLib", &existing, &[]), "OtherLib");
        assert_eq!(deduplicate_name("MyLib", &existing, &[]), "MyLib_2");
    }
}
