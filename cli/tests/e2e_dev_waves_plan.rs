//! E2E integration tests for the waves-based dev plan path.
//!
//! Exercises `validate plan` and `context pack` against a fixture plan that
//! uses the v1.0 `waves[].tasks[]` shape (no `memory_ref`). Confirms Wave 1's
//! validator + context changes work end-to-end against the release binary.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{fixture_root, run_cli};

fn combined(out: &std::process::Output) -> String {
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    s
}

#[test]
fn validate_waves_dev_plan_passes() {
    let path = fixture_root().join("plan_v1_dev_waves.json");
    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "validate plan should exit 0, got {:?}\nstdout: {}\nstderr: {}",
        out.status,
        stdout,
        stderr
    );

    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON output:\n{}\nerr: {}", stdout, e));

    let valid = parsed
        .get("valid")
        .or_else(|| parsed.get("ok"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(valid, "expected valid:true, got: {}", parsed);

    let violations_len = parsed
        .get("violations")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(
        violations_len,
        0,
        "violations should be empty, got: {:?}",
        parsed.get("violations")
    );
}

#[test]
fn context_pack_finds_waves_task() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");

    // Copy fixture to tempdir/plan.json so context pack can locate the task.
    let src = fixture_root().join("plan_v1_dev_waves.json");
    let dst = tmp.path().join("plan.json");
    std::fs::copy(&src, &dst).expect("copy plan.json into tempdir");

    // T1.context_pointers references "Cargo.toml" — provide a dummy so the
    // file slicer has something to read.
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"dummy\"\n",
    )
    .expect("write dummy Cargo.toml");

    let out = run_cli(&["context", "pack", tmp.path().to_str().unwrap(), "T1"]);
    assert!(
        out.status.success(),
        "context pack should exit 0, got {:?}\n{}",
        out.status,
        combined(&out)
    );

    let pack_path = tmp.path().join("T1.context.json");
    assert!(
        pack_path.exists(),
        "expected pack file at {}",
        pack_path.display()
    );

    let contents = std::fs::read_to_string(&pack_path).expect("read T1.context.json");
    let parsed: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("invalid pack JSON: {}\nerr: {}", contents, e));

    let files = parsed
        .get("files")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("pack should have .files object, got: {}", contents));
    assert!(
        !files.is_empty(),
        "pack should have >=1 file entry, got: {}",
        contents
    );
}
