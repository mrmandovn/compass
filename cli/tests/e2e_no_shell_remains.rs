//! Regression guard — Compass is a Rust + JS-launcher project. No `.sh`
//! files should ever land in the repo. This test walks the repo from
//! `cli/`'s parent (the repo root) and fails if any `.sh` file is present
//! outside `target/` and `.git/`. Catches accidental re-introduction of
//! bash artifacts in any future change.
//!
//! Covers REQ-01, REQ-02, REQ-05, REQ-07.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has a parent (the repo root)")
        .to_path_buf()
}

fn collect_shell_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name == "target" || name == ".git" || name == "node_modules" {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shell_files(&path, found);
        } else if path.extension().and_then(|e| e.to_str()) == Some("sh") {
            found.push(path);
        }
    }
}

#[test]
fn no_shell_files_remain_in_repo() {
    let root = repo_root();
    let mut found = Vec::new();
    collect_shell_files(&root, &mut found);
    assert!(
        found.is_empty(),
        "Found {} unexpected `.sh` file(s) in the repo (excluding target/, .git/, node_modules/):\n{}",
        found.len(),
        found
            .iter()
            .map(|p| format!("  {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
