//! E2E plan-time validator tests for the `/compass:run` workflow.
//!
//! Ported 1-1 from `tests/integration/e2e_run_missing_context.sh` (REQ-06).
//!
//! Covers the static rules that `compass-cli validate plan` enforces at Step 1
//! of `run.md`:
//!   1. A plan whose `context_pointers` references a non-existent file STILL
//!      passes static validation — file existence is a run-phase concern.
//!   2. A plan with `context_pointers: []` is rejected with
//!      `EMPTY_CONTEXT_POINTERS`.
//!   3. A plan with the `context_pointers` field entirely missing is rejected
//!      with `MISSING_CONTEXT_POINTERS`.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{fixture_root, run_cli};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Write `content` to `<tmp>/<name>` and return the absolute path.
fn write_plan(tmp: &TempDir, name: &str, content: &str) -> PathBuf {
    let p = tmp.path().join(name);
    fs::write(&p, content).unwrap_or_else(|e| panic!("write {}: {e}", p.display()));
    p
}

/// Load the valid plan fixture as a mutable `serde_json::Value`.
fn load_valid_plan() -> serde_json::Value {
    let raw = fs::read_to_string(fixture_root().join("plan_v1_valid.json"))
        .expect("read plan_v1_valid.json fixture");
    serde_json::from_str(&raw).expect("parse plan_v1_valid.json")
}

/// Serialize a plan `Value` to pretty JSON.
fn plan_to_string(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).expect("serialize plan")
}

// -----------------------------------------------------------------------------
// Test 1 — non-existent context_pointer file STILL passes static validation.
// The plan shape is valid; the pointer just names a file that isn't on disk.
// File-existence is a run-phase concern enforced by the workflow via
// AskUserQuestion, not by `validate plan`.
// -----------------------------------------------------------------------------
#[test]
fn plan_with_nonexistent_pointer_passes_static() {
    let tmp = TempDir::new().expect("create tempdir");
    let mut v = load_valid_plan();
    v["waves"][0]["tasks"][0]["context_pointers"] =
        serde_json::json!(["./does-not-exist.md"]);
    let path = write_plan(&tmp, "plan.json", &plan_to_string(&v));

    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);

    assert!(
        out.status.success(),
        "expected exit 0 (static rule passes); status={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

// -----------------------------------------------------------------------------
// Test 2 — context_pointers: [] → EMPTY_CONTEXT_POINTERS, non-zero exit.
// -----------------------------------------------------------------------------
#[test]
fn plan_with_empty_pointers_fails() {
    let tmp = TempDir::new().expect("create tempdir");
    let mut v = load_valid_plan();
    v["waves"][0]["tasks"][0]["context_pointers"] = serde_json::json!([]);
    let path = write_plan(&tmp, "plan.json", &plan_to_string(&v));

    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);

    assert!(
        !out.status.success(),
        "expected non-zero exit for empty context_pointers; got success\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        combined.contains("EMPTY_CONTEXT_POINTERS"),
        "expected EMPTY_CONTEXT_POINTERS violation in output; got:\n{combined}"
    );
}

// -----------------------------------------------------------------------------
// Test 3 — context_pointers field missing → MISSING_CONTEXT_POINTERS,
// non-zero exit.
// -----------------------------------------------------------------------------
#[test]
fn plan_missing_pointers_field_fails() {
    let tmp = TempDir::new().expect("create tempdir");
    let mut v = load_valid_plan();
    // Remove the context_pointers field entirely from waves[0].tasks[0].
    let task = v["waves"][0]["tasks"][0]
        .as_object_mut()
        .expect("waves[0].tasks[0] is a JSON object");
    task.remove("context_pointers");
    let path = write_plan(&tmp, "plan.json", &plan_to_string(&v));

    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);

    assert!(
        !out.status.success(),
        "expected non-zero exit for missing context_pointers field; got success\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        combined.contains("MISSING_CONTEXT_POINTERS"),
        "expected MISSING_CONTEXT_POINTERS violation in output; got:\n{combined}"
    );
}
