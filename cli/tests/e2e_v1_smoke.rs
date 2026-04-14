//! E2E smoke test — v1.0 end-to-end CLI surface exercise.
//!
//! Port of `tests/integration/e2e_v1_smoke.sh`. The Compass v1.0 workflows
//! (`/compass:init`, `/compass:brief`, `/compass:plan`, `/compass:prd`) are
//! LLM-interpreted and cannot run headless, but they all call into the same
//! compass-cli surfaces for memory, validation, and domain composition.
//! This smoke test exercises every CLI surface those workflows rely on
//! against a fresh empty project to prove the underlying gears work end-to-end.
//!
//! Covers REQ-01, REQ-03, REQ-04, REQ-06, REQ-07 — 7 sub-tests matching the
//! bash original 1-1.
//!
//! Note on $HOME isolation: every `memory {init,update,get}` invocation below
//! passes an explicit project path as a positional argument, so the CLI never
//! needs to resolve `$HOME` for these tests. `HomeGuard` is therefore not
//! required. The `validate plan|prd` paths also take explicit fixture paths.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{fixture_root, run_cli};

/// Helper: combine stdout+stderr into one `String` the way the bash script's
/// `2>&1` redirect does.
fn combined(out: &std::process::Output) -> String {
    let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    s
}

/// Test 1 — `memory init <tmp>` creates `.compass/.state/project-memory.json`
/// with `memory_version` = "1.0".
#[test]
fn memory_init_creates_file_with_version() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let proj = tmp.path().to_str().unwrap();

    let out = run_cli(&["memory", "init", proj]);
    assert!(
        out.status.success(),
        "memory init failed: {:?}\n{}",
        out.status,
        combined(&out)
    );

    let mem_path = tmp
        .path()
        .join(".compass")
        .join(".state")
        .join("project-memory.json");
    assert!(
        mem_path.exists(),
        "expected project-memory.json at {}",
        mem_path.display()
    );

    let raw = std::fs::read_to_string(&mem_path).expect("read project-memory.json");
    let v: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse json: {e}\nraw: {raw}"));
    assert_eq!(
        v["memory_version"], "1.0",
        "expected memory_version=\"1.0\", got {}",
        v["memory_version"]
    );
}

/// Test 2 — `memory init` is idempotent on a second call against the same
/// project: exit 0 and stdout/stderr contains the word "already".
#[test]
fn memory_init_is_idempotent() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let proj = tmp.path().to_str().unwrap();

    // First init — must succeed.
    let first = run_cli(&["memory", "init", proj]);
    assert!(
        first.status.success(),
        "first memory init failed: {}",
        combined(&first)
    );

    // Second init — must also succeed AND announce the existing file.
    let second = run_cli(&["memory", "init", proj]);
    assert!(
        second.status.success(),
        "second memory init failed: {:?}\n{}",
        second.status,
        combined(&second)
    );
    let out = combined(&second);
    assert!(
        out.contains("already"),
        "expected output to contain 'already', got: {out}"
    );
}

/// Test 3 — 11 session updates FIFO-cap to exactly 10.
#[test]
fn memory_update_applies_fifo_cap_of_10() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let proj = tmp.path().to_str().unwrap();

    // Init fresh project memory.
    let init = run_cli(&["memory", "init", proj]);
    assert!(
        init.status.success(),
        "memory init failed: {}",
        combined(&init)
    );

    // Push 11 single-session patches; cap should hold at 10 (FIFO-drop oldest).
    for i in 1..=11 {
        let nn = format!("{:02}", i);
        let patch = format!(
            "{{\"sessions\":[{{\"session_id\":\"sess-{nn}\",\"note\":\"iter-{nn}\"}}]}}"
        );
        let out = run_cli(&["memory", "update", proj, "--patch", &patch]);
        assert!(
            out.status.success(),
            "memory update iter {nn} failed: {:?}\n{}",
            out.status,
            combined(&out)
        );
    }

    // Read back the whole sessions array and assert length == 10.
    let get_out = run_cli(&["memory", "get", proj, "--key", "sessions"]);
    assert!(
        get_out.status.success(),
        "memory get sessions failed: {:?}\n{}",
        get_out.status,
        combined(&get_out)
    );
    let stdout = String::from_utf8_lossy(&get_out.stdout);
    let sessions: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("parse sessions json: {e}\nstdout: {stdout}"));
    let arr = sessions
        .as_array()
        .unwrap_or_else(|| panic!("expected sessions to be an array, got: {stdout}"));
    assert_eq!(
        arr.len(),
        10,
        "expected 10 sessions after FIFO cap, got {}\nstdout: {stdout}",
        arr.len()
    );
}

/// Test 4 — `memory get --key sessions.0.session_id` resolves a dot-path and
/// returns a non-empty string matching `sess-NN`.
#[test]
fn memory_get_resolves_dot_path() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let proj = tmp.path().to_str().unwrap();

    // Re-seed the same 11-session history so sessions.0 exists.
    let init = run_cli(&["memory", "init", proj]);
    assert!(
        init.status.success(),
        "memory init failed: {}",
        combined(&init)
    );
    for i in 1..=11 {
        let nn = format!("{:02}", i);
        let patch = format!(
            "{{\"sessions\":[{{\"session_id\":\"sess-{nn}\",\"note\":\"iter-{nn}\"}}]}}"
        );
        let out = run_cli(&["memory", "update", proj, "--patch", &patch]);
        assert!(
            out.status.success(),
            "memory update iter {nn} failed: {}",
            combined(&out)
        );
    }

    let out = run_cli(&["memory", "get", proj, "--key", "sessions.0.session_id"]);
    assert!(
        out.status.success(),
        "memory get dot-path failed: {:?}\n{}",
        out.status,
        combined(&out)
    );

    // Trim surrounding whitespace and optional JSON quotes, matching the
    // bash script's `tr -d '[:space:]' | sed -E 's/^"(.*)"$/\1/'`.
    let raw = String::from_utf8_lossy(&out.stdout);
    let trimmed: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    let val = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(&trimmed);

    assert!(
        !val.is_empty(),
        "expected non-empty string for sessions.0.session_id, got: {raw}"
    );

    // Must look like one of the session ids we wrote (sess-NN).
    let shape_ok = val.len() == 7
        && val.starts_with("sess-")
        && val[5..].chars().all(|c| c.is_ascii_digit());
    assert!(
        shape_ok,
        "expected sess-NN, got '{val}' (raw: {raw})"
    );
}

/// Test 5 — `validate plan plan_v1_valid.json` exits 0.
#[test]
fn validate_plan_v1_valid_passes() {
    let path = fixture_root().join("plan_v1_valid.json");
    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "expected exit 0 for valid plan, got {:?}\n{}",
        out.status,
        combined(&out)
    );
}

/// Test 6 — `validate plan plan_missing_pointers.json` exits non-zero and
/// reports `MISSING_CONTEXT_POINTERS`.
#[test]
fn validate_plan_missing_pointers_fails() {
    let path = fixture_root().join("plan_missing_pointers.json");
    let out = run_cli(&["validate", "plan", path.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "expected non-zero exit for missing-pointers plan, got 0\n{}",
        combined(&out)
    );
    let all = combined(&out);
    assert!(
        all.contains("MISSING_CONTEXT_POINTERS"),
        "expected stdout/stderr to contain 'MISSING_CONTEXT_POINTERS', got: {all}"
    );
}

/// Test 7 — `validate prd` accepts `prd_good_flow.md` and rejects
/// `prd_bad_flow_prose.md`. Kept as a single test to mirror the bash
/// original's test 7 (which contains sub-assertions 7a + 7b inline).
#[test]
fn validate_prd_good_and_bad_flow() {
    // 7a — good flow exits 0.
    let good = fixture_root().join("prd_good_flow.md");
    let out_good = run_cli(&["validate", "prd", good.to_str().unwrap()]);
    assert!(
        out_good.status.success(),
        "expected exit 0 for prd_good_flow, got {:?}\n{}",
        out_good.status,
        combined(&out_good)
    );

    // 7b — bad-flow prose exits non-zero.
    let bad = fixture_root().join("prd_bad_flow_prose.md");
    let out_bad = run_cli(&["validate", "prd", bad.to_str().unwrap()]);
    assert!(
        !out_bad.status.success(),
        "expected non-zero exit for prd_bad_flow_prose, got 0\n{}",
        combined(&out_bad)
    );
}
