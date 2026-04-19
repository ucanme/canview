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
}

/// Handle to a running server instance
pub struct ServerHandle {
    pub addr: SocketAddr,
    pub token: String,
    pub share_url: String,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl ServerHandle {
    /// Get the full share URL (with token)
    pub fn url(&self) -> &str {
        &self.share_url
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

    let state = Arc::new(SharedState {
        token_manager,
        config_path,
        libraries: Mutex::new(libraries),
    });

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let (addr_tx, addr_rx) = std::sync::mpsc::channel::<SocketAddr>();

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

                let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
                    .await
                    .expect("Failed to bind server port");

                let addr = listener.local_addr().expect("Failed to get local address");
                let _ = addr_tx.send(addr);

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

    let addr = addr_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| "Timeout waiting for server to start".to_string())?;

    // Try to get local IP for LAN sharing
    let ip = get_local_ip().unwrap_or_else(|| addr.ip());
    let share_url = format!("http://{}:{}/api/libraries?token={}", ip, addr.port(), token);

    Ok(ServerHandle {
        addr,
        token,
        share_url,
        shutdown_tx: Some(shutdown_tx),
    })
}

/// Import libraries from a remote CANVIEW server URL
pub async fn import_from_url(
    url: &str,
    existing_libraries: &[crate::models::SignalLibrary],
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

    // Deduplicate names
    let mut imported = Vec::new();
    for mut lib in remote_libs {
        let original_name = lib.name.clone();
        let deduped_name = deduplicate_name(&original_name, existing_libraries, &imported);
        if deduped_name != original_name {
            lib.name = deduped_name.clone();
            // Regenerate ID based on new name
            lib.id = crate::library::generate_library_id(&deduped_name);
        }
        imported.push(lib);
    }

    Ok(imported)
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
