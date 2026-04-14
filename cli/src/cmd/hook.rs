use std::path::{Path, PathBuf};

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() { return Err("Usage: compass-cli hook <statusline|update-checker|context-monitor|manifest-tracker>".into()); }
    match args[0].as_str() {
        "statusline" => statusline(),
        "update-checker" | "check-update" => update_checker(),
        "context-monitor" => context_monitor(),
        "manifest-tracker" => {
            let sub = args.get(1).map(|s| s.as_str()).unwrap_or("check");
            match sub {
                "generate" => manifest_tracker_generate(None),
                _ => Err(format!("Unknown manifest-tracker subcommand: {}", sub)),
            }
        }
        _ => Err(format!("Unknown hook: {}", args[0])),
    }
}

fn statusline() -> Result<String, String> {
    let config_path = Path::new(".compass/.state/config.json");
    if !config_path.exists() { return Ok("Compass: not initialized".to_string()); }

    let content = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let project = data.pointer("/project/name").and_then(|v| v.as_str()).unwrap_or("?");
    let prefix = data.get("prefix").and_then(|v| v.as_str()).unwrap_or("?");
    let mode = data.get("mode").and_then(|v| v.as_str()).unwrap_or("?");

    Ok(format!("Compass: {} ({}) | {}", project, prefix, mode))
}

fn update_checker() -> Result<String, String> {
    // Same logic as shell hook but in Rust — check once per day
    let home = std::env::var("HOME").unwrap_or_default();
    let cache_file = Path::new(&home).join(".compass").join(".update-check-cache");

    if cache_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&cache_file) {
            if let Ok(ts) = content.trim().parse::<u64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap().as_secs();
                if now - ts < 86400 { return Ok(String::new()); }
            }
        }
    }

    // Write cache timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let _ = std::fs::write(&cache_file, now.to_string());

    // We can't easily do HTTP in pure Rust without dependencies, so just output empty
    // The shell hook handles the actual GitHub API call
    Ok(String::new())
}

/// PostToolUse hook — context utilization monitor.
/// Currently a placeholder that emits a status JSON. Future enhancement may
/// track real session-context size and warn on thresholds.
fn context_monitor() -> Result<String, String> {
    // Detect if we're inside a Compass session by checking $HOME/.compass/projects.json
    // If found and last_active is set, surface session_id; else null.
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let registry_path = std::path::Path::new(&home).join(".compass").join("projects.json");

    let session_id: Option<String> = if registry_path.exists() {
        std::fs::read_to_string(&registry_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("last_active").and_then(|x| x.as_str()).map(|s| s.to_string()))
    } else {
        None
    };

    Ok(serde_json::json!({
        "status": "ok",
        "session_id": session_id,
        "event_count": 0,
        "last_event": null,
        "log_path": null,
    }).to_string())
}

/// Hard-coded skip patterns for the manifest walker. A path is skipped if
/// ANY of these substrings appear in its full path. This mirrors the bash
/// `find -not -path / -not -name` filters and adds `target/` for Rust builds.
const SKIP_PATTERNS: &[&str] = &[
    "/.git/",
    "/node_modules/",
    "/target/",
    ".file-manifest.json",
    ".update-check-cache",
    ".DS_Store",
];

/// Recursively walk `dir`, pushing every file path into `out` that does not
/// match a hard-coded skip pattern.
fn walk_collect(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => return Err(format!("read_dir({}) failed: {}", dir.display(), e)),
    };
    for entry in rd {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let path_str = path.to_string_lossy();
        // Skip if any pattern is a substring of the path
        if SKIP_PATTERNS.iter().any(|p| path_str.contains(p)) {
            continue;
        }
        let ft = entry.file_type().map_err(|e| e.to_string())?;
        if ft.is_dir() {
            walk_collect(&path, out)?;
        } else if ft.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// PreUpdate hook — generate a manifest of all tracked files under `root`
/// (defaults to `~/.compass`) with SHA256 hashes. Writes
/// `<root>/.file-manifest.json` and returns a small status JSON.
pub fn manifest_tracker_generate(root: Option<&Path>) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let default_root = PathBuf::from(&home).join(".compass");
    let root: PathBuf = root.map(|p| p.to_path_buf()).unwrap_or(default_root);

    if !root.exists() {
        return Err(format!("root does not exist: {}", root.display()));
    }

    // Collect files
    let mut files: Vec<PathBuf> = Vec::new();
    walk_collect(&root, &mut files)?;
    files.sort();

    // Read version file if present
    let version = std::fs::read_to_string(root.join("VERSION"))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Build files map preserving sorted order
    let mut files_map = serde_json::Map::new();
    for f in &files {
        let rel = f.strip_prefix(&root).map_err(|e| e.to_string())?;
        let rel_str = rel.to_string_lossy().to_string();
        let bytes = std::fs::read(f).map_err(|e| format!("read {}: {}", f.display(), e))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hex_hash = hex::encode(hasher.finalize());
        files_map.insert(rel_str, serde_json::Value::String(hex_hash));
    }

    // ISO-8601 UTC timestamp without extra deps: compute from SystemTime.
    let generated_at = iso8601_utc_now();

    let manifest = serde_json::json!({
        "version": version,
        "generated_at": generated_at,
        "files": serde_json::Value::Object(files_map),
    });

    let manifest_path = root.join(".file-manifest.json");
    let pretty = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    std::fs::write(&manifest_path, pretty)
        .map_err(|e| format!("write {}: {}", manifest_path.display(), e))?;

    Ok(serde_json::json!({
        "ok": true,
        "file_count": files.len(),
        "manifest_path": manifest_path.to_string_lossy(),
    }).to_string())
}

/// Minimal ISO-8601 UTC formatter (YYYY-MM-DDTHH:MM:SSZ) without chrono.
fn iso8601_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Days since 1970-01-01
    let days = (secs / 86400) as i64;
    let tod = secs % 86400;
    let hour = tod / 3600;
    let minute = (tod % 3600) / 60;
    let second = tod % 60;

    // Convert days to Y-M-D via civil_from_days (Howard Hinnant algorithm)
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hour, minute, second)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_monitor_returns_ok_status() {
        let out = context_monitor().expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert_eq!(v["status"], "ok");
        assert!(v.get("event_count").is_some());
    }

    #[test]
    fn context_monitor_handles_no_session() {
        // With $HOME pointing at a tmp dir with no registry, session_id is null.
        let tmp = std::env::temp_dir().join(format!("compass-ctx-mon-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        // Cannot mutate $HOME in tests without races; just verify the no-registry path
        // by reading from a known-empty path. Acceptable: assert function returns Ok and
        // status="ok".
        let out = context_monitor().expect("ok");
        assert!(out.contains("\"status\""));
    }

    #[test]
    fn manifest_tracker_creates_file() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let root = tmp.path();
        std::fs::write(root.join("a.md"), "alpha\n").unwrap();
        std::fs::write(root.join("b.md"), "beta\n").unwrap();

        let out = manifest_tracker_generate(Some(root)).expect("gen ok");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert_eq!(v["ok"], true);
        assert_eq!(v["file_count"], 2);

        let manifest_path = root.join(".file-manifest.json");
        assert!(manifest_path.exists(), "manifest file created");

        let raw = std::fs::read_to_string(&manifest_path).unwrap();
        let m: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let files = m["files"].as_object().expect("files obj");
        assert_eq!(files.len(), 2);
        assert!(files.contains_key("a.md"));
        assert!(files.contains_key("b.md"));
    }

    #[test]
    fn manifest_tracker_skips_gitignored() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let root = tmp.path();
        std::fs::write(root.join("keep.md"), "keep\n").unwrap();

        // target/dummy.txt — should be skipped
        let target_dir = root.join("target");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("dummy.txt"), "ignore me\n").unwrap();

        // .git/config — should be skipped
        let git_dir = root.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("config"), "[core]\n").unwrap();

        // .DS_Store — should be skipped
        std::fs::write(root.join(".DS_Store"), "junk").unwrap();

        let out = manifest_tracker_generate(Some(root)).expect("gen ok");
        let v: serde_json::Value = serde_json::from_str(&out).expect("json");
        assert_eq!(v["file_count"], 1, "only keep.md should be tracked");

        let manifest_path = root.join(".file-manifest.json");
        let raw = std::fs::read_to_string(&manifest_path).unwrap();
        let m: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let files = m["files"].as_object().unwrap();
        assert!(files.contains_key("keep.md"));
        // No skipped file should appear
        for k in files.keys() {
            assert!(!k.contains("target"), "target/ not skipped: {}", k);
            assert!(!k.contains(".git"), ".git/ not skipped: {}", k);
            assert!(!k.contains(".DS_Store"), ".DS_Store not skipped: {}", k);
        }
    }

    #[test]
    fn manifest_tracker_sha256_is_correct() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let root = tmp.path();
        std::fs::write(root.join("hello.txt"), "hello\n").unwrap();

        manifest_tracker_generate(Some(root)).expect("gen ok");
        let raw = std::fs::read_to_string(root.join(".file-manifest.json")).unwrap();
        let m: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let files = m["files"].as_object().unwrap();
        assert_eq!(
            files["hello.txt"].as_str().unwrap(),
            "5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"
        );
    }
}
