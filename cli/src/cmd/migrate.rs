//! v0.x → v1.0 plan migration.
//!
//! Walks `<project_root>/.compass/.state/sessions/<slug>/plan.json` files and
//! rewrites each pre-1.0 plan into the v1.0 schema documented in
//! `core/shared/SCHEMAS-v1.md`. A `plan.v0.json` sibling backup is written
//! before the rewrite (idempotent: never overwrites an existing backup). The
//! top-level project memory file is ensured via `crate::cmd::memory::init`.
//!
//! Design rules (see REQ-05):
//! - Idempotent: re-running yields `already v1.0, no-op` per session.
//! - `PARSE_ERROR` on invalid JSON — do not touch the file.
//! - `NEWER_VERSION_THAN_CLI` on `plan_version` > "1.0" — exit 1.
//! - Missing `.compass/.state` — exit 0, emit a friendly log line.

use crate::cmd::memory;
use crate::helpers;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Entry point invoked from `main.rs` with the sub-args slice.
pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(usage());
    }
    match args[0].as_str() {
        "--help" | "-h" | "help" => Ok(help_text()),
        project_root => migrate(project_root),
    }
}

fn usage() -> String {
    "Usage: compass-cli migrate <project_root>".into()
}

fn help_text() -> String {
    "compass-cli migrate <project_root>\n\
     \n\
     Migrate a Compass project's on-disk state from v0.x to v1.0:\n  \
       * For each .compass/.state/sessions/<slug>/plan.json:\n    \
           - Back up to plan.v0.json (never overwrites).\n    \
           - Rewrite plan.json to the v1.0 schema (SCHEMAS-v1.md).\n    \
           - context_pointers is seeded with [\"TBD_BY_MIGRATE\"].\n  \
       * Ensure .compass/.state/project-memory.json exists.\n  \
       * Idempotent. Safe to re-run.\n\
     \n\
     Exit codes:\n  \
       0  success (or nothing to migrate)\n  \
       1  parse error, or plan_version newer than this CLI supports\n"
        .to_string()
}

/// Main driver — returns JSON summary on success, error string on failure.
fn migrate(project_root: &str) -> Result<String, String> {
    let state_dir = Path::new(project_root).join(".compass").join(".state");

    if !state_dir.exists() {
        eprintln!("no compass state found, nothing to migrate");
        return Ok(json!({
            "ok": true,
            "project_root": project_root,
            "state_dir": state_dir.to_string_lossy(),
            "sessions": [],
            "memory": null,
            "note": "no compass state found, nothing to migrate",
        })
        .to_string());
    }

    let sessions_dir = state_dir.join("sessions");
    let mut session_results: Vec<Value> = Vec::new();

    if sessions_dir.exists() {
        let mut entries: Vec<PathBuf> = fs::read_dir(&sessions_dir)
            .map_err(|e| format!("Cannot read {}: {}", sessions_dir.display(), e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        entries.sort();

        for session_dir in entries {
            let result = migrate_session(&session_dir)?;
            session_results.push(result);
        }
    }

    // Ensure the project-memory.json exists — delegate to the memory module
    // via its public `run` entry point so we don't touch its internals. The
    // init path is itself idempotent (returns already_exists: true on re-run).
    let memory_args = vec!["init".to_string(), project_root.to_string()];
    let memory_result_str = memory::run(&memory_args)?;
    let memory_result: Value = serde_json::from_str(&memory_result_str)
        .map_err(|e| format!("memory init returned non-JSON: {}", e))?;

    Ok(json!({
        "ok": true,
        "project_root": project_root,
        "state_dir": state_dir.to_string_lossy(),
        "sessions": session_results,
        "memory": memory_result,
    })
    .to_string())
}

/// Migrate a single session directory. The only I/O side effects are the
/// backup file and the rewritten plan.json — both gated on the plan being
/// pre-1.0.
fn migrate_session(session_dir: &Path) -> Result<Value, String> {
    let slug = session_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    let plan_path = session_dir.join("plan.json");

    if !plan_path.exists() {
        return Ok(json!({
            "session": slug,
            "status": "skipped",
            "reason": "no plan.json",
        }));
    }

    // Read + parse. PARSE_ERROR bubbles up as a hard failure so we don't
    // silently rewrite garbage.
    let raw = fs::read_to_string(&plan_path)
        .map_err(|e| format!("Cannot read {}: {}", plan_path.display(), e))?;
    let plan: Value = serde_json::from_str(&raw)
        .map_err(|e| format!("PARSE_ERROR: {}: {}", plan_path.display(), e))?;

    let version = plan.get("plan_version").and_then(|v| v.as_str());

    match version {
        Some("1.0") => {
            eprintln!("{}: already v1.0, no-op", slug);
            Ok(json!({
                "session": slug,
                "status": "already_v1",
            }))
        }
        Some(other) if is_newer_than_1_0(other) => Err(format!(
            "NEWER_VERSION_THAN_CLI: {} has plan_version '{}', this CLI only supports 1.0",
            plan_path.display(),
            other
        )),
        _ => {
            // Any pre-1.0 version (including missing plan_version) → migrate.
            let backup_path = session_dir.join("plan.v0.json");
            if !backup_path.exists() {
                fs::write(&backup_path, &raw)
                    .map_err(|e| format!("Cannot write {}: {}", backup_path.display(), e))?;
            }

            let migrated = build_v1_plan(&plan, &slug);
            helpers::write_json(&plan_path, &migrated)?;

            eprintln!("{}: migrated to v1.0", slug);
            Ok(json!({
                "session": slug,
                "status": "migrated",
                "backup": backup_path.to_string_lossy(),
                "from_version": version.unwrap_or(""),
            }))
        }
    }
}

/// Return true if `v` parses as a version strictly newer than 1.0 on the
/// MAJOR.MINOR axis. Unparseable → false (treated as pre-1.0 legacy).
fn is_newer_than_1_0(v: &str) -> bool {
    let mut parts = v.split('.');
    let major = parts.next().and_then(|s| s.parse::<u32>().ok());
    let minor = parts.next().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    match major {
        Some(m) if m > 1 => true,
        Some(1) if minor > 0 => true,
        _ => false,
    }
}

/// Translate a legacy plan into the v1.0 schema. The v0 shape we know from
/// fixtures is `colleagues[]` with `{id, type, budget_tokens, depends_on,
/// output_files, briefing{...}, acceptance{...}}`. We also honour the rarer
/// `tasks[]` variant defensively.
fn build_v1_plan(legacy: &Value, slug: &str) -> Value {
    let empty_vec: Vec<Value> = Vec::new();
    let legacy_tasks = legacy
        .get("colleagues")
        .and_then(|v| v.as_array())
        .or_else(|| legacy.get("tasks").and_then(|v| v.as_array()))
        .unwrap_or(&empty_vec);

    // Compute dependency layers → waves. Tasks whose deps are all already
    // scheduled land in the next wave. Tasks with no deps start in wave 1.
    let waves = layer_into_waves(legacy_tasks);

    // Collect unique colleague types ordered by first appearance.
    let mut seen: HashSet<String> = HashSet::new();
    let mut colleagues_selected: Vec<String> = Vec::new();
    for t in legacy_tasks {
        let colleague = t
            .get("type")
            .or_else(|| t.get("colleague"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if !colleague.is_empty() && seen.insert(colleague.clone()) {
            colleagues_selected.push(colleague);
        }
    }

    let wave_array: Vec<Value> = waves
        .into_iter()
        .enumerate()
        .map(|(i, ids)| {
            let wave_id = (i as u64) + 1;
            let tasks: Vec<Value> = ids
                .into_iter()
                .filter_map(|id| {
                    legacy_tasks
                        .iter()
                        .find(|t| task_id_of(t) == id)
                        .map(|t| task_to_v1(t, slug))
                })
                .collect();
            json!({
                "wave_id": wave_id,
                "tasks": tasks,
            })
        })
        .collect();

    json!({
        "plan_version": "1.0",
        "session_id": slug,
        "colleagues_selected": colleagues_selected,
        "memory_ref": ".compass/.state/project-memory.json",
        "domain": Value::Null,
        "waves": wave_array,
    })
}

fn task_id_of(t: &Value) -> String {
    t.get("task_id")
        .or_else(|| t.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn task_to_v1(legacy_task: &Value, slug: &str) -> Value {
    let task_id = task_id_of(legacy_task);
    let colleague = legacy_task
        .get("type")
        .or_else(|| legacy_task.get("colleague"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let budget = legacy_task
        .get("budget_tokens")
        .or_else(|| legacy_task.get("budget"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let depends_on: Vec<Value> = legacy_task
        .get("depends_on")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let briefing_notes = extract_briefing_notes(legacy_task);
    let output_pattern = legacy_task
        .get("output_files")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            legacy_task
                .get("output_pattern")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| format!("outputs/{}-{}.md", slug, task_id));

    json!({
        "task_id": task_id,
        "colleague": colleague,
        "budget": budget,
        "depends_on": depends_on,
        "briefing_notes": briefing_notes,
        // Placeholder per REQ-05: downstream tooling will re-populate with
        // real pointers the first time /compass:plan revisits the session.
        "context_pointers": ["TBD_BY_MIGRATE"],
        "output_pattern": output_pattern,
    })
}

/// Flatten the v0 `briefing` object into a single human-readable string. We
/// don't lose data — we just serialize it compactly. Downstream consumers
/// treat `briefing_notes` as free-form per SCHEMAS-v1.md.
fn extract_briefing_notes(legacy_task: &Value) -> String {
    let briefing = match legacy_task.get("briefing") {
        Some(b) => b,
        None => {
            return legacy_task
                .get("briefing_notes")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }
    };

    let mut parts: Vec<String> = Vec::new();
    if let Some(ctx) = briefing.get("context").and_then(|v| v.as_array()) {
        let items: Vec<String> = ctx
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !items.is_empty() {
            parts.push(format!("Context: {}", items.join(", ")));
        }
    }
    if let Some(cs) = briefing.get("constraints").and_then(|v| v.as_array()) {
        let items: Vec<String> = cs
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !items.is_empty() {
            parts.push(format!("Constraints: {}", items.join("; ")));
        }
    }
    if let Some(sh) = briefing.get("stakeholders").and_then(|v| v.as_array()) {
        let items: Vec<String> = sh
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        if !items.is_empty() {
            parts.push(format!("Stakeholders: {}", items.join(", ")));
        }
    }
    if let Some(d) = briefing.get("deadline").and_then(|v| v.as_str()) {
        parts.push(format!("Deadline: {}", d));
    }
    parts.join(" | ")
}

/// Kahn-ish topological layering: each output layer is the set of tasks whose
/// dependencies all belong to earlier layers. Preserves input order within a
/// layer. Tasks with unknown/dangling deps land in the first layer they're
/// eligible for (treating unknown ids as already-satisfied) so a malformed v0
/// plan still migrates rather than wedging.
fn layer_into_waves(tasks: &[Value]) -> Vec<Vec<String>> {
    let ids_in_order: Vec<String> = tasks.iter().map(task_id_of).collect();
    let id_set: HashSet<String> = ids_in_order.iter().cloned().collect();

    let deps: BTreeMap<String, Vec<String>> = tasks
        .iter()
        .map(|t| {
            let id = task_id_of(t);
            let d: Vec<String> = t
                .get("depends_on")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .filter(|s| id_set.contains(s))
                        .collect()
                })
                .unwrap_or_default();
            (id, d)
        })
        .collect();

    let mut placed: HashSet<String> = HashSet::new();
    let mut layers: Vec<Vec<String>> = Vec::new();

    while placed.len() < ids_in_order.len() {
        let mut layer: Vec<String> = Vec::new();
        for id in &ids_in_order {
            if placed.contains(id) {
                continue;
            }
            let ready = deps
                .get(id)
                .map(|ds| ds.iter().all(|d| placed.contains(d)))
                .unwrap_or(true);
            if ready {
                layer.push(id.clone());
            }
        }
        if layer.is_empty() {
            // Cycle or bug — dump the remainder into a final layer so we
            // don't loop forever. v0 plans aren't supposed to cycle.
            for id in &ids_in_order {
                if !placed.contains(id) {
                    layer.push(id.clone());
                }
            }
        }
        for id in &layer {
            placed.insert(id.clone());
        }
        layers.push(layer);
    }

    if layers.is_empty() {
        layers.push(Vec::new());
    }
    layers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_version_detection_smoke() {
        assert!(is_newer_than_1_0("1.1"));
        assert!(is_newer_than_1_0("2.0"));
        assert!(!is_newer_than_1_0("1.0"));
        assert!(!is_newer_than_1_0("0.5"));
        assert!(!is_newer_than_1_0("garbage"));
    }

    #[test]
    fn build_v1_preserves_dependency_order_smoke() {
        let legacy = json!({
            "plan_version": "0.5",
            "colleagues": [
                {"id": "C-01", "type": "researcher", "budget_tokens": 1000, "depends_on": [],
                 "output_files": ["research/x.md"], "briefing": {"context": ["a.md"]}},
                {"id": "C-02", "type": "writer", "budget_tokens": 2000, "depends_on": ["C-01"],
                 "output_files": ["PRDs/y.md"], "briefing": {"constraints": ["ship Q1"]}}
            ]
        });
        let v1 = build_v1_plan(&legacy, "slug");
        assert_eq!(v1["plan_version"], "1.0");
        assert_eq!(v1["session_id"], "slug");
        assert_eq!(v1["memory_ref"], ".compass/.state/project-memory.json");
        assert!(v1["domain"].is_null());
        let waves = v1["waves"].as_array().unwrap();
        assert_eq!(waves.len(), 2);
        assert_eq!(waves[0]["tasks"][0]["task_id"], "C-01");
        assert_eq!(waves[1]["tasks"][0]["task_id"], "C-02");
        assert_eq!(
            waves[0]["tasks"][0]["context_pointers"][0],
            "TBD_BY_MIGRATE"
        );
        assert_eq!(v1["colleagues_selected"][0], "researcher");
        assert_eq!(v1["colleagues_selected"][1], "writer");
    }

    #[test]
    fn idempotent_on_v1_input_smoke() {
        let dir = tempdir();
        let session = dir.join("sessions").join("s1");
        fs::create_dir_all(&session).unwrap();
        let v1 = json!({
            "plan_version": "1.0",
            "session_id": "s1",
            "colleagues_selected": [],
            "memory_ref": ".compass/.state/project-memory.json",
            "domain": null,
            "waves": [],
        });
        fs::write(session.join("plan.json"), serde_json::to_string(&v1).unwrap()).unwrap();
        let res = migrate_session(&session).unwrap();
        assert_eq!(res["status"], "already_v1");
        // Backup must NOT be created for already-v1 plans.
        assert!(!session.join("plan.v0.json").exists());
    }

    #[test]
    fn migrates_legacy_and_writes_backup_smoke() {
        let dir = tempdir();
        let session = dir.join("sessions").join("legacy");
        fs::create_dir_all(&session).unwrap();
        let legacy = json!({
            "plan_version": "0.5",
            "colleagues": [
                {"id": "C-01", "type": "writer", "budget_tokens": 1000,
                 "depends_on": [], "output_files": ["PRDs/x.md"], "briefing": {}}
            ]
        });
        let plan_path = session.join("plan.json");
        fs::write(&plan_path, serde_json::to_string(&legacy).unwrap()).unwrap();

        let res = migrate_session(&session).unwrap();
        assert_eq!(res["status"], "migrated");

        let rewritten: Value =
            serde_json::from_str(&fs::read_to_string(&plan_path).unwrap()).unwrap();
        assert_eq!(rewritten["plan_version"], "1.0");
        assert!(session.join("plan.v0.json").exists());

        // Running again is a no-op.
        let again = migrate_session(&session).unwrap();
        assert_eq!(again["status"], "already_v1");
    }

    #[test]
    fn parse_error_does_not_rewrite_smoke() {
        let dir = tempdir();
        let session = dir.join("sessions").join("broken");
        fs::create_dir_all(&session).unwrap();
        let plan_path = session.join("plan.json");
        fs::write(&plan_path, "{ this is not json").unwrap();
        let err = migrate_session(&session).unwrap_err();
        assert!(err.starts_with("PARSE_ERROR"), "got: {}", err);
        assert!(!session.join("plan.v0.json").exists());
    }

    #[test]
    fn newer_version_errors_out_smoke() {
        let dir = tempdir();
        let session = dir.join("sessions").join("future");
        fs::create_dir_all(&session).unwrap();
        let plan = json!({"plan_version": "2.0"});
        fs::write(
            session.join("plan.json"),
            serde_json::to_string(&plan).unwrap(),
        )
        .unwrap();
        let err = migrate_session(&session).unwrap_err();
        assert!(err.starts_with("NEWER_VERSION_THAN_CLI"), "got: {}", err);
    }

    // ---------- Full-coverage tests per TEST-SPEC T-14 ----------

    /// Recursively copy a directory tree. Used to stage the v0_project fixture
    /// into an isolated tmp dir so each test mutates its own copy.
    fn copy_dir_recursive(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let ft = entry.file_type().unwrap();
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if ft.is_dir() {
                copy_dir_recursive(&src_path, &dst_path);
            } else {
                fs::copy(&src_path, &dst_path).unwrap();
            }
        }
    }

    /// Path to the checked-in v0 fixture (two session dirs with v0.5 plans).
    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("v0_project")
    }

    /// Stage the fixture into a fresh tmp dir and return the staged root.
    fn stage_fixture(label: &str) -> (PathBuf, PathBuf) {
        let td = tempdir();
        let root = td.join(label);
        copy_dir_recursive(&fixture_root(), &root);
        (td, root)
    }

    #[test]
    fn v0_to_v1_plan() {
        let (_td, root) = stage_fixture("v0_to_v1_plan");
        let args = vec![root.to_string_lossy().to_string()];
        let summary_str = run(&args).expect("migrate should succeed on v0 fixture");
        let summary: Value = serde_json::from_str(&summary_str).unwrap();
        assert_eq!(summary["ok"], json!(true));

        let sessions_dir = root.join(".compass").join(".state").join("sessions");
        let session_entries: Vec<PathBuf> = fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        assert!(
            session_entries.len() >= 2,
            "fixture should provide >= 2 session dirs, got {}",
            session_entries.len()
        );

        for session_dir in &session_entries {
            let plan_path = session_dir.join("plan.json");
            let backup_path = session_dir.join("plan.v0.json");
            assert!(
                plan_path.exists(),
                "plan.json missing in {}",
                session_dir.display()
            );
            assert!(
                backup_path.exists(),
                "plan.v0.json backup missing in {}",
                session_dir.display()
            );

            let migrated: Value =
                serde_json::from_str(&fs::read_to_string(&plan_path).unwrap()).unwrap();
            assert_eq!(
                migrated["plan_version"], "1.0",
                "plan_version not rewritten in {}",
                session_dir.display()
            );
            assert!(
                migrated["domain"].is_null(),
                "domain should be null in {}",
                session_dir.display()
            );
            assert_eq!(
                migrated["memory_ref"], ".compass/.state/project-memory.json",
                "memory_ref not set in {}",
                session_dir.display()
            );

            // Every task in every wave should carry the TBD_BY_MIGRATE placeholder.
            let waves = migrated["waves"].as_array().expect("waves array");
            let mut saw_task = false;
            for wave in waves {
                for task in wave["tasks"].as_array().expect("tasks array") {
                    saw_task = true;
                    let cp = task["context_pointers"]
                        .as_array()
                        .expect("context_pointers array");
                    assert_eq!(cp.len(), 1, "context_pointers should be single placeholder");
                    assert_eq!(cp[0], "TBD_BY_MIGRATE");
                }
            }
            assert!(
                saw_task,
                "expected at least one task in migrated plan at {}",
                session_dir.display()
            );
        }
    }

    #[test]
    fn idempotent() {
        let (_td, root) = stage_fixture("idempotent");
        let args = vec![root.to_string_lossy().to_string()];

        // First run: perform the migration.
        let first = run(&args).expect("first migrate run should succeed");
        let first_json: Value = serde_json::from_str(&first).unwrap();
        assert_eq!(first_json["ok"], json!(true));

        // Capture each session's backup mtime before the second run so we can
        // later assert it was not rewritten (no double-backup).
        let sessions_dir = root.join(".compass").join(".state").join("sessions");
        let session_paths: Vec<PathBuf> = fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();

        // Second run: must be a no-op, exit 0, every session already_v1.
        let second = run(&args).expect("second migrate run should also succeed (exit 0)");
        let second_json: Value = serde_json::from_str(&second).unwrap();
        assert_eq!(second_json["ok"], json!(true));

        let statuses: Vec<&str> = second_json["sessions"]
            .as_array()
            .expect("sessions array")
            .iter()
            .map(|s| s.get("status").and_then(|v| v.as_str()).unwrap_or(""))
            .collect();
        assert!(
            !statuses.is_empty(),
            "expected session statuses on second run, got none"
        );
        for status in &statuses {
            assert_eq!(
                *status, "already_v1",
                "second run should report already_v1 for every session, got {:?}",
                statuses
            );
        }

        // No double-backup: plan.v0.plan.v0.json must NOT exist in any session.
        // Also: plan.v0.json must exist (from the first run) and must NOT have
        // been re-created as a nested backup.
        for session_dir in &session_paths {
            let nested = session_dir.join("plan.v0.plan.v0.json");
            assert!(
                !nested.exists(),
                "double-backup detected at {}",
                nested.display()
            );
            assert!(
                session_dir.join("plan.v0.json").exists(),
                "first-run backup missing at {}",
                session_dir.display()
            );
        }
    }

    #[test]
    fn creates_memory() {
        let (_td, root) = stage_fixture("creates_memory");
        let args = vec![root.to_string_lossy().to_string()];
        let summary_str = run(&args).expect("migrate should succeed");
        let summary: Value = serde_json::from_str(&summary_str).unwrap();
        assert_eq!(summary["ok"], json!(true));

        let memory_path = root
            .join(".compass")
            .join(".state")
            .join("project-memory.json");
        assert!(
            memory_path.exists(),
            "project-memory.json should be created at {}",
            memory_path.display()
        );

        let memory: Value =
            serde_json::from_str(&fs::read_to_string(&memory_path).unwrap()).unwrap();
        assert_eq!(
            memory["memory_version"], "1.0",
            "memory_version must be \"1.0\""
        );
        let sessions = memory["sessions"]
            .as_array()
            .expect("sessions must be an array");
        assert!(
            sessions.is_empty(),
            "newly created memory must have empty sessions, got {:?}",
            sessions
        );
    }

    #[test]
    fn corrupt_plan() {
        let (_td, root) = stage_fixture("corrupt_plan");
        let sessions_dir = root.join(".compass").join(".state").join("sessions");

        // Overwrite one of the fixture session plans with invalid JSON.
        let mut session_paths: Vec<PathBuf> = fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        session_paths.sort();
        let target_session = session_paths
            .first()
            .expect("fixture must provide at least one session");
        let plan_path = target_session.join("plan.json");
        let garbage = "{ this is not valid json";
        fs::write(&plan_path, garbage).unwrap();

        let args = vec![root.to_string_lossy().to_string()];
        let result = run(&args);
        assert!(
            result.is_err(),
            "migrate must fail (exit != 0) when a plan.json is corrupt"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("PARSE_ERROR"),
            "error must advertise PARSE_ERROR, got: {}",
            err
        );

        // File was NOT rewritten — raw bytes unchanged on disk.
        let on_disk = fs::read_to_string(&plan_path).unwrap();
        assert_eq!(
            on_disk, garbage,
            "corrupt plan.json must not be rewritten by migrate"
        );
        // And no backup was written for the corrupt file.
        assert!(
            !target_session.join("plan.v0.json").exists(),
            "no backup should be written for a plan that failed to parse"
        );
    }

    // Minimal tempdir helper — we avoid pulling the `tempfile` crate just for
    // smoke tests. Each test gets a unique subdir under the OS temp root.
    fn tempdir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let p = std::env::temp_dir().join(format!(
            "compass-migrate-test-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&p).unwrap();
        p
    }
}
