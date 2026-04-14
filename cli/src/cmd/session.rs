use serde_json::json;
use std::path::Path;
use std::fs;
use std::cmp::Reverse;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() { return Err("Usage: compass-cli session <latest|list> [dir]".into()); }
    let sessions_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        ".compass/.state/sessions".to_string()
    };

    match args[0].as_str() {
        "latest" => session_latest(Path::new(&sessions_dir)),
        "list" => session_list(Path::new(&sessions_dir)),
        _ => Err(format!("Unknown session command: {}", args[0])),
    }
}

fn session_latest(dir: &Path) -> Result<String, String> {
    if !dir.exists() {
        return Ok(serde_json::to_string_pretty(&json!({"found": false})).unwrap());
    }
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    match entries.first() {
        Some(entry) => {
            let p = entry.path();
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let files: Vec<String> = fs::read_dir(&p)
                .map(|rd| rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect())
                .unwrap_or_default();
            Ok(serde_json::to_string_pretty(&json!({
                "found": true,
                "name": name,
                "dir": p.to_string_lossy(),
                "files": files,
            })).unwrap())
        }
        None => Ok(serde_json::to_string_pretty(&json!({"found": false})).unwrap()),
    }
}

fn session_list(dir: &Path) -> Result<String, String> {
    if !dir.exists() { return Ok(json!({"sessions": []}).to_string()); }
    let sessions: Vec<serde_json::Value> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| json!({
            "name": e.file_name().to_string_lossy(),
            "path": e.path().to_string_lossy(),
        }))
        .collect();
    Ok(serde_json::to_string_pretty(&json!({"sessions": sessions})).unwrap())
}
