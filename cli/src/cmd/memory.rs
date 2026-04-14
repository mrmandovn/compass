use crate::helpers;
use fs2::FileExt;
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MEMORY_VERSION: &str = "1.0";
const MAX_SESSIONS: usize = 10;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(usage());
    }

    match args[0].as_str() {
        "init" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli memory init <project_root>".into());
            }
            init(&args[1])
        }
        "get" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli memory get <project_root> [--key <dot.path>]".into());
            }
            let key = parse_flag(args, "--key");
            get(&args[1], key.as_deref())
        }
        "update" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli memory update <project_root> --patch <json>".into());
            }
            let patch = parse_flag(args, "--patch")
                .or_else(|| args.get(2).cloned())
                .ok_or_else(|| "Missing --patch <json>".to_string())?;
            update(&args[1], &patch)
        }
        "list-sessions" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli memory list-sessions <project_root>".into());
            }
            list_sessions(&args[1])
        }
        other => Err(format!("Unknown memory command: {}", other)),
    }
}

fn usage() -> String {
    "Usage: compass-cli memory <init|get|update|list-sessions> <project_root> [...]".into()
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

/// Resolve the canonical memory-file path for a project root.
///
/// Canonical location is `<project_root>/.compass/.state/project-memory.json`.
/// For backwards compatibility with already-present flat files, if the canonical
/// parent doesn't exist yet but a flat `<project_root>/project-memory.json` does,
/// we use the flat one.
fn resolve_memory_path(project_root: &str) -> PathBuf {
    let canonical = Path::new(project_root)
        .join(".compass")
        .join(".state")
        .join("project-memory.json");
    if canonical.exists() {
        return canonical;
    }
    let flat = Path::new(project_root).join("project-memory.json");
    if flat.exists() {
        return flat;
    }
    canonical
}

fn now_iso() -> String {
    // RFC3339/ISO-8601 UTC without pulling chrono in — seconds granularity is fine
    // for schema compliance (`created_at`/`updated_at`).
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_iso_utc(secs)
}

fn format_iso_utc(total_secs: u64) -> String {
    // Civil-from-days algorithm (Howard Hinnant) — enough for a schema timestamp,
    // no external deps required.
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

fn init(project_root: &str) -> Result<String, String> {
    let path = Path::new(project_root)
        .join(".compass")
        .join(".state")
        .join("project-memory.json");

    if path.exists() {
        eprintln!("already exists: {}", path.display());
        return Ok(json!({
            "ok": true,
            "already_exists": true,
            "path": path.to_string_lossy(),
        })
        .to_string());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create {}: {}", parent.display(), e))?;
    }

    let now = now_iso();
    let skeleton = json!({
        "memory_version": MEMORY_VERSION,
        "created_at": now,
        "updated_at": now,
        "sessions": [],
        "decisions": [],
        "discovered_conventions": [],
        "resolved_ambiguities": [],
        "glossary": {},
    });

    helpers::write_json(&path, &skeleton)?;

    Ok(json!({
        "ok": true,
        "already_exists": false,
        "path": path.to_string_lossy(),
    })
    .to_string())
}

fn read_memory(path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("CORRUPT_MEMORY: {} ({})", path.display(), e))?;
    check_version(&value)?;
    Ok(value)
}

fn check_version(value: &Value) -> Result<(), String> {
    match value.get("memory_version").and_then(|v| v.as_str()) {
        Some(MEMORY_VERSION) => Ok(()),
        Some(other) => Err(format!(
            "UNSUPPORTED_MEMORY_VERSION: found {:?}, expected {:?}",
            other, MEMORY_VERSION
        )),
        None => Err("UNSUPPORTED_MEMORY_VERSION: missing memory_version".into()),
    }
}

fn get(project_root: &str, key: Option<&str>) -> Result<String, String> {
    let path = resolve_memory_path(project_root);
    let data = read_memory(&path)?;

    let value = match key {
        Some(k) => lookup_dot_path(&data, k)
            .ok_or_else(|| format!("Key not found: {}", k))?
            .clone(),
        None => data,
    };

    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON serialize error: {}", e))
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

fn update(project_root: &str, patch_str: &str) -> Result<String, String> {
    let path = resolve_memory_path(project_root);

    if !path.exists() {
        return Err(format!(
            "Memory file not found at {}. Run `memory init` first.",
            path.display()
        ));
    }

    let patch: Value = serde_json::from_str(patch_str)
        .map_err(|e| format!("Invalid JSON patch: {}", e))?;

    // Exclusive advisory lock around the read-merge-write cycle to dodge the
    // TEST-SPEC race between two concurrent `memory update` callers.
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    file.lock_exclusive()
        .map_err(|e| format!("Cannot lock {}: {}", path.display(), e))?;

    let result = (|| -> Result<Value, String> {
        let mut data = read_memory(&path)?;
        deep_merge(&mut data, &patch);
        enforce_fifo_and_aggregate(&mut data);
        if let Some(obj) = data.as_object_mut() {
            obj.insert("updated_at".to_string(), json!(now_iso()));
        }
        helpers::write_json(&path, &data)?;
        Ok(data)
    })();

    let _ = file.unlock();

    let data = result?;
    Ok(json!({
        "ok": true,
        "path": path.to_string_lossy(),
        "sessions_count": data.get("sessions").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
    })
    .to_string())
}

/// Recursive deep merge: patch values override dst values. Objects merge
/// key-by-key; arrays and scalars are replaced wholesale, EXCEPT that when both
/// sides are arrays at the same path we concatenate (dst first, patch after).
/// This lets callers append to `sessions[]` / `decisions[]` naturally by
/// patching `{"sessions":[new_entry]}`.
fn deep_merge(dst: &mut Value, patch: &Value) {
    match (dst, patch) {
        (Value::Object(dst_map), Value::Object(patch_map)) => {
            for (k, v) in patch_map {
                match dst_map.get_mut(k) {
                    Some(existing) => deep_merge(existing, v),
                    None => {
                        dst_map.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (Value::Array(dst_arr), Value::Array(patch_arr)) => {
            for item in patch_arr {
                dst_arr.push(item.clone());
            }
        }
        (dst_slot, patch_val) => {
            *dst_slot = patch_val.clone();
        }
    }
}

/// Enforce `sessions` length ≤ 10; before dropping index 0, merge its aggregates
/// into the top-level aggregates with dedup on the schema-defined composite keys.
fn enforce_fifo_and_aggregate(data: &mut Value) {
    let obj = match data.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    loop {
        let overflow = obj
            .get("sessions")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
            > MAX_SESSIONS;
        if !overflow {
            break;
        }

        // Pop oldest session
        let oldest = match obj.get_mut("sessions").and_then(|v| v.as_array_mut()) {
            Some(arr) if !arr.is_empty() => arr.remove(0),
            _ => break,
        };

        // Merge oldest's aggregates into top-level, deduped.
        if let Some(decisions) = oldest.get("decisions").and_then(|v| v.as_array()) {
            merge_dedup(obj, "decisions", decisions, &["topic", "decision"]);
        }
        if let Some(convs) = oldest.get("discovered_conventions").and_then(|v| v.as_array()) {
            merge_dedup(obj, "discovered_conventions", convs, &["area", "convention"]);
        }
        if let Some(ambigs) = oldest.get("resolved_ambiguities").and_then(|v| v.as_array()) {
            merge_dedup(obj, "resolved_ambiguities", ambigs, &["question", "answer"]);
        }
    }
}

fn merge_dedup(
    obj: &mut serde_json::Map<String, Value>,
    key: &str,
    incoming: &[Value],
    id_fields: &[&str],
) {
    let slot = obj.entry(key.to_string()).or_insert_with(|| json!([]));
    let arr = match slot.as_array_mut() {
        Some(a) => a,
        None => return,
    };
    for item in incoming {
        if !arr.iter().any(|existing| composite_eq(existing, item, id_fields)) {
            arr.push(item.clone());
        }
    }
}

fn composite_eq(a: &Value, b: &Value, id_fields: &[&str]) -> bool {
    id_fields.iter().all(|f| a.get(*f) == b.get(*f))
}

fn list_sessions(project_root: &str) -> Result<String, String> {
    let path = resolve_memory_path(project_root);
    let data = read_memory(&path)?;

    let empty: Vec<Value> = Vec::new();
    let sessions = data
        .get("sessions")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty);

    let mut rows: Vec<Value> = sessions
        .iter()
        .map(|s| {
            let deliverables_count = s
                .get("deliverables")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            json!({
                "session_id": s.get("session_id").cloned().unwrap_or(Value::Null),
                "slug": s.get("slug").cloned().unwrap_or(Value::Null),
                "finished_at": s.get("finished_at").cloned().unwrap_or(Value::Null),
                "deliverables_count": deliverables_count,
            })
        })
        .collect();

    // Newest first — sessions[] is FIFO (index 0 oldest), so reverse for output.
    rows.reverse();

    serde_json::to_string_pretty(&Value::Array(rows))
        .map_err(|e| format!("JSON serialize error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // ---- helpers ------------------------------------------------------------

    static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Create a unique temp directory under std::env::temp_dir() and return its path.
    /// Caller is responsible for cleanup via `cleanup_tmp`.
    fn unique_tmp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!(
            "compass-cli-memtest-{}-{}-{}-{}",
            tag, pid, nanos, n
        ));
        fs::create_dir_all(&dir).expect("create unique tmp dir");
        dir
    }

    fn cleanup_tmp(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).expect("valid json from command")
    }

    // ---- tests --------------------------------------------------------------

    #[test]
    fn deep_merge_scalars_override() {
        let mut dst = json!({"a": 1, "b": 2});
        deep_merge(&mut dst, &json!({"b": 99, "c": 3}));
        assert_eq!(dst, json!({"a": 1, "b": 99, "c": 3}));
    }

    #[test]
    fn deep_merge_arrays_concat() {
        let mut dst = json!({"xs": [1, 2]});
        deep_merge(&mut dst, &json!({"xs": [3]}));
        assert_eq!(dst, json!({"xs": [1, 2, 3]}));
    }

    #[test]
    fn fifo_preserves_aggregates_with_dedup() {
        let mut data = json!({
            "memory_version": "1.0",
            "sessions": [],
            "decisions": [],
            "discovered_conventions": [],
            "resolved_ambiguities": [],
            "glossary": {},
        });

        // Seed 11 sessions; the first one has a decision that must survive.
        let sessions = data
            .get_mut("sessions")
            .unwrap()
            .as_array_mut()
            .unwrap();
        sessions.push(json!({
            "session_id": "s0",
            "slug": "s0",
            "finished_at": "2026-01-01T00:00:00Z",
            "deliverables": [],
            "decisions": [{"topic": "T", "decision": "D", "rationale": "R", "session_id": "s0"}],
            "discovered_conventions": [],
            "resolved_ambiguities": [],
        }));
        for i in 1..=10 {
            sessions.push(json!({
                "session_id": format!("s{}", i),
                "slug": format!("s{}", i),
                "finished_at": "2026-01-01T00:00:00Z",
                "deliverables": [],
                "decisions": [],
                "discovered_conventions": [],
                "resolved_ambiguities": [],
            }));
        }
        enforce_fifo_and_aggregate(&mut data);

        assert_eq!(data["sessions"].as_array().unwrap().len(), 10);
        assert_eq!(data["sessions"][0]["session_id"], "s1");
        let agg = data["decisions"].as_array().unwrap();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0]["topic"], "T");

        // Running again with the same decision should NOT duplicate.
        enforce_fifo_and_aggregate(&mut data);
        assert_eq!(data["decisions"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn dot_path_lookup() {
        let v = json!({"sessions": [{"session_id": "abc"}]});
        assert_eq!(
            lookup_dot_path(&v, "sessions.0.session_id").unwrap(),
            &json!("abc")
        );
        assert!(lookup_dot_path(&v, "sessions.5").is_none());
    }

    /// REQ-04: `init` on a fresh project root creates
    /// `.compass/.state/project-memory.json` with the v1 skeleton.
    #[test]
    fn init_default() {
        let root = unique_tmp_dir("init_default");
        let root_str = root.to_string_lossy().to_string();

        let out = init(&root_str).expect("init ok");
        let out_val = parse(&out);
        assert_eq!(out_val["ok"], json!(true));
        assert_eq!(out_val["already_exists"], json!(false));

        let mem_path = root
            .join(".compass")
            .join(".state")
            .join("project-memory.json");
        assert!(mem_path.exists(), "memory file should exist at {}", mem_path.display());

        let content = fs::read_to_string(&mem_path).expect("read memory file");
        let data: Value = serde_json::from_str(&content).expect("valid json");

        assert_eq!(data["memory_version"], json!("1.0"));
        assert_eq!(data["sessions"], json!([]));
        // The 3 schema aggregates MUST be present as empty arrays.
        assert_eq!(data["decisions"], json!([]));
        assert_eq!(data["discovered_conventions"], json!([]));
        assert_eq!(data["resolved_ambiguities"], json!([]));
        // Glossary is a required object (may be empty).
        assert!(data["glossary"].is_object(), "glossary must be an object");
        // Timestamps required by schema.
        assert!(data["created_at"].is_string());
        assert!(data["updated_at"].is_string());

        cleanup_tmp(&root);
    }

    /// REQ-04: after 11 `update` calls each appending one session entry,
    /// `sessions.len() == 10` and the oldest entry is dropped (FIFO).
    #[test]
    fn fifo_rotate() {
        let root = unique_tmp_dir("fifo_rotate");
        let root_str = root.to_string_lossy().to_string();
        init(&root_str).expect("init ok");

        for i in 0..11 {
            let patch = json!({
                "sessions": [{
                    "session_id": format!("sess-{:02}", i),
                    "slug": format!("sess-{:02}", i),
                    "finished_at": "2026-01-01T00:00:00Z",
                    "deliverables": [],
                    "decisions": [],
                    "discovered_conventions": [],
                    "resolved_ambiguities": [],
                }]
            });
            update(&root_str, &patch.to_string()).expect("update ok");
        }

        let mem_path = root
            .join(".compass")
            .join(".state")
            .join("project-memory.json");
        let data: Value = serde_json::from_str(
            &fs::read_to_string(&mem_path).expect("read memory file"),
        )
        .expect("valid json");

        let sessions = data["sessions"].as_array().expect("sessions array");
        assert_eq!(sessions.len(), 10, "should cap at 10 after 11 updates");
        // The oldest (`sess-00`) must be gone; index 0 is now `sess-01`.
        assert_eq!(sessions[0]["session_id"], json!("sess-01"));
        assert_eq!(sessions[9]["session_id"], json!("sess-10"));

        cleanup_tmp(&root);
    }

    /// REQ-04: the session dropped by rotation carried 2 decisions +
    /// 1 discovered_convention; after rotation those survive at top level.
    #[test]
    fn preserve_aggregates() {
        let root = unique_tmp_dir("preserve_aggregates");
        let root_str = root.to_string_lossy().to_string();
        init(&root_str).expect("init ok");

        // Session 0 is the one that will be dropped by rotation.
        // Give it 2 decisions and 1 discovered_convention.
        let first = json!({
            "sessions": [{
                "session_id": "sess-00",
                "slug": "sess-00",
                "finished_at": "2026-01-01T00:00:00Z",
                "deliverables": [],
                "decisions": [
                    {"topic": "T1", "decision": "D1", "rationale": "R1", "session_id": "sess-00"},
                    {"topic": "T2", "decision": "D2", "rationale": "R2", "session_id": "sess-00"}
                ],
                "discovered_conventions": [
                    {"area": "A1", "convention": "C1", "source_session": "sess-00"}
                ],
                "resolved_ambiguities": []
            }]
        });
        update(&root_str, &first.to_string()).expect("update ok (sess-00)");

        // Ten more plain sessions; this causes sess-00 to be rotated out.
        for i in 1..=10 {
            let patch = json!({
                "sessions": [{
                    "session_id": format!("sess-{:02}", i),
                    "slug": format!("sess-{:02}", i),
                    "finished_at": "2026-01-01T00:00:00Z",
                    "deliverables": [],
                    "decisions": [],
                    "discovered_conventions": [],
                    "resolved_ambiguities": []
                }]
            });
            update(&root_str, &patch.to_string()).expect("update ok");
        }

        let mem_path = root
            .join(".compass")
            .join(".state")
            .join("project-memory.json");
        let data: Value = serde_json::from_str(
            &fs::read_to_string(&mem_path).expect("read memory file"),
        )
        .expect("valid json");

        // Rotation happened.
        let sessions = data["sessions"].as_array().expect("sessions array");
        assert_eq!(sessions.len(), 10);
        assert!(
            !sessions
                .iter()
                .any(|s| s["session_id"] == json!("sess-00")),
            "sess-00 should have been rotated out"
        );

        // Aggregates preserved (dedup-aware: >= original counts).
        let decisions = data["decisions"].as_array().expect("decisions array");
        assert!(
            decisions.len() >= 2,
            "expected >=2 top-level decisions after rotation, got {}",
            decisions.len()
        );
        let convs = data["discovered_conventions"]
            .as_array()
            .expect("discovered_conventions array");
        assert!(
            convs.len() >= 1,
            "expected >=1 top-level discovered_conventions after rotation, got {}",
            convs.len()
        );

        cleanup_tmp(&root);
    }

    /// REQ-04: `get --key sessions.0.session_id` returns the FIRST session's
    /// id — which, given FIFO semantics after a single append, is the most
    /// recently added one (the array has length 1 at that point).
    #[test]
    fn get_dot_path() {
        let root = unique_tmp_dir("get_dot_path");
        let root_str = root.to_string_lossy().to_string();
        init(&root_str).expect("init ok");

        let patch = json!({
            "sessions": [{
                "session_id": "most-recent-abc",
                "slug": "most-recent-abc",
                "finished_at": "2026-01-01T00:00:00Z",
                "deliverables": [],
                "decisions": [],
                "discovered_conventions": [],
                "resolved_ambiguities": []
            }]
        });
        update(&root_str, &patch.to_string()).expect("update ok");

        let raw = get(&root_str, Some("sessions.0.session_id")).expect("get ok");
        // `get` pretty-prints via serde_json; the returned string must parse
        // to a JSON string equal to the session id.
        let parsed: Value = serde_json::from_str(&raw).expect("get output is json");
        assert_eq!(parsed, json!("most-recent-abc"));

        cleanup_tmp(&root);
    }
}
