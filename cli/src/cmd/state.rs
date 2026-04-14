use crate::helpers;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const CONFIG_CACHE_TTL: u64 = 300;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() { return Err("Usage: compass-cli state <get|update|get-config> <dir> [json]".into()); }

    match args[0].as_str() {
        "get" => {
            if args.len() < 2 { return Err("Usage: compass-cli state get <dir>".into()); }
            let state_path = resolve_state_path(&args[1]);
            let data = helpers::read_json(&state_path)?;
            Ok(serde_json::to_string_pretty(&data).unwrap())
        }
        "update" => {
            if args.len() < 3 { return Err("Usage: compass-cli state update <dir> <json>".into()); }
            let state_path = resolve_state_path(&args[1]);
            let mut data = if state_path.exists() {
                helpers::read_json(&state_path)?
            } else {
                json!({})
            };
            let patch: serde_json::Value = serde_json::from_str(&args[2])
                .map_err(|e| format!("Invalid JSON patch: {}", e))?;
            if let (Some(obj), Some(patch_obj)) = (data.as_object_mut(), patch.as_object()) {
                for (k, v) in patch_obj { obj.insert(k.clone(), v.clone()); }
                obj.insert("_updated_at".to_string(), json!(chrono_now()));
            }
            helpers::write_json(&state_path, &data)?;
            Ok(serde_json::to_string_pretty(&json!({"success": true, "state": data})).unwrap())
        }
        "get-config" => {
            // Legacy form: `state get-config <dir> [--no-cache]` — kept for back-compat.
            // New form (no positional dir): forward to `project resolve`.
            let positional_dir = args
                .iter()
                .skip(1)
                .find(|a| !a.starts_with("--"))
                .map(|s| s.as_str());
            let no_cache = args.iter().any(|a| a == "--no-cache");
            match positional_dir {
                Some(dir) => {
                    eprintln!(
                        "DEPRECATED: 'compass-cli state get-config <dir>' is deprecated. \
                         Use 'compass-cli project resolve' instead."
                    );
                    get_config(dir, no_cache)
                }
                None => get_config_via_project_resolve(),
            }
        }
        _ => Err(format!("Unknown state command: {}", args[0])),
    }
}

fn resolve_state_path(dir: &str) -> PathBuf {
    let primary = Path::new(dir).join(".state").join("state.json");
    if primary.exists() { primary } else { Path::new(dir).join("state.json") }
}

/// New-style `state get-config` (no positional dir): delegate to
/// `compass-cli project resolve` and, on `status=ok`, return only the `config`
/// object as pretty JSON — preserving the v1.1 API shape that callers expect.
/// On `status=ambiguous` or `status=none`, return an Err pointing the user to
/// `project resolve` for full status info (exit 1 semantics via main dispatcher).
fn get_config_via_project_resolve() -> Result<String, String> {
    let raw = crate::cmd::project::run(&["resolve".to_string()])?;
    let val: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse project resolve output: {}", e))?;

    let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
    match status {
        "ok" => {
            let config = val.get("config").cloned().unwrap_or(json!({}));
            Ok(serde_json::to_string_pretty(&config).unwrap())
        }
        "ambiguous" => Err(
            "Multiple Compass projects registered and none is active. \
             Run 'compass-cli project resolve' for full status, \
             then 'compass-cli project use <path>' to pick one."
                .to_string(),
        ),
        "none" => Err(
            "No Compass project found. \
             Run 'compass-cli project resolve' for details, \
             or '/compass:init' to create one."
                .to_string(),
        ),
        other => Err(format!(
            "Unexpected status '{}' from project resolve. \
             Run 'compass-cli project resolve' directly to inspect.",
            other
        )),
    }
}

fn get_config(dir: &str, no_cache: bool) -> Result<String, String> {
    let config_path = Path::new(dir).join(".compass").join(".state").join("config.json");

    if !config_path.exists() {
        return Err(format!("Config not found at {}. Run /compass:init first.", config_path.display()));
    }

    let cache_path = cache_path_for(&config_path);

    if !no_cache {
        if let Some(cached) = read_cache_if_fresh(&cache_path, &config_path) {
            return Ok(cached);
        }
    }

    let data = helpers::read_json(&config_path)?;
    let serialized = serde_json::to_string_pretty(&data).unwrap();

    let _ = fs::write(&cache_path, &serialized);

    Ok(serialized)
}

fn cache_path_for(config_path: &Path) -> PathBuf {
    let abs = fs::canonicalize(config_path).unwrap_or_else(|_| config_path.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(abs.to_string_lossy().as_bytes());
    let hash = hex::encode(hasher.finalize());
    let short = &hash[..16];
    std::env::temp_dir().join(format!("compass-config-{}.json", short))
}

fn read_cache_if_fresh(cache_path: &Path, source_path: &Path) -> Option<String> {
    let cache_meta = fs::metadata(cache_path).ok()?;
    let cache_mtime = cache_meta.modified().ok()?;
    let age = SystemTime::now().duration_since(cache_mtime).ok()?;

    if age >= Duration::from_secs(CONFIG_CACHE_TTL) {
        return None;
    }

    if let Ok(src_meta) = fs::metadata(source_path) {
        if let Ok(src_mtime) = src_meta.modified() {
            if src_mtime > cache_mtime {
                return None;
            }
        }
    }

    fs::read_to_string(cache_path).ok()
}

fn chrono_now() -> String {
    let d = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    format!("{}Z", d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::UNIX_EPOCH;

    // Shared guard with project.rs — $HOME is process-global, every module
    // that mutates it MUST hold the same Mutex or they race each other.
    use crate::cmd::project::test_support::HOME_GUARD;
    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_tmp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!(
                "compass-cli-statetest-{}-{}-{}-{}",
                tag, pid, nanos, n
        ));
        fs::create_dir_all(&dir).expect("create unique tmp dir");
        dir
    }

    struct HomeGuard {
        home: PathBuf,
        prev_home: Option<std::ffi::OsString>,
        prev_cwd: Option<PathBuf>,
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl HomeGuard {
        fn new(tag: &str) -> Self {
            let lock = HOME_GUARD.lock().unwrap_or_else(|e| e.into_inner());
            let home = unique_tmp_dir(tag);
            let prev_home = std::env::var_os("HOME");
            let prev_cwd = std::env::current_dir().ok();
            std::env::set_var("HOME", &home);
            fs::create_dir_all(home.join(".compass")).expect("mkdir ~/.compass");
            HomeGuard { home, prev_home, prev_cwd, _lock: lock }
        }

        fn registry_file(&self) -> PathBuf {
            self.home.join(".compass").join("projects.json")
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            if let Some(p) = &self.prev_cwd {
                let _ = std::env::set_current_dir(p);
            }
            match &self.prev_home {
                Some(p) => std::env::set_var("HOME", p),
                None => std::env::remove_var("HOME"),
            }
            let _ = fs::remove_dir_all(&self.home);
        }
    }

    fn make_project(parent: &Path, name: &str) -> PathBuf {
        let root = parent.join(name);
        let state = root.join(".compass").join(".state");
        fs::create_dir_all(&state).expect("create state dir");
        let config = json!({
            "version": "1.1.1",
            "project": {"name": name, "po": "@test"},
            "lang": "en",
        });
        fs::write(state.join("config.json"), config.to_string())
            .expect("write config.json");
        fs::canonicalize(&root).unwrap_or(root)
    }

    /// REQ-02: `state get-config` with NO dir arg forwards to
    /// `project resolve` and returns the resolved project's config.
    #[test]
    fn get_config_forwards_to_resolve() {
        let g = HomeGuard::new("get_config_fwd");
        let parent = unique_tmp_dir("get_config_fwd_parent");
        let root = make_project(&parent, "proj_fwd");
        let root_str = root.to_string_lossy().to_string();

        let reg = json!({
            "version": "1.0",
            "last_active": root_str,
            "projects": [{
                "path": root_str,
                "name": "proj_fwd",
                "created_at": "2026-04-01T00:00:00Z",
                "last_used": "2026-04-01T00:00:00Z",
            }],
        });
        fs::write(g.registry_file(), reg.to_string())
            .expect("write registry");

        let out = run(&["get-config".to_string()]).expect("get-config (no dir) ok");
        // Must return the project's config JSON — contains `project` and `lang`.
        assert!(
            out.contains("\"project\"") && out.contains("\"lang\""),
            "expected forwarded config to contain project+lang fields; got:\n{}",
            out
        );
        let v: serde_json::Value =
            serde_json::from_str(&out).expect("returned body is valid JSON");
        assert_eq!(v["project"]["name"], json!("proj_fwd"));
        assert_eq!(v["lang"], json!("en"));

        let _ = fs::remove_dir_all(&parent);
    }

    /// REQ-13: `state get-config <dir>` is the legacy alias. It must
    /// still work (back-compat) and emit a DEPRECATED warning. We
    /// verify the legacy branch executes by asserting it returns the
    /// legacy-dir config (a path not reachable via the forwarded
    /// `project resolve` flow when no registry is present).
    #[test]
    fn get_config_arg_warns() {
        let _g = HomeGuard::new("get_config_arg_warns");
        // Build a legacy-shape project dir: <dir>/.compass/.state/config.json.
        let tmp = unique_tmp_dir("get_config_arg_warns_proj");
        let state_dir = tmp.join(".compass").join(".state");
        fs::create_dir_all(&state_dir).expect("create legacy state dir");
        let legacy_cfg = json!({
            "version": "1.1.1",
            "project": {"name": "legacy_proj", "po": "@legacy"},
            "legacy_marker": true,
        });
        fs::write(state_dir.join("config.json"), legacy_cfg.to_string())
            .expect("write legacy config.json");

        let dir_str = tmp.to_string_lossy().to_string();
        // --no-cache ensures the legacy branch actually reads the file
        // (not a stale cache from a prior run in the shared tmp).
        let out = run(&[
            "get-config".to_string(),
            dir_str,
            "--no-cache".to_string(),
        ])
        .expect("legacy get-config <dir> ok");

        // Deprecation branch executed iff it returned the legacy config
        // (note: `project resolve` path would not have `legacy_marker`).
        let v: serde_json::Value =
            serde_json::from_str(&out).expect("legacy output is JSON");
        assert_eq!(v["legacy_marker"], json!(true));
        assert_eq!(v["project"]["name"], json!("legacy_proj"));

        // Stderr capture is not feasible in an in-process unit test without
        // platform-specific fd dup; integration tests cover the DEPRECATED
        // string itself. The legacy_marker assertion above proves the
        // deprecated branch ran (the `eprintln!("DEPRECATED: ...")` fires
        // immediately before `get_config(dir, no_cache)` is called).

        let _ = fs::remove_dir_all(&tmp);
    }
}
