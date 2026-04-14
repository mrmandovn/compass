//! Shared helpers for `cli/tests/e2e_*.rs` integration test binaries.
//!
//! Cargo treats every file directly under `cli/tests/` as its own integration
//! test binary. Because of that, this file is NOT a library — consumers pull
//! it in with a path-based module declaration:
//!
//! ```ignore
//! #[path = "e2e_common.rs"] mod e2e_common;
//! use e2e_common::{HomeGuard, tmp_project, run_cli, run_cli_ok, run_cli_err, fixture_root};
//! ```
//!
//! This avoids the `tests/common/mod.rs` convention (which Cargo treats
//! inconsistently on some platforms) while still giving every binary its own
//! copy of the helpers compiled against its own crate graph.
//!
//! Each integration-test binary is its own OS process, so cross-binary races
//! on `$HOME` cannot happen — but tests WITHIN a single binary run on
//! multiple threads by default, so `HOME_GUARD` serializes them.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Process-global lock for tests that mutate `$HOME`. Serializes tests within
/// one integration-test binary; cross-binary isolation is handled by Cargo
/// running each binary in its own process.
pub static HOME_GUARD: Mutex<()> = Mutex::new(());

/// RAII guard: locks `HOME_GUARD`, swaps `$HOME` to a fresh tempdir, and
/// restores the previous `$HOME` (and best-effort cwd) on drop.
pub struct HomeGuard {
    pub tmp: tempfile::TempDir,
    prev_home: Option<std::ffi::OsString>,
    prev_cwd: Option<PathBuf>,
    // Held for the lifetime of the guard. `'static` is sound because
    // `HOME_GUARD` is a `static`.
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl HomeGuard {
    /// Acquire the global HOME lock, create a fresh tempdir, and point
    /// `$HOME` at it. Panics on failure — these are test helpers.
    pub fn new() -> Self {
        // Recover from a poisoned mutex: a prior test panicked while holding
        // it, but the data we guard (a `()`) is fine to reuse.
        let lock = HOME_GUARD.lock().unwrap_or_else(|e| e.into_inner());

        let prev_home = std::env::var_os("HOME");
        let prev_cwd = std::env::current_dir().ok();

        let tmp = tempfile::tempdir().expect("create tempdir for HomeGuard");
        std::env::set_var("HOME", tmp.path());

        HomeGuard {
            tmp,
            prev_home,
            prev_cwd,
            _lock: lock,
        }
    }

    /// Path of the tempdir now pointed at by `$HOME`.
    pub fn home_path(&self) -> &Path {
        self.tmp.path()
    }
}

impl Default for HomeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        // Restore `$HOME` first so any Drop-time lookups see a sane value.
        match self.prev_home.take() {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        // Best-effort: restore cwd so tests that cd'd into the tempdir don't
        // leave a dangling cwd after the tempdir is unlinked.
        if let Some(cwd) = self.prev_cwd.take() {
            let _ = std::env::set_current_dir(cwd);
        }
        // `_lock` drops automatically, releasing HOME_GUARD.
    }
}

/// Resolve `cli/tests/fixtures/...` as an absolute path.
pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Create a tmp dir and seed a minimal valid Compass project rooted at it.
///
/// Layout:
/// - `<root>/.compass/.state/config.json` — v1.2.0 skeleton
/// - `<root>/prd/` (empty)
/// - `<root>/epics/` (empty)
///
/// Returns `(TempDir guard, project_root)`. Keep the `TempDir` alive for the
/// duration of the test — dropping it deletes the directory.
pub fn tmp_project(name: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("create tempdir for tmp_project");
    let root = tmp.path().to_path_buf();

    let state_dir = root.join(".compass").join(".state");
    std::fs::create_dir_all(&state_dir).expect("mkdir .compass/.state");
    std::fs::create_dir_all(root.join("prd")).expect("mkdir prd");
    std::fs::create_dir_all(root.join("epics")).expect("mkdir epics");

    // Build via serde_json so the name is safely escaped rather than
    // string-interpolated into JSON.
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

    std::fs::write(
        state_dir.join("config.json"),
        serde_json::to_vec_pretty(&cfg).expect("serialize config"),
    )
    .expect("write config.json");

    (tmp, root)
}

/// Absolute path to the release `compass-cli` binary.
fn compass_cli_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("release")
        .join("compass-cli")
}

/// Ensure the release binary exists; build it if not.
fn ensure_release_binary() -> PathBuf {
    let bin = compass_cli_path();
    if bin.exists() {
        return bin;
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let status = std::process::Command::new("cargo")
        .args(["build", "--release", "--bin", "compass-cli"])
        .current_dir(manifest_dir)
        .status()
        .expect("spawn `cargo build --release`");
    assert!(
        status.success(),
        "`cargo build --release --bin compass-cli` failed with {status}"
    );

    assert!(
        bin.exists(),
        "release binary still missing after build: {}",
        bin.display()
    );
    bin
}

/// Spawn the release `compass-cli` binary with the given args and return its
/// raw `Output`. Builds the binary on demand the first time it's needed.
pub fn run_cli(args: &[&str]) -> std::process::Output {
    let bin = ensure_release_binary();
    std::process::Command::new(&bin)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn {}: {e}", bin.display()))
}

/// Run the CLI, assert exit code 0, and return stdout as a `String`.
/// Panics with stdout+stderr on non-zero exit — test failure output is
/// useless without both streams.
pub fn run_cli_ok(args: &[&str]) -> String {
    let out = run_cli(args);
    if !out.status.success() {
        panic!(
            "run_cli_ok({:?}) failed: status={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    String::from_utf8(out.stdout).expect("stdout not valid UTF-8")
}

/// Run the CLI, assert exit code non-zero, and return stderr as a `String`.
/// Panics on unexpected success.
pub fn run_cli_err(args: &[&str]) -> String {
    let out = run_cli(args);
    if out.status.success() {
        panic!(
            "run_cli_err({:?}) unexpectedly succeeded\n--- stdout ---\n{}\n--- stderr ---\n{}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    String::from_utf8(out.stderr).expect("stderr not valid UTF-8")
}
