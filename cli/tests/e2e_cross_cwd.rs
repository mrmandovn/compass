//! E2E — resolve works across cwd (shell cwd ≠ project root) + zero-effort
//! v1.1 → v1.2 registry migration. Ported from `tests/integration/e2e_cross_cwd.sh`.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{run_cli_ok, HomeGuard};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("invalid JSON: {e}\n---\n{out}"))
}

fn make_project(root: &Path, name: &str, prefix: &str) {
    let state = root.join(".compass").join(".state");
    fs::create_dir_all(&state).expect("mkdir .compass/.state");
    fs::create_dir_all(root.join("prd")).expect("mkdir prd");
    fs::create_dir_all(root.join("epics")).expect("mkdir epics");

    let cfg = serde_json::json!({
        "version": "1.2.0",
        "project": {"name": name, "po": "@t"},
        "lang": "en", "spec_lang": "en", "mode": "standalone",
        "prefix": prefix, "domain": "ard",
        "output_paths": {"prd": "prd", "story": "epics/{EPIC}/user-stories", "epic": "epics"},
        "naming": {"prd": "{slug}.md"}
    });
    fs::write(
        state.join("config.json"),
        serde_json::to_vec_pretty(&cfg).expect("serialize config"),
    )
    .expect("write config.json");
}

/// Covers REQ-06, REQ-07, REQ-10 — the whole cross-cwd lifecycle as a single
/// test since the stages share state (registry mutations accumulate).
#[test]
fn resolve_works_across_cwd_and_handles_v11_migration() {
    let _g = HomeGuard::new();

    let tmp = TempDir::new().expect("tempdir");
    let dir_a = tmp.path().join("project-a");
    let dir_b = tmp.path().join("random-dir-b");
    fs::create_dir_all(&dir_b).expect("mkdir dir_b");
    make_project(&dir_a, "ProjectA", "PA");

    // --- Stage 1: register + activate ProjectA --------------------------
    run_cli_ok(&["project", "add", dir_a.to_str().unwrap()]);
    run_cli_ok(&["project", "use", dir_a.to_str().unwrap()]);

    // --- Stage 2: resolve from dir_b (unrelated sibling) ----------------
    std::env::set_current_dir(&dir_b).expect("cd dir_b");
    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok", "expected ok from sibling cwd; got {r}");
    let project_root = r["project_root"].as_str().expect("project_root string");
    assert!(
        project_root.contains("project-a"),
        "project_root should point at project-a, got {project_root}"
    );
    assert_eq!(r["name"], "ProjectA");

    // --- Stage 3: resolve from /tmp (even more unrelated cwd) -----------
    std::env::set_current_dir("/tmp").expect("cd /tmp");
    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok", "expected ok from /tmp; got {r}");

    // --- Stage 4: writes to $PROJECT_ROOT land at dir_a ----------------
    let prd_path = dir_a.join("prd").join("test-prd.md");
    fs::write(&prd_path, "# test\n").expect("write PRD");
    assert!(prd_path.exists(), "PRD should exist at {}", prd_path.display());

    // --- Stage 5: zero-effort v1.1 → registry migration -----------------
    let home = std::env::var("HOME").expect("HOME set by HomeGuard");
    let registry = Path::new(&home).join(".compass").join("projects.json");
    if registry.exists() {
        fs::remove_file(&registry).expect("rm registry");
    }

    std::env::set_current_dir(&dir_a).expect("cd dir_a");
    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok", "auto-migrate should succeed; got {r}");
    assert_eq!(r["migrated_from_v11"], true, "migrated_from_v11 should be true");
    assert!(
        registry.exists(),
        "registry should have been auto-created at {}",
        registry.display()
    );
}
