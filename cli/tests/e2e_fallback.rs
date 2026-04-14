//! E2E fallback — ambiguous fallback after last_active dies.
//!
//! Port of `tests/integration/e2e_fallback.sh`. Covers REQ-03, REQ-04, REQ-11.
//!
//! Scenarios exercised against `compass-cli project resolve`:
//!   1. Happy path: last_active alive returns status=ok.
//!   2. last_active dies → status=ambiguous, dead path pruned from registry.
//!   3. User picks a survivor → status=ok.
//!   4. Only one survivor left → smart auto-pick, status=ok.
//!   5. All registry paths dead → status=none, reason=all_paths_dead.
//!
//! All 5 stages share state — the registry mutates progressively across
//! stages — so they run as a single sequential test function.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{run_cli_ok, HomeGuard};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Seed a minimal valid v1.2.0 project config at `<root>/.compass/.state/config.json`.
fn make_project(root: &Path, name: &str) {
    let state = root.join(".compass").join(".state");
    fs::create_dir_all(&state).expect("mkdir .compass/.state");
    let cfg = serde_json::json!({
        "version": "1.2.0",
        "project": { "name": name, "po": "@t" },
        "lang": "en",
        "spec_lang": "en",
        "mode": "standalone",
        "prefix": "PX",
        "domain": "ard",
        "output_paths": {
            "prd": "prd",
            "story": "epics/{EPIC}/user-stories",
            "epic": "epics"
        },
        "naming": { "prd": "{slug}.md" }
    });
    fs::write(state.join("config.json"), cfg.to_string()).expect("write config.json");
}

#[test]
fn fallback_full_lifecycle() {
    let _guard = HomeGuard::new();

    // Parent dir holding the 3 fake projects. Kept alive via `tmp`.
    let tmp = TempDir::new().expect("create TMP_ROOT");
    let alpha = tmp.path().join("proj_alpha");
    let beta = tmp.path().join("proj_beta");
    let gamma = tmp.path().join("proj_gamma");

    // -------------------------------------------------------------------
    // Setup — three projects, each with a valid v1.2.0 config.json.
    // Register all three and mark beta as last_active.
    // -------------------------------------------------------------------
    make_project(&alpha, "proj_alpha");
    make_project(&beta, "proj_beta");
    make_project(&gamma, "proj_gamma");

    run_cli_ok(&["project", "add", alpha.to_str().unwrap()]);
    run_cli_ok(&["project", "add", beta.to_str().unwrap()]);
    run_cli_ok(&["project", "add", gamma.to_str().unwrap()]);
    run_cli_ok(&["project", "use", beta.to_str().unwrap()]);

    // -------------------------------------------------------------------
    // Test 1 — resolve happy path: last_active alive returns ok/beta.
    // -------------------------------------------------------------------
    let out = run_cli_ok(&["project", "resolve"]);
    let r: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("T1 resolve stdout not JSON: {e}\n{out}"));
    assert_eq!(r["status"], "ok", "T1 expected ok, got {}", r["status"]);
    let active = r["project_root"]
        .as_str()
        .expect("T1 project_root missing");
    assert!(
        active.contains("proj_beta"),
        "T1 expected beta active, got {active}"
    );

    // -------------------------------------------------------------------
    // Test 2 — kill last_active, expect ambiguous with 2 survivors + pruning.
    // -------------------------------------------------------------------
    fs::remove_dir_all(&beta).expect("rm beta");
    let out = run_cli_ok(&["project", "resolve"]);
    let r: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("T2 resolve stdout not JSON: {e}\n{out}"));
    assert_eq!(
        r["status"], "ambiguous",
        "T2 expected ambiguous, got {}",
        r["status"]
    );
    let cands = r["candidates"]
        .as_array()
        .expect("T2 candidates not array");
    assert_eq!(cands.len(), 2, "T2 expected 2 candidates, got {}", cands.len());

    // Registry should have pruned the dead beta entry.
    let home = std::env::var("HOME").expect("HOME set");
    let registry_path = Path::new(&home).join(".compass").join("projects.json");
    let registry_raw = fs::read_to_string(&registry_path).expect("read registry");
    let registry: serde_json::Value =
        serde_json::from_str(&registry_raw).expect("registry is JSON");
    let paths: Vec<String> = registry["projects"]
        .as_array()
        .expect("projects array")
        .iter()
        .map(|p| p["path"].as_str().unwrap_or("").to_string())
        .collect();
    let joined = paths.join(",");
    assert!(
        !joined.contains("proj_beta"),
        "T2 beta should have been pruned from registry, paths={joined}"
    );

    // -------------------------------------------------------------------
    // Test 3 — user picks alpha; resolve returns ok/alpha.
    // -------------------------------------------------------------------
    run_cli_ok(&["project", "use", alpha.to_str().unwrap()]);
    let out = run_cli_ok(&["project", "resolve"]);
    let r: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("T3 resolve stdout not JSON: {e}\n{out}"));
    assert_eq!(
        r["status"], "ok",
        "T3 expected ok after use alpha, got {}",
        r["status"]
    );
    let active = r["project_root"]
        .as_str()
        .expect("T3 project_root missing");
    assert!(
        active.contains("proj_alpha"),
        "T3 expected alpha active, got {active}"
    );

    // -------------------------------------------------------------------
    // Test 4 — kill alpha too; only gamma survives → smart auto-pick.
    // -------------------------------------------------------------------
    fs::remove_dir_all(&alpha).expect("rm alpha");
    let out = run_cli_ok(&["project", "resolve"]);
    let r: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("T4 resolve stdout not JSON: {e}\n{out}"));
    assert_eq!(
        r["status"], "ok",
        "T4 expected ok (smart auto-pick single survivor), got {}",
        r["status"]
    );
    let active = r["project_root"]
        .as_str()
        .expect("T4 project_root missing");
    assert!(
        active.contains("proj_gamma"),
        "T4 expected gamma active after auto-pick, got {active}"
    );

    // -------------------------------------------------------------------
    // Test 5 — kill gamma; no survivors → status=none, reason=all_paths_dead.
    // -------------------------------------------------------------------
    fs::remove_dir_all(&gamma).expect("rm gamma");
    let out = run_cli_ok(&["project", "resolve"]);
    let r: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|e| panic!("T5 resolve stdout not JSON: {e}\n{out}"));
    assert_eq!(
        r["status"], "none",
        "T5 expected none, got {}",
        r["status"]
    );
    assert_eq!(
        r["reason"], "all_paths_dead",
        "T5 expected all_paths_dead, got {}",
        r["reason"]
    );
}
