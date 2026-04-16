use fs2::FileExt;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const REGISTRY_VERSION: &str = "1.0";
const GLOBAL_CONFIG_VERSION: &str = "1.0";

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(usage());
    }

    match args[0].as_str() {
        "--help" | "-h" | "help" => Ok(usage()),
        "resolve" => resolve(),
        "list" => list(),
        "use" => {
            let path = args
                .get(1)
                .ok_or_else(|| "Usage: compass-cli project use <path>".to_string())?;
            use_project(path)
        }
        "add" => {
            let path = args
                .get(1)
                .ok_or_else(|| "Usage: compass-cli project add <path>".to_string())?;
            add(path)
        }
        "remove" => {
            let path = args
                .get(1)
                .ok_or_else(|| "Usage: compass-cli project remove <path>".to_string())?;
            remove(path)
        }
        "global-config" => global_config(&args[1..]),
        "gate" => {
            let args_text = parse_flag(&args[1..], "--args").ok_or_else(|| {
                "MISSING_FLAG: --args is required (Usage: compass-cli project gate --args <text> --artifact-type <type>)".to_string()
            })?;
            let artifact_type = parse_flag(&args[1..], "--artifact-type").ok_or_else(|| {
                "MISSING_FLAG: --artifact-type is required (Usage: compass-cli project gate --args <text> --artifact-type <type>)".to_string()
            })?;
            gate(&args_text, &artifact_type)
        }
        other => Err(format!("Unknown project command: {}. Run 'compass-cli project --help' for usage.", other)),
    }
}

fn usage() -> String {
    "Usage: compass-cli project <resolve|list|use|add|remove|global-config|gate> [...]".into()
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

// ---------------------------------------------------------------------------
// Paths & timestamp helpers
// ---------------------------------------------------------------------------

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn compass_dir() -> PathBuf {
    home_dir().join(".compass")
}

fn registry_path() -> PathBuf {
    compass_dir().join("projects.json")
}

fn global_config_path() -> PathBuf {
    compass_dir().join("global-config.json")
}

fn ensure_compass_dir() -> Result<(), String> {
    let dir = compass_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Cannot create {}: {}", dir.display(), e))?;
    // Registry + global-config hold user-level state (list of projects, prefs).
    // Tighten perms to owner-only so the file is not world-readable on shared
    // hosts. Unix-only; Windows ignores the mode.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
    }
    Ok(())
}

fn project_config_path(project_root: &Path) -> PathBuf {
    project_root
        .join(".compass")
        .join(".state")
        .join("config.json")
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_iso_utc(secs)
}

fn format_iso_utc(total_secs: u64) -> String {
    let days = (total_secs / 86_400) as i64;
    let tod = total_secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    let hh = tod / 3600;
    let mm = (tod % 3600) / 60;
    let ss = tod % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hh, mm, ss
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

/// Canonicalize a user-supplied path. Falls back when `canonicalize` fails
/// (e.g. path does not exist yet): walk up ancestors finding the nearest
/// existing parent, canonicalize THAT, then re-attach the missing tail. This
/// resolves `..` components and symlinks even for not-yet-created paths, so
/// registry entries are stable regardless of how the user spelled the path
/// (e.g. macOS `/var` vs `/private/var`).
fn canonicalize_path(p: &str) -> PathBuf {
    let raw = Path::new(p);
    if let Ok(c) = fs::canonicalize(raw) {
        return strip_trailing_slash(&c);
    }
    // Fallback: find nearest existing ancestor, canonicalize it, append tail.
    let base = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|c| c.join(raw))
            .unwrap_or_else(|_| raw.to_path_buf())
    };
    let mut ancestor = base.clone();
    let mut tail_components: Vec<std::ffi::OsString> = Vec::new();
    while !ancestor.exists() {
        match (ancestor.parent(), ancestor.file_name()) {
            (Some(parent), Some(name)) => {
                tail_components.push(name.to_os_string());
                ancestor = parent.to_path_buf();
            }
            _ => break, // hit root
        }
    }
    let canonical_base = fs::canonicalize(&ancestor).unwrap_or(ancestor);
    let mut out = canonical_base;
    for comp in tail_components.iter().rev() {
        // Manually resolve `..` vs real names on the synthesized tail.
        if comp == std::ffi::OsStr::new("..") {
            if let Some(p) = out.parent() {
                out = p.to_path_buf();
            }
        } else if comp != std::ffi::OsStr::new(".") {
            out.push(comp);
        }
    }
    strip_trailing_slash(&out)
}

fn strip_trailing_slash(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        out.push(comp.as_os_str());
    }
    out
}

fn cwd_string() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

// ---------------------------------------------------------------------------
// Registry I/O
// ---------------------------------------------------------------------------

/// Load the registry, handling three cases:
/// - file missing → empty registry
/// - file present but unparseable → back up to `.bak`, reset to empty, warn stderr
/// - file present and parseable → return it
fn load_registry_or_reset() -> Value {
    let path = registry_path();
    if !path.exists() {
        return empty_registry();
    }
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warn: cannot read {}: {}", path.display(), e);
            return empty_registry();
        }
    };
    match serde_json::from_str::<Value>(&raw) {
        Ok(v) => {
            if v.is_object() {
                v
            } else {
                eprintln!(
                    "warn: {} is not a JSON object; resetting",
                    path.display()
                );
                backup_and_reset(&path, &raw);
                empty_registry()
            }
        }
        Err(e) => {
            eprintln!(
                "warn: CORRUPT_REGISTRY at {} ({}); backing up to .bak and resetting",
                path.display(),
                e
            );
            backup_and_reset(&path, &raw);
            empty_registry()
        }
    }
}

fn backup_and_reset(path: &Path, raw: &str) {
    let bak = path.with_extension("json.bak");
    let _ = fs::write(&bak, raw);
    let _ = fs::write(path, empty_registry().to_string());
}

fn empty_registry() -> Value {
    json!({
        "version": REGISTRY_VERSION,
        "last_active": Value::Null,
        "projects": [],
    })
}

/// Write the registry atomically (tmp + rename) under an exclusive file lock.
/// Creates `~/.compass/` if missing.
fn write_registry(value: &Value) -> Result<(), String> {
    ensure_compass_dir()?;
    let path = registry_path();

    // Touch-ensure the target file exists so we can lock it.
    if !path.exists() {
        fs::write(&path, "{}")
            .map_err(|e| format!("Cannot create {}: {}", path.display(), e))?;
    }

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    file.lock_exclusive()
        .map_err(|e| format!("Cannot lock {}: {}", path.display(), e))?;

    let result = (|| -> Result<(), String> {
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_string_pretty(value)
            .map_err(|e| format!("JSON serialize error: {}", e))?;
        fs::write(&tmp, &body)
            .map_err(|e| format!("Cannot write {}: {}", tmp.display(), e))?;
        fs::rename(&tmp, &path)
            .map_err(|e| format!("Cannot rename {} → {}: {}", tmp.display(), path.display(), e))?;
        Ok(())
    })();

    let _ = file.unlock();
    result
}

/// Sort `projects[]` by `last_used` desc in-place.
fn sort_projects_desc(reg: &mut Value) {
    if let Some(arr) = reg
        .get_mut("projects")
        .and_then(|v| v.as_array_mut())
    {
        arr.sort_by(|a, b| {
            let la = a.get("last_used").and_then(|v| v.as_str()).unwrap_or("");
            let lb = b.get("last_used").and_then(|v| v.as_str()).unwrap_or("");
            lb.cmp(la)
        });
    }
}

fn projects_array(reg: &Value) -> Vec<Value> {
    reg.get("projects")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

fn set_projects(reg: &mut Value, projects: Vec<Value>) {
    if let Some(obj) = reg.as_object_mut() {
        obj.insert("projects".to_string(), Value::Array(projects));
    }
}

fn last_active(reg: &Value) -> Option<String> {
    reg.get("last_active")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn set_last_active(reg: &mut Value, value: Option<&str>) {
    if let Some(obj) = reg.as_object_mut() {
        obj.insert(
            "last_active".to_string(),
            match value {
                Some(s) => Value::String(s.to_string()),
                None => Value::Null,
            },
        );
    }
}

fn is_alive(project_path: &str) -> bool {
    project_config_path(Path::new(project_path)).exists()
}

fn prune_dead(reg: &mut Value) -> Vec<String> {
    let projects = projects_array(reg);
    let mut alive = Vec::new();
    let mut dead = Vec::new();
    for p in projects {
        let path = p.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if path.is_empty() {
            continue;
        }
        if is_alive(path) {
            alive.push(p);
        } else {
            dead.push(path.to_string());
        }
    }
    set_projects(reg, alive);

    // If last_active references a dead entry, clear it.
    if let Some(la) = last_active(reg) {
        if dead.iter().any(|d| d == &la) || !is_alive(&la) {
            set_last_active(reg, None);
        }
    }

    dead
}

// ---------------------------------------------------------------------------
// `resolve`
// ---------------------------------------------------------------------------

fn resolve() -> Result<String, String> {
    let cwd = cwd_string();
    let mut reg = load_registry_or_reset();
    let mut migrated = false;
    let mut dirty = false;

    // If registry empty and cwd is a compass project → auto-migrate (REQ-10).
    let starting_empty = projects_array(&reg).is_empty();
    if starting_empty {
        let cwd_path = Path::new(&cwd);
        if project_config_path(cwd_path).exists() {
            let abs = canonicalize_path(&cwd);
            let abs_str = abs.to_string_lossy().to_string();
            let name = read_project_name(&abs).unwrap_or_else(|| "(unknown)".to_string());
            let now = now_iso();
            let entry = json!({
                "path": abs_str,
                "name": name,
                "created_at": now,
                "last_used": now,
            });
            let mut projects = projects_array(&reg);
            projects.push(entry);
            set_projects(&mut reg, projects);
            set_last_active(&mut reg, Some(&abs_str));
            migrated = true;
            dirty = true;
        } else {
            // Empty and no local config → status: none.
            return Ok(json!({
                "status": "none",
                "reason": "empty_registry",
                "cwd": cwd,
            })
            .to_string());
        }
    }

    // Prune dead entries (REQ-11).
    let dead = prune_dead(&mut reg);
    if !dead.is_empty() {
        dirty = true;
    }

    let remaining = projects_array(&reg);
    if remaining.is_empty() {
        if dirty {
            let _ = write_registry(&reg);
        }
        return Ok(json!({
            "status": "none",
            "reason": "all_paths_dead",
            "cwd": cwd,
        })
        .to_string());
    }

    // Decide active.
    let active_path = match last_active(&reg) {
        Some(p) if is_alive(&p) => Some(p),
        _ => {
            // last_active was pruned or never set.
            if remaining.len() == 1 {
                let only = remaining[0]
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(p) = &only {
                    set_last_active(&mut reg, Some(p));
                    dirty = true;
                }
                only
            } else {
                None
            }
        }
    };

    // If cwd has a compass config but isn't registered, warn the user on
    // stderr. Avoid auto-add to keep resolve side-effect-free for this path
    // (migration auto-add only fires when the registry was EMPTY above).
    if !migrated {
        let cwd_path = Path::new(&cwd);
        if project_config_path(cwd_path).exists() {
            let cwd_abs = canonicalize_path(&cwd);
            let cwd_abs_str = cwd_abs.to_string_lossy().to_string();
            let already_registered = projects_array(&reg).iter().any(|p| {
                p.get("path").and_then(|v| v.as_str()) == Some(cwd_abs_str.as_str())
            });
            if !already_registered {
                eprintln!(
                    "note: cwd has an unregistered Compass project at {}. \
                     Run 'compass-cli project add {}' to register it, or \
                     'compass-cli project use {}' to switch.",
                    cwd_abs_str, cwd_abs_str, cwd_abs_str
                );
            }
        }
    }

    match active_path {
        Some(active) => {
            // Bump last_used for the active entry.
            let now = now_iso();
            {
                let projects = projects_array(&reg);
                let updated: Vec<Value> = projects
                    .into_iter()
                    .map(|mut p| {
                        if p.get("path").and_then(|v| v.as_str()) == Some(active.as_str()) {
                            if let Some(obj) = p.as_object_mut() {
                                obj.insert("last_used".to_string(), json!(now));
                            }
                        }
                        p
                    })
                    .collect();
                set_projects(&mut reg, updated);
            }
            sort_projects_desc(&mut reg);
            let _ = write_registry(&reg);

            // Load the project config.
            let config_path = project_config_path(Path::new(&active));
            let config_val = match fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            {
                Some(v) => v,
                None => {
                    // Alive check passed earlier but file disappeared; fall back to empty object.
                    json!({})
                }
            };
            let name = config_val
                .get("project")
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)")
                .to_string();

            let shared_root = compute_shared_root(Path::new(&active));
            Ok(json!({
                "status": "ok",
                "project_root": active,
                "shared_root": shared_root,
                "name": name,
                "config": config_val,
                "migrated_from_v11": migrated,
            })
            .to_string())
        }
        None => {
            // Ambiguous: >= 2 alive, no last_active.
            sort_projects_desc(&mut reg);
            if dirty {
                let _ = write_registry(&reg);
            }
            let candidates: Vec<Value> = projects_array(&reg)
                .into_iter()
                .map(|p| {
                    let path_str = p.get("path").and_then(|v| v.as_str()).unwrap_or("");
                    let shared = compute_shared_root(Path::new(path_str));
                    json!({
                        "path": p.get("path").cloned().unwrap_or(Value::Null),
                        "name": p.get("name").cloned().unwrap_or(Value::Null),
                        "last_used": p.get("last_used").cloned().unwrap_or(Value::Null),
                        "shared_root": shared,
                    })
                })
                .collect();
            Ok(json!({
                "status": "ambiguous",
                "candidates": candidates,
                "cwd": cwd,
            })
            .to_string())
        }
    }
}

/// Compute `shared_root` = absolute path to `$PARENT/shared/` where
/// `$PARENT = dirname(project_root)`, ONLY if that directory exists.
/// Returns None if the sibling `shared/` directory does not exist.
fn compute_shared_root(project_root: &Path) -> Option<String> {
    let parent = project_root.parent()?;
    let candidate = parent.join("shared");
    if candidate.is_dir() {
        let canon = fs::canonicalize(&candidate).unwrap_or(candidate);
        Some(canon.to_string_lossy().to_string())
    } else {
        None
    }
}

fn read_project_name(project_root: &Path) -> Option<String> {
    let p = project_config_path(project_root);
    let raw = fs::read_to_string(&p).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    v.get("project")
        .and_then(|o| o.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

// ---------------------------------------------------------------------------
// `gate` — pipeline + project choice gate (Step 0d port)
// ---------------------------------------------------------------------------

/// Stopwords filtered out before computing Jaccard overlap. Locked per
/// DESIGN-SPEC; do not extend without a version bump.
const GATE_STOPWORDS: &[&str] = &[
    "a", "an", "the", "for", "and", "or", "of", "in", "on", "to",
    "app", "new", "old",
];

/// Tokenize a string for Jaccard scoring:
/// - split on whitespace and any non-alphanumeric character (punctuation)
/// - lowercase each token
/// - drop empty tokens and stopwords
fn tokenize_for_jaccard(s: &str) -> std::collections::HashSet<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .map(|t| t.to_lowercase())
        .filter(|t| !t.is_empty() && !GATE_STOPWORDS.contains(&t.as_str()))
        .collect()
}

/// Jaccard similarity: |A ∩ B| / |A ∪ B|. Returns 0.0 when both sets are
/// empty after stopword filtering (no signal).
fn jaccard(a: &str, b: &str) -> f32 {
    let set_a = tokenize_for_jaccard(a);
    let set_b = tokenize_for_jaccard(b);
    if set_a.is_empty() && set_b.is_empty() {
        return 0.0;
    }
    let inter = set_a.intersection(&set_b).count() as f32;
    let union = set_a.union(&set_b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

/// Parse an ISO-8601 UTC timestamp (format emitted by `now_iso` /
/// `format_iso_utc`: `YYYY-MM-DDTHH:MM:SSZ`) back to epoch seconds. Returns
/// None on any parse failure — caller should treat as "unknown age".
fn parse_iso_utc_to_epoch(s: &str) -> Option<u64> {
    // Minimal, dependency-free parser tolerant of fractional seconds and
    // trailing 'Z'. Expect: YYYY-MM-DDTHH:MM:SS[.fff]Z or without Z.
    let bytes = s.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let year: i64 = s.get(0..4)?.parse().ok()?;
    if bytes[4] != b'-' {
        return None;
    }
    let month: u32 = s.get(5..7)?.parse().ok()?;
    if bytes[7] != b'-' {
        return None;
    }
    let day: u32 = s.get(8..10)?.parse().ok()?;
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    let hour: u64 = s.get(11..13)?.parse().ok()?;
    if bytes[13] != b':' {
        return None;
    }
    let minute: u64 = s.get(14..16)?.parse().ok()?;
    if bytes[16] != b':' {
        return None;
    }
    let second: u64 = s.get(17..19)?.parse().ok()?;

    // Compute days since 1970-01-01 using the civil_to_days inverse of
    // civil_from_days (Howard Hinnant algorithm).
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = month as u64;
    let d = day as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe as i64 - 719_468;
    if days < 0 {
        return None;
    }
    Some(days as u64 * 86_400 + hour * 3600 + minute * 60 + second)
}

fn gate(args_text: &str, _artifact_type: &str) -> Result<String, String> {
    // Step 1: resolve first; propagate error on non-ok status.
    let resolve_out = resolve()?;
    let resolve_val: Value = serde_json::from_str(&resolve_out)
        .map_err(|e| format!("internal: resolve() returned invalid JSON: {}", e))?;

    let status = resolve_val.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if status != "ok" {
        return Err(format!(
            "RESOLVE_FAILED: status={}; resolve={}",
            status, resolve_val
        ));
    }

    let project_root = resolve_val
        .get("project_root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let project_name = resolve_val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("(unknown)")
        .to_string();

    // Step 2: scan sessions/*/pipeline.json for "status": "active".
    let sessions_dir = Path::new(&project_root)
        .join(".compass")
        .join(".state")
        .join("sessions");

    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut active: Vec<(String, String, String, u32, u32, bool, f32)> = Vec::new();
    // Tuple: (slug, title, created_at, artifacts_count, age_days, stale, relevance)

    if let Ok(read_dir) = fs::read_dir(&sessions_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let pipeline_path = path.join("pipeline.json");
            let pipeline_raw = match fs::read_to_string(&pipeline_path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let pipeline_val: Value = match serde_json::from_str(&pipeline_raw) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let is_active = pipeline_val
                .get("status")
                .and_then(|v| v.as_str())
                .map(|s| s == "active")
                .unwrap_or(false);
            if !is_active {
                continue;
            }

            let slug = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let context_val: Value = fs::read_to_string(path.join("context.json"))
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_else(|| json!({}));
            let title = context_val
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let created_at = pipeline_val
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let age_days: u32 = parse_iso_utc_to_epoch(&created_at)
                .map(|created| {
                    if now_secs > created {
                        ((now_secs - created) / 86_400) as u32
                    } else {
                        0
                    }
                })
                .unwrap_or(0);

            let artifacts_count: u32 = pipeline_val
                .get("artifacts")
                .and_then(|v| v.as_array())
                .map(|a| a.len() as u32)
                .unwrap_or(0);

            let stale = age_days > 14 && artifacts_count == 0;
            let relevance = jaccard(args_text, &title);

            active.push((slug, title, created_at, artifacts_count, age_days, stale, relevance));
        }
    }

    // Sort by relevance desc; tie-break by created_at desc (more recent first).
    active.sort_by(|a, b| {
        b.6.partial_cmp(&a.6)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.2.cmp(&a.2))
    });

    let pipeline_count = active.len();
    let top_relevance = active.first().map(|t| t.6).unwrap_or(0.0);
    let top_slug = active.first().map(|t| t.0.clone()).unwrap_or_default();

    // Build active_pipelines JSON array.
    let active_pipelines: Vec<Value> = active
        .iter()
        .map(|(slug, title, created_at, artifacts_count, age_days, stale, relevance)| {
            json!({
                "slug": slug,
                "title": title,
                "created_at": created_at,
                "artifacts_count": artifacts_count,
                "age_days": age_days,
                "stale": stale,
                "relevance": relevance,
            })
        })
        .collect();

    // Step 4: select case + suggested_action.
    let (case, suggested_action) = if pipeline_count == 0 {
        (3u8, "current_project".to_string())
    } else if pipeline_count == 1 {
        if top_relevance >= 0.2 {
            (1u8, format!("continue:{}", top_slug))
        } else {
            (2u8, "standalone".to_string())
        }
    } else {
        // pipeline_count >= 2
        (4u8, format!("continue:{}", top_slug))
    };

    // Step 5: build options_spec per case.
    let top_stale = active.first().map(|t| t.5).unwrap_or(false);
    let options_spec: Vec<Value> = match case {
        1 => vec![
            json!({
                "id": format!("continue:{}", top_slug),
                "pipeline_slug": top_slug,
                "stale_warning": top_stale,
            }),
            json!({ "id": "standalone", "pipeline_slug": Value::Null, "stale_warning": false }),
            json!({ "id": "other_project", "pipeline_slug": Value::Null, "stale_warning": false }),
        ],
        2 => vec![
            json!({ "id": "standalone", "pipeline_slug": Value::Null, "stale_warning": false }),
            json!({ "id": "other_project", "pipeline_slug": Value::Null, "stale_warning": false }),
            json!({
                "id": "close_first",
                "pipeline_slug": top_slug,
                "stale_warning": top_stale,
            }),
        ],
        3 => vec![
            json!({ "id": "current_project", "pipeline_slug": Value::Null, "stale_warning": false }),
            json!({ "id": "other_project", "pipeline_slug": Value::Null, "stale_warning": false }),
        ],
        4 => {
            let mut opts: Vec<Value> = Vec::with_capacity(5 + pipeline_count);
            opts.push(json!({
                "id": format!("continue:{}", top_slug),
                "pipeline_slug": top_slug,
                "stale_warning": top_stale,
            }));
            opts.push(json!({ "id": "pick_pipeline", "pipeline_slug": Value::Null, "stale_warning": false }));
            opts.push(json!({ "id": "standalone", "pipeline_slug": Value::Null, "stale_warning": false }));
            opts.push(json!({ "id": "other_project", "pipeline_slug": Value::Null, "stale_warning": false }));
            opts.push(json!({ "id": "close_first", "pipeline_slug": Value::Null, "stale_warning": false }));
            // One pick_pipeline:<slug> entry per additional pipeline (skip the
            // top one — it's already covered by "continue:<top_slug>").
            for (slug, _, _, _, _, stale, _) in active.iter().skip(1) {
                opts.push(json!({
                    "id": format!("pick_pipeline:{}", slug),
                    "pipeline_slug": slug,
                    "stale_warning": stale,
                }));
            }
            opts
        }
        _ => Vec::new(),
    };

    Ok(json!({
        "case": case,
        "project_name": project_name,
        "project_root": project_root,
        "active_pipelines": active_pipelines,
        "suggested_action": suggested_action,
        "options_spec": options_spec,
    })
    .to_string())
}

// ---------------------------------------------------------------------------
// `list`
// ---------------------------------------------------------------------------

fn list() -> Result<String, String> {
    // Read-only: do NOT prune here.
    let mut reg = load_registry_or_reset();
    sort_projects_desc(&mut reg);
    let active = last_active(&reg);

    let rows: Vec<Value> = projects_array(&reg)
        .into_iter()
        .map(|p| {
            let path = p.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let last_used = p
                .get("last_used")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let is_active = active.as_deref() == Some(path.as_str());
            json!({
                "path": path,
                "name": name,
                "last_used": last_used,
                "is_active": is_active,
            })
        })
        .collect();

    Ok(Value::Array(rows).to_string())
}

// ---------------------------------------------------------------------------
// `use`
// ---------------------------------------------------------------------------

fn use_project(path_arg: &str) -> Result<String, String> {
    let abs = canonicalize_path(path_arg);
    let abs_str = abs.to_string_lossy().to_string();

    if !abs.exists() {
        return Ok(json!({
            "ok": false,
            "error": format!("PATH_NOT_FOUND: {}", abs_str),
        })
        .to_string());
    }
    let config_path = project_config_path(&abs);
    if !config_path.exists() {
        return Ok(json!({
            "ok": false,
            "error": format!("NO_CONFIG_AT_PATH: {}", config_path.display()),
        })
        .to_string());
    }

    let mut reg = load_registry_or_reset();
    let now = now_iso();

    let mut projects = projects_array(&reg);
    let present = projects
        .iter()
        .any(|p| p.get("path").and_then(|v| v.as_str()) == Some(abs_str.as_str()));

    if !present {
        let name = read_project_name(&abs).unwrap_or_else(|| "(unknown)".to_string());
        projects.push(json!({
            "path": abs_str,
            "name": name,
            "created_at": now,
            "last_used": now,
        }));
    } else {
        // Bump last_used.
        for p in projects.iter_mut() {
            if p.get("path").and_then(|v| v.as_str()) == Some(abs_str.as_str()) {
                if let Some(obj) = p.as_object_mut() {
                    obj.insert("last_used".to_string(), json!(now));
                }
            }
        }
    }
    set_projects(&mut reg, projects);
    set_last_active(&mut reg, Some(&abs_str));
    sort_projects_desc(&mut reg);
    write_registry(&reg)?;

    Ok(json!({
        "ok": true,
        "active": abs_str,
    })
    .to_string())
}

// ---------------------------------------------------------------------------
// `add`
// ---------------------------------------------------------------------------

fn add(path_arg: &str) -> Result<String, String> {
    let abs = canonicalize_path(path_arg);
    let abs_str = abs.to_string_lossy().to_string();

    if !abs.exists() {
        return Err(format!("PATH_NOT_FOUND: {}", abs_str));
    }
    let config_path = project_config_path(&abs);
    if !config_path.exists() {
        return Err(format!("NO_CONFIG_AT_PATH: {}", config_path.display()));
    }

    let mut reg = load_registry_or_reset();
    let mut projects = projects_array(&reg);
    let present = projects
        .iter()
        .any(|p| p.get("path").and_then(|v| v.as_str()) == Some(abs_str.as_str()));
    if present {
        return Ok(json!({
            "ok": true,
            "path": abs_str,
            "already_present": true,
        })
        .to_string());
    }
    let name = read_project_name(&abs).unwrap_or_else(|| "(unknown)".to_string());
    let now = now_iso();
    projects.push(json!({
        "path": abs_str,
        "name": name,
        "created_at": now,
        "last_used": now,
    }));
    set_projects(&mut reg, projects);
    // Do NOT change last_active on `add`.
    sort_projects_desc(&mut reg);
    write_registry(&reg)?;

    Ok(json!({
        "ok": true,
        "path": abs_str,
        "already_present": false,
    })
    .to_string())
}

// ---------------------------------------------------------------------------
// `remove`
// ---------------------------------------------------------------------------

fn remove(path_arg: &str) -> Result<String, String> {
    let abs = canonicalize_path(path_arg);
    let abs_str = abs.to_string_lossy().to_string();

    let mut reg = load_registry_or_reset();
    let before = projects_array(&reg);
    let after: Vec<Value> = before
        .into_iter()
        .filter(|p| p.get("path").and_then(|v| v.as_str()) != Some(abs_str.as_str()))
        .collect();
    set_projects(&mut reg, after);

    if last_active(&reg).as_deref() == Some(abs_str.as_str()) {
        set_last_active(&mut reg, None);
    }
    sort_projects_desc(&mut reg);
    write_registry(&reg)?;

    Ok(json!({
        "ok": true,
        "removed": abs_str,
    })
    .to_string())
}

// ---------------------------------------------------------------------------
// `global-config`
// ---------------------------------------------------------------------------

fn global_config(args: &[String]) -> Result<String, String> {
    let sub = args
        .first()
        .ok_or_else(|| "Usage: compass-cli project global-config <get|set> [...]".to_string())?;
    match sub.as_str() {
        "get" => {
            let key = parse_flag(args, "--key");
            global_config_get(key.as_deref())
        }
        "set" => {
            let key = parse_flag(args, "--key").ok_or_else(|| {
                "Usage: compass-cli project global-config set --key <k> --value <v>".to_string()
            })?;
            let value = parse_flag(args, "--value").ok_or_else(|| {
                "Usage: compass-cli project global-config set --key <k> --value <v>".to_string()
            })?;
            global_config_set(&key, &value)
        }
        other => Err(format!("Unknown global-config command: {}", other)),
    }
}

fn load_global_config() -> Value {
    let path = global_config_path();
    if !path.exists() {
        return json!({});
    }
    match fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
    {
        Some(v) => v,
        None => {
            eprintln!(
                "warn: CORRUPT global-config at {}; treating as empty",
                path.display()
            );
            json!({})
        }
    }
}

fn write_global_config(value: &Value) -> Result<(), String> {
    ensure_compass_dir()?;
    let path = global_config_path();
    if !path.exists() {
        fs::write(&path, "{}")
            .map_err(|e| format!("Cannot create {}: {}", path.display(), e))?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    file.lock_exclusive()
        .map_err(|e| format!("Cannot lock {}: {}", path.display(), e))?;

    let result = (|| -> Result<(), String> {
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_string_pretty(value)
            .map_err(|e| format!("JSON serialize error: {}", e))?;
        fs::write(&tmp, &body)
            .map_err(|e| format!("Cannot write {}: {}", tmp.display(), e))?;
        fs::rename(&tmp, &path)
            .map_err(|e| format!("Cannot rename {} → {}: {}", tmp.display(), path.display(), e))?;
        Ok(())
    })();

    let _ = file.unlock();
    result
}

fn global_config_get(key: Option<&str>) -> Result<String, String> {
    let data = load_global_config();
    let value = match key {
        Some(k) => lookup_dot_path(&data, k)
            .cloned()
            .unwrap_or(Value::Null),
        None => data,
    };
    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON serialize error: {}", e))
}

/// Whitelist of top-level keys allowed in `global-config set`. Anything else
/// is rejected so a stray key (or path-traversal attempt via dot-path) cannot
/// pollute the file.
const ALLOWED_GLOBAL_KEYS: &[&str] = &[
    "lang",
    "default_tech_stack",
    "default_review_style",
    "default_domain",
];

fn global_config_set(key: &str, raw_value: &str) -> Result<String, String> {
    // Reject unknown top-level keys. Dot-paths are allowed only within an
    // allowed root (e.g. `default_tech_stack.0` would traverse an array).
    let top = key.split('.').next().unwrap_or("");
    if !ALLOWED_GLOBAL_KEYS.contains(&top) {
        return Err(format!(
            "INVALID_KEY: '{}' is not an allowed global-config key. Allowed: {:?}",
            key, ALLOWED_GLOBAL_KEYS
        ));
    }

    let mut data = load_global_config();

    // Auto-create skeleton.
    if !data.is_object() {
        data = json!({});
    }
    {
        let obj = data.as_object_mut().expect("data is object");
        let now = now_iso();
        obj.entry("version")
            .or_insert_with(|| json!(GLOBAL_CONFIG_VERSION));
        obj.entry("created_at")
            .or_insert_with(|| json!(now.clone()));
        obj.insert("updated_at".to_string(), json!(now));
    }

    // Parse value as JSON; fall back to raw string.
    let parsed: Value = serde_json::from_str(raw_value).unwrap_or_else(|_| json!(raw_value));
    set_dot_path(&mut data, key, parsed);

    write_global_config(&data)?;
    Ok(json!({
        "ok": true,
        "key": key,
    })
    .to_string())
}

fn lookup_dot_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    for seg in path.split('.') {
        if seg.is_empty() {
            return None;
        }
        cur = match cur {
            Value::Object(map) => map.get(seg)?,
            Value::Array(arr) => {
                let idx: usize = seg.parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }
    Some(cur)
}

fn set_dot_path(root: &mut Value, path: &str, new_val: Value) {
    let segs: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
    if segs.is_empty() {
        return;
    }
    let mut cur = root;
    for (i, seg) in segs.iter().enumerate() {
        let is_last = i == segs.len() - 1;
        if !cur.is_object() {
            *cur = json!({});
        }
        let obj = cur.as_object_mut().unwrap();
        if is_last {
            obj.insert(seg.to_string(), new_val.clone());
            return;
        }
        if !obj.contains_key(*seg) {
            obj.insert(seg.to_string(), json!({}));
        }
        cur = obj.get_mut(*seg).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Test support — shared across modules that mutate $HOME in tests.
// `#[cfg(test)]` keeps it out of release builds; `#[doc(hidden)]` hides it
// from `cargo doc`. Visibility stays `pub(crate)` so peer-module test trees
// (e.g. `cmd::state::tests`) can import the same Mutex — mandatory so cross-
// module tests serialize on the one process-global `$HOME`.
// ---------------------------------------------------------------------------

#[cfg(test)]
#[doc(hidden)]
pub(crate) mod test_support {
    use std::sync::Mutex;
    pub static HOME_GUARD: Mutex<()> = Mutex::new(());
}

// ---------------------------------------------------------------------------
// Tests (full unit coverage — T-12)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_support::HOME_GUARD;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_tmp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!(
            "compass-cli-projtest-{}-{}-{}-{}",
            tag, pid, nanos, n
        ));
        fs::create_dir_all(&dir).expect("create unique tmp dir");
        dir
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    /// Build a minimal project root with `.compass/.state/config.json` inside
    /// the supplied parent directory; returns the project root path
    /// canonicalized so it matches what `canonicalize_path` produces at
    /// runtime (important on macOS where `/var` is a symlink to
    /// `/private/var`).
    fn make_project(parent: &Path, name: &str) -> PathBuf {
        let root = parent.join(name);
        let state = root.join(".compass").join(".state");
        fs::create_dir_all(&state).expect("create state dir");
        let config = json!({
            "version": "1.1.1",
            "project": {"name": name, "po": "@test"},
        });
        fs::write(state.join("config.json"), config.to_string())
            .expect("write config.json");
        fs::canonicalize(&root).unwrap_or(root)
    }

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).expect("valid json")
    }

    /// RAII guard: locks the shared `HOME_GUARD` mutex, sets `$HOME` to a
    /// unique tmp dir, and restores on drop. Also restores `cwd` if the caller
    /// changed it by tracking the value present at guard construction.
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
            HomeGuard {
                home,
                prev_home,
                prev_cwd,
                _lock: lock,
            }
        }

        fn home(&self) -> &Path {
            &self.home
        }

        fn registry_file(&self) -> PathBuf {
            self.home.join(".compass").join("projects.json")
        }

        fn write_registry_raw(&self, body: &str) {
            fs::write(self.registry_file(), body).expect("write registry");
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
            cleanup(&self.home);
        }
    }

    // ===== T-12 NAMED UNIT TESTS (15) =====

    #[test]
    fn resolve_ok_single_alive() {
        let g = HomeGuard::new("t12_resolve_ok_single_alive");
        let parent = unique_tmp_dir("t12_ros_parent");
        let root = make_project(&parent, "proj_one");
        let root_str = root.to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": root_str,
            "projects": [{
                "path": root_str,
                "name": "proj_one",
                "created_at": "2026-04-01T00:00:00Z",
                "last_used": "2026-04-01T00:00:00Z",
            }],
        });
        g.write_registry_raw(&reg.to_string());

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ok"));
        assert_eq!(v["project_root"], json!(root_str));
        assert_eq!(v["name"], json!("proj_one"));
        assert_eq!(v["migrated_from_v11"], json!(false));
        assert!(v["config"].is_object());
        assert_eq!(v["config"]["project"]["name"], json!("proj_one"));

        cleanup(&parent);
    }

    #[test]
    fn resolve_ok_last_active_alive() {
        let g = HomeGuard::new("t12_resolve_ok_last_active");
        let parent = unique_tmp_dir("t12_rla_parent");
        let a = make_project(&parent, "alpha");
        let b = make_project(&parent, "beta");
        let c = make_project(&parent, "gamma");
        let a_str = a.to_string_lossy().to_string();
        let b_str = b.to_string_lossy().to_string();
        let c_str = c.to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": b_str,
            "projects": [
                {"path": a_str, "name": "alpha", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-05T00:00:00Z"},
                {"path": b_str, "name": "beta",  "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-04T00:00:00Z"},
                {"path": c_str, "name": "gamma", "created_at": "2026-04-03T00:00:00Z", "last_used": "2026-04-03T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ok"));
        assert_eq!(v["project_root"], json!(b_str));
        assert_eq!(v["name"], json!("beta"));
        assert_eq!(v["migrated_from_v11"], json!(false));

        cleanup(&parent);
    }

    #[test]
    fn resolve_ambiguous_last_active_dead() {
        let g = HomeGuard::new("t12_resolve_ambiguous");
        let parent = unique_tmp_dir("t12_amb_parent");
        let a = make_project(&parent, "alpha");
        let b = make_project(&parent, "beta");
        let a_str = a.to_string_lossy().to_string();
        let b_str = b.to_string_lossy().to_string();
        // Dead entry: real dir removed immediately.
        let dead_dir = parent.join("dead_one");
        let dead_str = dead_dir.to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": dead_str,
            "projects": [
                {"path": dead_str, "name": "dead_one", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-10T00:00:00Z"},
                {"path": a_str,    "name": "alpha",    "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-09T00:00:00Z"},
                {"path": b_str,    "name": "beta",     "created_at": "2026-04-03T00:00:00Z", "last_used": "2026-04-08T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ambiguous"));
        let cands = v["candidates"].as_array().expect("candidates array");
        assert_eq!(cands.len(), 2, "should have 2 alive candidates");
        // Sorted desc by last_used: alpha (04-09) then beta (04-08).
        assert_eq!(cands[0]["path"], json!(a_str));
        assert_eq!(cands[1]["path"], json!(b_str));

        // Registry on disk should be pruned.
        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        let disk_projs = disk["projects"].as_array().unwrap();
        assert_eq!(disk_projs.len(), 2);
        assert!(disk_projs
            .iter()
            .all(|p| p["path"].as_str() != Some(&dead_str)));
        assert!(disk["last_active"].is_null(), "last_active cleared after prune");

        cleanup(&parent);
    }

    #[test]
    fn resolve_fallback_auto() {
        let g = HomeGuard::new("t12_resolve_fallback_auto");
        let parent = unique_tmp_dir("t12_rfa_parent");
        let alive = make_project(&parent, "alive_one");
        let alive_str = alive.to_string_lossy().to_string();
        let dead_str = parent.join("dead_one").to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": dead_str,
            "projects": [
                {"path": dead_str,  "name": "dead_one",  "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-10T00:00:00Z"},
                {"path": alive_str, "name": "alive_one", "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-09T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ok"));
        assert_eq!(v["project_root"], json!(alive_str));
        assert_eq!(v["name"], json!("alive_one"));

        // Registry post: dead pruned, last_active reset to alive.
        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        assert_eq!(disk["last_active"], json!(alive_str));
        let disk_projs = disk["projects"].as_array().unwrap();
        assert_eq!(disk_projs.len(), 1);
        assert_eq!(disk_projs[0]["path"], json!(alive_str));

        cleanup(&parent);
    }

    #[test]
    fn resolve_none_empty() {
        let _g = HomeGuard::new("t12_resolve_none_empty");

        // cwd somewhere with no compass config.
        let cwd = unique_tmp_dir("t12_rne_cwd");
        std::env::set_current_dir(&cwd).unwrap();

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("none"));
        assert_eq!(v["reason"], json!("empty_registry"));

        cleanup(&cwd);
    }

    #[test]
    fn resolve_none_all_dead() {
        let g = HomeGuard::new("t12_resolve_none_all_dead");

        // cwd far away from any compass project.
        let cwd = unique_tmp_dir("t12_rnad_cwd");
        std::env::set_current_dir(&cwd).unwrap();

        let parent = unique_tmp_dir("t12_rnad_parent");
        let d1 = parent.join("dead_1").to_string_lossy().to_string();
        let d2 = parent.join("dead_2").to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": d1,
            "projects": [
                {"path": d1, "name": "dead_1", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-10T00:00:00Z"},
                {"path": d2, "name": "dead_2", "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-09T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("none"));
        assert_eq!(v["reason"], json!("all_paths_dead"));

        // Registry on disk pruned to empty.
        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        assert!(disk["projects"].as_array().unwrap().is_empty());
        assert!(disk["last_active"].is_null());

        cleanup(&parent);
        cleanup(&cwd);
    }

    #[test]
    fn resolve_migrates_v11() {
        let g = HomeGuard::new("t12_resolve_migrates_v11");

        // No registry file at all.
        assert!(!g.registry_file().exists());

        let parent = unique_tmp_dir("t12_mig_parent");
        let root = make_project(&parent, "legacy_proj");
        std::env::set_current_dir(&root).unwrap();

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ok"));
        assert_eq!(v["migrated_from_v11"], json!(true));
        assert_eq!(v["name"], json!("legacy_proj"));

        // Registry now exists with the cwd entry.
        assert!(g.registry_file().exists());
        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        let projs = disk["projects"].as_array().unwrap();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0]["name"], json!("legacy_proj"));
        assert!(disk["last_active"].is_string());

        cleanup(&parent);
    }

    #[test]
    fn use_updates_last_active() {
        let g = HomeGuard::new("t12_use_updates_last_active");
        let parent = unique_tmp_dir("t12_ula_parent");
        let a = make_project(&parent, "alpha");
        let b = make_project(&parent, "beta");
        let a_str = a.to_string_lossy().to_string();
        let b_str = b.to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": a_str,
            "projects": [
                {"path": a_str, "name": "alpha", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-05T00:00:00Z"},
                {"path": b_str, "name": "beta",  "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-04T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = use_project(&b_str).expect("use ok");
        let v = parse(&out);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["active"], json!(b_str));

        // Disk state: last_active = b, and b's last_used is fresh (non-empty, != old value).
        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        assert_eq!(disk["last_active"], json!(b_str));
        let b_entry = disk["projects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"].as_str() == Some(&b_str))
            .expect("entry b");
        let lu = b_entry["last_used"].as_str().unwrap_or("");
        assert!(!lu.is_empty());
        assert_ne!(lu, "2026-04-04T00:00:00Z", "last_used must be bumped");

        cleanup(&parent);
    }

    #[test]
    fn use_auto_adds() {
        let g = HomeGuard::new("t12_use_auto_adds");
        let parent = unique_tmp_dir("t12_uaa_parent");
        let root = make_project(&parent, "fresh_proj");
        let root_str = root.to_string_lossy().to_string();

        // Registry starts empty / absent.
        assert!(!g.registry_file().exists());

        let out = use_project(&root_str).expect("use ok");
        let v = parse(&out);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["active"], json!(root_str));

        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        let projs = disk["projects"].as_array().unwrap();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0]["path"], json!(root_str));
        assert_eq!(projs[0]["name"], json!("fresh_proj"));
        assert_eq!(disk["last_active"], json!(root_str));

        cleanup(&parent);
    }

    #[test]
    fn add_rejects_no_config() {
        let g = HomeGuard::new("t12_add_rejects_no_config");
        let bare = unique_tmp_dir("t12_arnc_bare");
        let bare_str = bare.to_string_lossy().to_string();

        let err = add(&bare_str).expect_err("add must reject missing config");
        assert!(
            err.contains("NO_CONFIG_AT_PATH") || err.to_lowercase().contains("config"),
            "error should reference missing config, got: {}",
            err
        );

        // Registry must remain unchanged (absent).
        assert!(
            !g.registry_file().exists(),
            "registry must not be created on rejected add"
        );

        cleanup(&bare);
    }

    #[test]
    fn list_sorted() {
        let g = HomeGuard::new("t12_list_sorted");

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": "/tmp/compass-nonexistent-a",
            "projects": [
                {"path": "/tmp/compass-nonexistent-a", "name": "A", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-10T00:00:00Z"},
                {"path": "/tmp/compass-nonexistent-b", "name": "B", "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-13T00:00:00Z"},
                {"path": "/tmp/compass-nonexistent-c", "name": "C", "created_at": "2026-04-03T00:00:00Z", "last_used": "2026-04-11T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = list().expect("list ok");
        let arr = parse(&out);
        let rows = arr.as_array().expect("array");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["path"], json!("/tmp/compass-nonexistent-b"));
        assert_eq!(rows[1]["path"], json!("/tmp/compass-nonexistent-c"));
        assert_eq!(rows[2]["path"], json!("/tmp/compass-nonexistent-a"));
        assert_eq!(rows[0]["is_active"], json!(false));
        assert_eq!(rows[1]["is_active"], json!(false));
        assert_eq!(rows[2]["is_active"], json!(true));
    }

    #[test]
    fn remove_clears_last_active() {
        let g = HomeGuard::new("t12_remove_clears_last_active");
        let parent = unique_tmp_dir("t12_rcla_parent");
        let a = make_project(&parent, "alpha");
        let b = make_project(&parent, "beta");
        let a_str = a.to_string_lossy().to_string();
        let b_str = b.to_string_lossy().to_string();

        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": a_str,
            "projects": [
                {"path": a_str, "name": "alpha", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-05T00:00:00Z"},
                {"path": b_str, "name": "beta",  "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-04T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let out = remove(&a_str).expect("remove ok");
        let v = parse(&out);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["removed"], json!(a_str));

        let reg_raw = fs::read_to_string(g.registry_file()).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).unwrap();
        assert!(
            disk["last_active"].is_null(),
            "last_active must be cleared after removing the active entry; got {:?}",
            disk["last_active"]
        );
        let projs = disk["projects"].as_array().unwrap();
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0]["path"], json!(b_str));

        cleanup(&parent);
    }

    #[test]
    fn registry_file_lock() {
        let g = HomeGuard::new("t12_registry_file_lock");
        let parent = unique_tmp_dir("t12_rfl_parent");
        let a = make_project(&parent, "alpha");
        let b = make_project(&parent, "beta");
        let a_str = a.to_string_lossy().to_string();
        let b_str = b.to_string_lossy().to_string();

        // Seed registry with both entries to avoid concurrent read-auto-add
        // races; both threads just bump last_active.
        let reg = json!({
            "version": REGISTRY_VERSION,
            "last_active": Value::Null,
            "projects": [
                {"path": a_str, "name": "alpha", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-01T00:00:00Z"},
                {"path": b_str, "name": "beta",  "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-02T00:00:00Z"},
            ],
        });
        g.write_registry_raw(&reg.to_string());

        let a_c = a_str.clone();
        let b_c = b_str.clone();
        let t1 = std::thread::spawn(move || use_project(&a_c));
        let t2 = std::thread::spawn(move || use_project(&b_c));
        let r1 = t1.join().expect("thread 1 joined");
        let r2 = t2.join().expect("thread 2 joined");
        assert!(r1.is_ok(), "thread 1 failed: {:?}", r1);
        assert!(r2.is_ok(), "thread 2 failed: {:?}", r2);

        // Registry must still be valid JSON and last_active must match one
        // of the two paths.
        let reg_raw = fs::read_to_string(g.registry_file()).expect("read registry");
        let disk: Value = serde_json::from_str(&reg_raw).expect("valid JSON after concurrent use");
        let la = disk["last_active"].as_str().unwrap_or("");
        assert!(
            la == a_str || la == b_str,
            "last_active must be one of the used paths; got {}",
            la
        );
        let projs = disk["projects"].as_array().unwrap();
        assert_eq!(projs.len(), 2, "no duplicate entries created");

        cleanup(&parent);
    }

    #[test]
    fn registry_corrupt_backup() {
        let g = HomeGuard::new("t12_registry_corrupt_backup");
        let cwd = unique_tmp_dir("t12_rcb_cwd");
        std::env::set_current_dir(&cwd).unwrap();

        let reg_path = g.registry_file();
        fs::write(&reg_path, "{broken-json").unwrap();

        let out = resolve().expect("resolve ok despite corrupt registry");
        let v = parse(&out);
        assert_eq!(v["status"], json!("none"));
        assert_eq!(v["reason"], json!("empty_registry"));

        // Backup file exists with bad payload.
        let bak = reg_path.with_extension("json.bak");
        assert!(bak.exists(), "backup .bak must exist");
        let bak_raw = fs::read_to_string(&bak).unwrap();
        assert!(bak_raw.contains("broken-json"), "backup should contain original bad payload");

        // Main registry reset to empty skeleton.
        let reg_raw = fs::read_to_string(&reg_path).unwrap();
        let disk: Value = serde_json::from_str(&reg_raw).expect("main registry parseable again");
        assert!(disk["projects"].as_array().unwrap().is_empty());
        assert!(disk["last_active"].is_null());
        assert_eq!(disk["version"], json!(REGISTRY_VERSION));

        cleanup(&cwd);
    }

    #[test]
    fn global_config_init() {
        let g = HomeGuard::new("t12_global_config_init");
        let gc_path = g.home().join(".compass").join("global-config.json");
        assert!(!gc_path.exists());

        let out = global_config_set("lang", "vi").expect("set ok");
        let v = parse(&out);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["key"], json!("lang"));

        assert!(gc_path.exists(), "global-config.json created");
        let raw = fs::read_to_string(&gc_path).unwrap();
        let disk: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(disk["version"], json!(GLOBAL_CONFIG_VERSION));
        assert_eq!(disk["lang"], json!("vi"));
        assert!(disk["created_at"].is_string());
        assert!(disk["updated_at"].is_string());

        // Round-trip: get --key lang returns "vi".
        let got = global_config_get(Some("lang")).expect("get ok");
        let got_v: Value = serde_json::from_str(&got).unwrap();
        assert_eq!(got_v, json!("vi"));
    }

    // ===== Legacy smoke tests (renamed per T-12 rules) =====

    #[test]
    fn resolve_empty_registry_returns_none_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("resolve_empty");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        // cwd far away from any compass project — pick the tmp dir itself,
        // which has no `.compass/.state/config.json`.
        let cwd_guard = unique_tmp_dir("resolve_empty_cwd");
        let prev_cwd = std::env::current_dir().ok();
        std::env::set_current_dir(&cwd_guard).unwrap();

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("none"));
        assert_eq!(v["reason"], json!("empty_registry"));

        // Restore.
        if let Some(p) = prev_cwd {
            let _ = std::env::set_current_dir(p);
        }
        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
        cleanup(&cwd_guard);
    }

    #[test]
    fn resolve_one_alive_returns_ok_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("resolve_one_alive");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        let projects_parent = unique_tmp_dir("resolve_one_alive_projs");
        let root = make_project(&projects_parent, "proj_one");
        let root_str = root.to_string_lossy().to_string();

        // Seed registry.
        fs::create_dir_all(home.join(".compass")).unwrap();
        let reg = json!({
            "version": "1.0",
            "last_active": root_str,
            "projects": [{
                "path": root_str,
                "name": "proj_one",
                "created_at": "2026-04-01T00:00:00Z",
                "last_used": "2026-04-01T00:00:00Z",
            }],
        });
        fs::write(home.join(".compass").join("projects.json"), reg.to_string()).unwrap();

        let out = resolve().expect("resolve ok");
        let v = parse(&out);
        assert_eq!(v["status"], json!("ok"));
        assert_eq!(v["project_root"], json!(root_str));
        assert_eq!(v["name"], json!("proj_one"));
        assert_eq!(v["migrated_from_v11"], json!(false));
        assert!(v["config"].is_object());

        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
        cleanup(&projects_parent);
    }

    #[test]
    fn use_sets_last_active_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("use_sets_active");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        let projects_parent = unique_tmp_dir("use_sets_active_projs");
        let root = make_project(&projects_parent, "proj_use");
        let root_str = root.to_string_lossy().to_string();

        let out = use_project(&root_str).expect("use ok");
        let v = parse(&out);
        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["active"], json!(root_str));

        // Verify registry on disk.
        let reg_raw = fs::read_to_string(home.join(".compass").join("projects.json"))
            .expect("registry exists");
        let reg: Value = serde_json::from_str(&reg_raw).unwrap();
        assert_eq!(reg["last_active"], json!(root_str));
        assert_eq!(reg["projects"].as_array().unwrap().len(), 1);

        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
        cleanup(&projects_parent);
    }

    #[test]
    fn add_requires_config_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("add_requires_config");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        // Directory exists but has no .compass/.state/config.json.
        let bare = unique_tmp_dir("add_bare");
        let bare_str = bare.to_string_lossy().to_string();

        let err = add(&bare_str).expect_err("add must reject missing config");
        assert!(
            err.contains("NO_CONFIG_AT_PATH"),
            "error should mention NO_CONFIG_AT_PATH, got: {}",
            err
        );

        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
        cleanup(&bare);
    }

    #[test]
    fn list_sorted_by_last_used_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("list_sorted");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        fs::create_dir_all(home.join(".compass")).unwrap();
        let reg = json!({
            "version": "1.0",
            "last_active": "/tmp/a",
            "projects": [
                {"path": "/tmp/a", "name": "A", "created_at": "2026-04-01T00:00:00Z", "last_used": "2026-04-10T00:00:00Z"},
                {"path": "/tmp/b", "name": "B", "created_at": "2026-04-02T00:00:00Z", "last_used": "2026-04-13T00:00:00Z"},
                {"path": "/tmp/c", "name": "C", "created_at": "2026-04-03T00:00:00Z", "last_used": "2026-04-11T00:00:00Z"},
            ],
        });
        fs::write(home.join(".compass").join("projects.json"), reg.to_string()).unwrap();

        let out = list().expect("list ok");
        let arr = parse(&out);
        let rows = arr.as_array().expect("array");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["path"], json!("/tmp/b"));
        assert_eq!(rows[1]["path"], json!("/tmp/c"));
        assert_eq!(rows[2]["path"], json!("/tmp/a"));
        assert_eq!(rows[0]["is_active"], json!(false));
        assert_eq!(rows[2]["is_active"], json!(true));

        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
    }

    #[test]
    fn corrupt_registry_backed_up_and_reset_smoke() {
        let _g = HOME_GUARD.lock().unwrap();
        let home = unique_tmp_dir("corrupt_registry");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);

        fs::create_dir_all(home.join(".compass")).unwrap();
        let reg_path = home.join(".compass").join("projects.json");
        fs::write(&reg_path, "{not-valid json").unwrap();

        // load_registry_or_reset is pure-read-with-side-effects.
        let v = load_registry_or_reset();
        assert_eq!(v["version"], json!(REGISTRY_VERSION));
        assert!(v["projects"].as_array().unwrap().is_empty());

        let bak = home.join(".compass").join("projects.json.bak");
        assert!(bak.exists(), "backup should exist at {}", bak.display());
        let bak_raw = fs::read_to_string(&bak).unwrap();
        assert!(bak_raw.contains("not-valid"));

        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        cleanup(&home);
    }
}
