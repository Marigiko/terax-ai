use std::env;
use std::fmt::Write;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;

use tauri::command;

/// Start the AI Workstation gateway as a docker-compose service.
/// Returns Ok(()) on start or if already running.
#[command]
pub async fn start_gateway(gateway_dir: Option<String>) -> Result<(), String> {
    let dir = gateway_dir.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("ai-workstation").to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let compose_path = PathBuf::from(&dir).join("infrastructure/compose/docker-compose.prod.yml");
    if !compose_path.exists() {
        return Err(format!(
            "docker-compose.prod.yml not found at {}. Clone ai-workstation first.",
            compose_path.display()
        ));
    }

    // Check if gateway is already healthy
    if let Ok(resp) = reqwest::get("http://localhost:8000/health").await {
        if resp.status().is_success() {
            return Ok(());
        }
    }

    let status = std::process::Command::new("docker")
        .args([
            "compose",
            "-f",
            compose_path.to_str().unwrap(),
            "up",
            "-d",
        ])
        .current_dir(&dir)
        .status()
        .map_err(|e| format!("Failed to start gateway: {e}"))?;

    if !status.success() {
        return Err("docker compose up failed".to_string());
    }

    // Wait for health
    for _ in 0..60 {
        if let Ok(resp) = reqwest::get("http://localhost:8000/health").await {
            if resp.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    Err("Gateway did not become healthy within 120 seconds".to_string())
}

/// Resolve the ai-workstation gateway directory.
/// Priority: AI_WORKSTATION_DIR env var > ~/Projects/ai-workstation > ~/ai-workstation.
#[command]
pub fn get_gateway_dir() -> Option<String> {
    if let Ok(dir) = std::env::var("AI_WORKSTATION_DIR") {
        let path = std::path::Path::new(&dir);
        if path.join("infrastructure/compose/docker-compose.prod.yml").exists() {
            return Some(dir);
        }
    }
    let candidates = ["Projects/ai-workstation", "ai-workstation"];
    for rel in candidates {
        if let Some(home) = dirs::home_dir() {
            let p = home.join(rel);
            if p.join("infrastructure/compose/docker-compose.prod.yml").exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Serve the built UI as a web app on the given port.
/// This opens the dist/ folder via a minimal HTTP server.
#[command]
pub fn serve_web(port: Option<u16>) -> Result<String, String> {
    let port = port.unwrap_or(8080);
    let dist_dir = find_dist_dir()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .map_err(|e| format!("Cannot bind to port {port}: {e}"))?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            handle_request(&mut stream, &dist_dir);
        }
    });

    Ok(format!("http://localhost:{port}"))
}

fn handle_request(stream: &mut std::net::TcpStream, dist_dir: &std::path::Path) {
    use std::io::{BufRead, BufReader, Write};

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let requested_path = parts[1];
    let relative = if requested_path == "/" {
        "/index.html"
    } else {
        requested_path
    };

    // Sanitize path to prevent directory traversal
    let relative = relative.split('?').next().unwrap_or("/");
    let relative = relative.trim_start_matches('/');
    let relative = relative.replace("..", "");

    let file_path = dist_dir.join(relative);

    // Ensure the resolved path is within dist_dir
    if !file_path.starts_with(dist_dir) {
        let response = "HTTP/1.1 403 Forbidden\r\n\r\n";
        let _ = stream.write_all(response.as_bytes());
        return;
    }

    let (status, content_type, body) = if file_path.exists() && file_path.is_file() {
        let content_type = match file_path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript",
            Some("css") => "text/css",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("woff2") => "font/woff2",
            _ => "application/octet-stream",
        };
        match fs::read(&file_path) {
            Ok(data) => ("HTTP/1.1 200 OK", content_type, data),
            Err(_) => ("HTTP/1.1 500 Internal Server Error", "text/plain", b"Read error".to_vec()),
        }
    } else {
        // SPA fallback: serve index.html for unknown routes
        let index_path = dist_dir.join("index.html");
        if index_path.exists() {
            match fs::read(&index_path) {
                Ok(data) => ("HTTP/1.1 200 OK", "text/html; charset=utf-8", data),
                Err(_) => ("HTTP/1.1 404 Not Found", "text/plain", b"Not found".to_vec()),
            }
        } else {
            ("HTTP/1.1 404 Not Found", "text/plain", b"Not found".to_vec())
        }
    };

    let mut header = String::new();
    let _ = write!(
        header,
        "{status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );

    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
    let _ = stream.flush();
}

fn find_dist_dir() -> Result<PathBuf, String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = PathBuf::from(&manifest_dir);

    // Try relative to src-tauri: ../dist
    let candidates = [
        manifest_path.parent().map(|p| p.join("dist")),
        Some(PathBuf::from("./dist")),
        Some(PathBuf::from("../dist")),
    ];

    for candidate in candidates.flatten() {
        let index = candidate.join("index.html");
        if index.exists() {
            return Ok(candidate);
        }
    }

    Err("dist/ directory not found. Run `pnpm build` first.".to_string())
}
