//! E2E validator gate test for the `/compass:check` workflow.
//!
//! Port of `tests/integration/e2e_check_validates_prd.sh`. Exercises the same
//! CLI call that `check.md`'s Step 3b invokes:
//!
//!   compass-cli validate prd <path>
//!
//! Covers REQ-02 (PRD taste gate: R-FLOW + R-XREF). Runs five scenarios —
//! one good PRD, two R-FLOW violations (prose + bullet), one R-XREF dangling,
//! one R-XREF valid — asserting the gate blocks on bad input and passes on
//! good input.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{fixture_root, run_cli};

#[test]
fn good_flow_passes() {
    let path = fixture_root().join("prd_good_flow.md");
    let out = run_cli(&["validate", "prd", path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("parse json: {e}\nstdout: {stdout}"));
    assert_eq!(v["ok"], true, "expected ok=true, got {}", stdout);
}

#[test]
fn bad_flow_prose_fails() {
    let path = fixture_root().join("prd_bad_flow_prose.md");
    let out = run_cli(&["validate", "prd", path.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "expected non-zero exit, got 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("R-FLOW"),
        "expected R-FLOW violation, got stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn bad_flow_bullet_fails() {
    let path = fixture_root().join("prd_bad_flow_bullet.md");
    let out = run_cli(&["validate", "prd", path.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "expected non-zero exit, got 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("R-FLOW"),
        "expected R-FLOW violation, got stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn xref_dangling_fails() {
    let path = fixture_root().join("prd_xref_dangling.md");
    let out = run_cli(&["validate", "prd", path.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "expected non-zero exit, got 0\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("R-XREF"),
        "expected R-XREF violation, got stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn xref_valid_passes() {
    let path = fixture_root().join("prd_xref_valid.md");
    let out = run_cli(&["validate", "prd", path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("parse json: {e}\nstdout: {stdout}"));
    assert_eq!(v["ok"], true, "expected ok=true, got {}", stdout);
}
