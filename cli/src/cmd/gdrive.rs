// Google Drive OAuth (PKCE flow) + file download for compass:sprint import.
// Client ID embedded via gitignored `gdrive_client.rs` module.
// Uses PKCE so no client secret is needed at runtime.

use crate::gdrive_client::{CLIENT_ID, CLIENT_SECRET};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

const TOKEN_PATH_REL: &str = ".compass/.gdrive-token.json";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

fn token_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(TOKEN_PATH_REL)
}

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("Usage: compass-cli gdrive <auth|download|status> [args]".into());
    }
    match args[0].as_str() {
        "auth" => auth(),
        "status" => status(),
        "download" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli gdrive download <file_id_or_url> [output_path]".into());
            }
            let id = extract_file_id(&args[1]);
            let out = args.get(2).cloned();
            download(&id, out.as_deref())
        }
        other => Err(format!("Unknown gdrive subcommand: {}", other)),
    }
}

// ────────────────────────────────────────────────────────────────────
// auth — PKCE flow
// ────────────────────────────────────────────────────────────────────

fn auth() -> Result<String, String> {
    // 1. Generate PKCE verifier + challenge
    let verifier = random_string(64);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    // 2. Start local loopback server to catch redirect
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Cannot bind local port: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("port: {}", e))?
        .port();
    let redirect = format!("http://127.0.0.1:{}", port);

    // 3. Build auth URL
    let state = random_string(16);
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&state={}&access_type=offline&prompt=consent",
        AUTH_URL,
        urlencode(CLIENT_ID),
        urlencode(&redirect),
        urlencode(SCOPE),
        challenge,
        state
    );

    // 4. Open browser
    eprintln!("Opening browser for Google login...");
    eprintln!("If browser doesn't open, visit: {}", auth_url);
    let _ = std::process::Command::new("open").arg(&auth_url).spawn();

    // 5. Wait for redirect with code
    listener
        .set_nonblocking(false)
        .map_err(|e| format!("listener: {}", e))?;
    let (mut stream, _) = listener
        .accept()
        .map_err(|e| format!("accept: {}", e))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .ok();

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).map_err(|e| format!("read: {}", e))?;
    let req = String::from_utf8_lossy(&buf[..n]);

    let code = extract_query_param(&req, "code")
        .ok_or("No 'code' in callback. User may have cancelled.")?;
    let returned_state = extract_query_param(&req, "state").unwrap_or_default();
    if returned_state != state {
        return Err("State mismatch — possible CSRF attack".into());
    }

    // 6. Write response to browser
    let html = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<h2>Compass GDrive authorized</h2><p>You can close this tab.</p>";
    let _ = stream.write_all(html);
    let _ = stream.flush();

    // 7. Exchange code for tokens (secret + verifier)
    let body = format!(
        "client_id={}&client_secret={}&code={}&code_verifier={}&grant_type=authorization_code&redirect_uri={}",
        urlencode(CLIENT_ID),
        urlencode(CLIENT_SECRET),
        urlencode(&code),
        urlencode(&verifier),
        urlencode(&redirect),
    );
    let resp_result = ureq::post(TOKEN_URL)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body);
    let token_resp: Value = match resp_result {
        Ok(r) => r.into_json().map_err(|e| format!("JSON parse: {}", e))?,
        Err(ureq::Error::Status(code, r)) => {
            let body = r.into_string().unwrap_or_default();
            return Err(format!("Token exchange {} {}: {}", code, TOKEN_URL, body));
        }
        Err(e) => return Err(format!("Token exchange transport: {}", e)),
    };

    let access_token = token_resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("No access_token in response")?;
    let refresh_token = token_resp
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or("No refresh_token — ensure access_type=offline + prompt=consent")?;
    let expires_in = token_resp
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    let expires_at = now_secs() + expires_in;

    let token_file = json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_at": expires_at,
    });

    let path = token_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&path, serde_json::to_string_pretty(&token_file).unwrap())
        .map_err(|e| format!("Write token: {}", e))?;
    // Restrict perms to owner only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&path)
            .map_err(|e| format!("metadata: {}", e))?
            .permissions();
        perm.set_mode(0o600);
        fs::set_permissions(&path, perm).ok();
    }

    Ok(json!({
        "ok": true,
        "saved_to": path.to_string_lossy(),
    })
    .to_string())
}

// ────────────────────────────────────────────────────────────────────
// status — check if authenticated
// ────────────────────────────────────────────────────────────────────

fn status() -> Result<String, String> {
    let path = token_path();
    if !path.exists() {
        return Ok(json!({
            "authenticated": false,
            "reason": "No token file. Run: compass-cli gdrive auth"
        })
        .to_string());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
    let tok: Value = serde_json::from_str(&raw).map_err(|e| format!("parse: {}", e))?;
    let expires_at = tok.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0);
    let valid = expires_at > now_secs();
    Ok(json!({
        "authenticated": true,
        "access_token_valid": valid,
        "expires_at": expires_at,
    })
    .to_string())
}

// ────────────────────────────────────────────────────────────────────
// download — fetch file by id, export xlsx if Google Sheet
// ────────────────────────────────────────────────────────────────────

fn download(file_id: &str, output: Option<&str>) -> Result<String, String> {
    let access_token = ensure_access_token()?;
    // First: get file metadata to determine type
    let meta: Value = ureq::get(&format!(
        "https://www.googleapis.com/drive/v3/files/{}?fields=id,name,mimeType",
        file_id
    ))
    .set("Authorization", &format!("Bearer {}", access_token))
    .call()
    .map_err(|e| format!("Metadata fetch failed: {}", e))?
    .into_json()
    .map_err(|e| format!("Metadata JSON: {}", e))?;

    let name = meta
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("file");
    let mime = meta
        .get("mimeType")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Build URL based on type
    let url = if mime == "application/vnd.google-apps.spreadsheet" {
        // Google Sheet → export as xlsx
        format!(
            "https://www.googleapis.com/drive/v3/files/{}/export?mimeType={}",
            file_id,
            urlencode("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        )
    } else {
        // Regular file → download
        format!(
            "https://www.googleapis.com/drive/v3/files/{}?alt=media",
            file_id
        )
    };

    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let mut bytes: Vec<u8> = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read body: {}", e))?;

    // Output path
    let out_path = match output {
        Some(p) => p.to_string(),
        None => {
            let safe_name = name.replace('/', "_");
            format!("/tmp/compass-{}.xlsx", safe_name)
        }
    };
    fs::write(&out_path, &bytes).map_err(|e| format!("Write: {}", e))?;

    Ok(json!({
        "ok": true,
        "path": out_path,
        "size": bytes.len(),
        "name": name,
        "mime": mime,
    })
    .to_string())
}

fn ensure_access_token() -> Result<String, String> {
    let path = token_path();
    if !path.exists() {
        return Err("Not authenticated. Run: compass-cli gdrive auth".into());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("read: {}", e))?;
    let tok: Value = serde_json::from_str(&raw).map_err(|e| format!("parse: {}", e))?;
    let expires_at = tok.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0);

    if expires_at > now_secs() + 60 {
        // Still valid (with 60s grace)
        return Ok(tok
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or("No access_token")?
            .to_string());
    }

    // Refresh
    let refresh_token = tok
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or("No refresh_token")?;
    let body = format!(
        "client_id={}&client_secret={}&refresh_token={}&grant_type=refresh_token",
        urlencode(CLIENT_ID),
        urlencode(CLIENT_SECRET),
        urlencode(refresh_token),
    );
    let resp_result = ureq::post(TOKEN_URL)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body);
    let resp: Value = match resp_result {
        Ok(r) => r.into_json().map_err(|e| format!("JSON: {}", e))?,
        Err(ureq::Error::Status(code, r)) => {
            let b = r.into_string().unwrap_or_default();
            return Err(format!("Refresh {} {}: {}", code, TOKEN_URL, b));
        }
        Err(e) => return Err(format!("Refresh transport: {}", e)),
    };

    let new_access = resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Refresh: no access_token")?;
    let new_expires = resp
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    let new_expires_at = now_secs() + new_expires;

    let updated = json!({
        "access_token": new_access,
        "refresh_token": refresh_token,
        "expires_at": new_expires_at,
    });
    fs::write(&path, serde_json::to_string_pretty(&updated).unwrap()).ok();

    Ok(new_access.to_string())
}

// ────────────────────────────────────────────────────────────────────
// helpers
// ────────────────────────────────────────────────────────────────────

fn extract_file_id(url_or_id: &str) -> String {
    // Patterns:
    //   https://docs.google.com/spreadsheets/d/<ID>/edit#gid=...
    //   https://drive.google.com/file/d/<ID>/view
    //   https://drive.google.com/open?id=<ID>
    //   <ID> (raw)
    if let Some(idx) = url_or_id.find("/d/") {
        let rest = &url_or_id[idx + 3..];
        if let Some(end) = rest.find('/') {
            return rest[..end].to_string();
        }
        return rest.to_string();
    }
    if let Some(idx) = url_or_id.find("id=") {
        let rest = &url_or_id[idx + 3..];
        if let Some(end) = rest.find('&') {
            return rest[..end].to_string();
        }
        return rest.to_string();
    }
    url_or_id.to_string()
}

fn extract_query_param(request: &str, key: &str) -> Option<String> {
    // Parse first line: "GET /?code=...&state=... HTTP/1.1"
    let first = request.lines().next()?;
    let qs_start = first.find('?')? + 1;
    let qs_end = first[qs_start..]
        .find(' ')
        .map(|i| qs_start + i)
        .unwrap_or(first.len());
    let qs = &first[qs_start..qs_end];
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(urldecode(v));
            }
        }
    }
    None
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn urldecode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(h) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("00"),
                16,
            ) {
                out.push(h);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn random_string(n: usize) -> String {
    // Use OS /dev/urandom for PKCE verifier entropy.
    let mut bytes: Vec<u8> = vec![0; n];
    if let Ok(mut f) = fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut bytes);
    }
    const CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    bytes
        .iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect()
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
