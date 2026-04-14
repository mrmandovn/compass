//! E2E test for `compass-cli migrate` (v0.5 -> v1.0).
//!
//! Port of `tests/integration/e2e_migrate_v05.sh`. Covers REQ-06:
//!
//!   1. First migrate rewrites every session's `plan.json` to `plan_version:
//!      "1.0"`, writes a `plan.v0.json` backup equal to the original plan, and
//!      creates `.compass/.state/project-memory.json`.
//!   2. Each migrated plan validates via `compass-cli validate plan`.
//!   3. A second migrate is a no-op: exit 0, logs "already v1.0, no-op" for
//!      each session, no `plan.v0.plan.v0.json` double-backup, and backup
//!      mtimes unchanged.
//!
//! Each test stages its OWN copy of the fixture into a fresh tempdir so the
//! tests are safe under `cargo test` parallelism.

#[path = "e2e_common.rs"]
mod e2e_common;

use std::path::{Path, PathBuf};

use e2e_common::{fixture_root, run_cli};

/// Recursively copy `src` into `dst`. `dst` is created if it does not exist.
/// Rust stdlib has no such helper, hence this tiny clone of `cp -R src/. dst/`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Stage the v0 fixture into a fresh tempdir. Returns `(TempDir guard,
/// work_dir)`. Keep the guard alive for the duration of the test.
fn setup() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("create tempdir for migrate e2e");
    let work_dir = tmp.path().join("compass-migrate-test");
    let fixture_src = fixture_root().join("v0_project");
    assert!(
        fixture_src.is_dir(),
        "fixture missing: {}",
        fixture_src.display()
    );
    copy_dir_recursive(&fixture_src, &work_dir).expect("copy v0 fixture into tempdir");
    (tmp, work_dir)
}

/// List session directories under `<work_dir>/.compass/.state/sessions`, sorted
/// lexicographically (matching the bash `find ... | sort`).
fn list_sessions(work_dir: &Path) -> Vec<PathBuf> {
    let base = work_dir.join(".compass").join(".state").join("sessions");
    let mut out: Vec<PathBuf> = std::fs::read_dir(&base)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", base.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    out.sort();
    out
}

/// Read a top-level string field from a JSON file.
fn read_json_string_field(path: &Path, field: &str) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get(field).and_then(|x| x.as_str()).map(str::to_string)
}

#[test]
fn first_migrate_rewrites_plans_and_creates_memory() {
    let (_guard, work_dir) = setup();

    let sessions = list_sessions(&work_dir);
    assert!(
        sessions.len() >= 2,
        "fixture should provide >= 2 session dirs, got {}",
        sessions.len()
    );

    // Capture originals + assert precondition plan_version == "0.5".
    let mut originals: Vec<String> = Vec::with_capacity(sessions.len());
    for sd in &sessions {
        let plan_path = sd.join("plan.json");
        let orig = std::fs::read_to_string(&plan_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", plan_path.display()));
        let v = read_json_string_field(&plan_path, "plan_version");
        assert_eq!(
            v.as_deref(),
            Some("0.5"),
            "fixture precondition: {} expected plan_version 0.5, got {:?}",
            plan_path.display(),
            v
        );
        originals.push(orig);
    }

    // Run migrate — must exit 0.
    let out = run_cli(&["migrate", work_dir.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "first migrate expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // Per-session assertions: plan.json is v1.0, plan.v0.json matches original.
    for (i, sd) in sessions.iter().enumerate() {
        let plan_path = sd.join("plan.json");
        let backup_path = sd.join("plan.v0.json");

        assert!(
            plan_path.is_file(),
            "plan.json missing after migrate: {}",
            plan_path.display()
        );
        assert!(
            backup_path.is_file(),
            "plan.v0.json backup missing after migrate: {}",
            backup_path.display()
        );

        let v = read_json_string_field(&plan_path, "plan_version");
        assert_eq!(
            v.as_deref(),
            Some("1.0"),
            "expected plan_version 1.0 in {}, got {:?}",
            plan_path.display(),
            v
        );

        let backup_raw = std::fs::read_to_string(&backup_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", backup_path.display()));
        assert_eq!(
            backup_raw, originals[i],
            "{} does not match original plan.json",
            backup_path.display()
        );
    }

    // project-memory.json must be created.
    let memory_path = work_dir
        .join(".compass")
        .join(".state")
        .join("project-memory.json");
    assert!(
        memory_path.is_file(),
        "project-memory.json missing after migrate: {}",
        memory_path.display()
    );
}

#[test]
fn migrated_plans_validate_v1() {
    let (_guard, work_dir) = setup();

    // Migrate first — prerequisite for this test.
    let out = run_cli(&["migrate", work_dir.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "migrate failed: {:?}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let sessions = list_sessions(&work_dir);
    assert!(sessions.len() >= 2, "need >= 2 sessions");

    for sd in &sessions {
        let plan_path = sd.join("plan.json");
        let out = run_cli(&["validate", "plan", plan_path.to_str().unwrap()]);
        assert!(
            out.status.success(),
            "validate plan exited {:?} for {}\nstdout: {}\nstderr: {}",
            out.status,
            plan_path.display(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
}

#[test]
fn second_migrate_is_idempotent() {
    let (_guard, work_dir) = setup();

    // First migrate.
    let out1 = run_cli(&["migrate", work_dir.to_str().unwrap()]);
    assert!(
        out1.status.success(),
        "first migrate failed: {:?}\nstdout: {}\nstderr: {}",
        out1.status,
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr),
    );

    let sessions = list_sessions(&work_dir);
    assert!(sessions.len() >= 2, "need >= 2 sessions");

    // Snapshot backup mtimes.
    let mtimes_before: Vec<std::time::SystemTime> = sessions
        .iter()
        .map(|sd| {
            let bp = sd.join("plan.v0.json");
            std::fs::metadata(&bp)
                .unwrap_or_else(|e| panic!("stat {}: {e}", bp.display()))
                .modified()
                .expect("backup mtime")
        })
        .collect();

    // Second migrate — must succeed and log "already v1.0, no-op" per session.
    let out2 = run_cli(&["migrate", work_dir.to_str().unwrap()]);
    assert!(
        out2.status.success(),
        "second migrate expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        out2.status,
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr),
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr),
    );

    for sd in &sessions {
        let slug = sd
            .file_name()
            .and_then(|s| s.to_str())
            .expect("session dir name");
        // Bash used: grep -qE "${slug}:[[:space:]]+already v1.0, no-op"
        // Check that somewhere in the combined output a line contains
        // "<slug>:" followed by whitespace then "already v1.0, no-op".
        let needle_ok = combined.lines().any(|line| {
            if let Some(idx) = line.find(&format!("{slug}:")) {
                let after = &line[idx + slug.len() + 1..];
                let trimmed = after.trim_start();
                after.len() != trimmed.len() && trimmed.contains("already v1.0, no-op")
            } else {
                false
            }
        });
        assert!(
            needle_ok,
            "expected '{slug}: already v1.0, no-op' on second run\n--- combined ---\n{combined}",
        );
    }

    // No double-backup: `plan.v0.plan.v0.json` must not exist anywhere under
    // `work_dir`.
    fn assert_no_double_backup(dir: &Path) {
        for entry in std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        {
            let entry = entry.expect("dir entry");
            let ft = entry.file_type().expect("file type");
            if ft.is_dir() {
                assert_no_double_backup(&entry.path());
            } else if ft.is_file() {
                let name = entry.file_name();
                assert_ne!(
                    name.to_string_lossy(),
                    "plan.v0.plan.v0.json",
                    "found double-backup at {}",
                    entry.path().display()
                );
            }
        }
    }
    assert_no_double_backup(&work_dir);

    // Backups must still exist with unchanged mtimes.
    for (i, sd) in sessions.iter().enumerate() {
        let backup_path = sd.join("plan.v0.json");
        assert!(
            backup_path.is_file(),
            "{} vanished after second migrate",
            backup_path.display()
        );
        let m_after = std::fs::metadata(&backup_path)
            .unwrap_or_else(|e| panic!("stat {}: {e}", backup_path.display()))
            .modified()
            .expect("backup mtime after");
        assert_eq!(
            m_after, mtimes_before[i],
            "{} mtime changed across idempotent migrate",
            backup_path.display()
        );
    }
}
