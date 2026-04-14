use crate::helpers;
use serde_json::json;
use std::path::Path;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err("Usage: compass-cli progress <save|load|clear> <session-dir> [step] [data-json]".into());
    }

    match args[0].as_str() {
        "save" => cmd_save(args),
        "load" => cmd_load(args),
        "clear" => cmd_clear(args),
        _ => Err(format!("Unknown progress command: {}", args[0])),
    }
}

fn progress_path(session_dir: &str) -> std::path::PathBuf {
    Path::new(session_dir).join("progress.json")
}

fn cmd_save(args: &[String]) -> Result<String, String> {
    // args: ["save", <session-dir>, <step>, <data-json>]
    if args.len() < 4 {
        return Err("Usage: compass-cli progress save <session-dir> <step> <data-json>".into());
    }
    let session_dir = &args[1];
    let step = &args[2];
    let data_raw = &args[3];

    let data: serde_json::Value = serde_json::from_str(data_raw)
        .map_err(|e| format!("Invalid data JSON: {}", e))?;

    // Derive workflow from session dir name (last path component, strip timestamps/slugs)
    let workflow = Path::new(session_dir)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let saved_at = chrono_now();

    let payload = json!({
        "workflow": workflow,
        "step": step,
        "data": data,
        "saved_at": saved_at
    });

    let path = progress_path(session_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    helpers::write_json(&path, &payload)?;

    Ok(json!({
        "success": true,
        "path": path.to_string_lossy(),
        "step": step,
        "saved_at": saved_at
    })
    .to_string())
}

fn cmd_load(args: &[String]) -> Result<String, String> {
    // args: ["load", <session-dir>]
    if args.len() < 2 {
        return Err("Usage: compass-cli progress load <session-dir>".into());
    }
    let session_dir = &args[1];
    let path = progress_path(session_dir);

    if !path.exists() {
        return Ok(json!({"exists": false}).to_string());
    }

    let data = helpers::read_json(&path)?;
    Ok(serde_json::to_string_pretty(&data).unwrap())
}

fn cmd_clear(args: &[String]) -> Result<String, String> {
    // args: ["clear", <session-dir>]
    if args.len() < 2 {
        return Err("Usage: compass-cli progress clear <session-dir>".into());
    }
    let session_dir = &args[1];
    let path = progress_path(session_dir);

    if !path.exists() {
        return Ok(json!({"success": true, "existed": false}).to_string());
    }

    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(json!({"success": true, "existed": true, "cleared": path.to_string_lossy()}).to_string())
}

/// Returns a simple ISO-8601 timestamp without pulling in chrono crate.
/// Uses `date` system command for compatibility.
fn chrono_now() -> String {
    std::process::Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
