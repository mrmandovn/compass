//! Regression guard — every slash-command wrapper at
//! `core/commands/compass/*.md` must carry the anti-menu directive so the
//! orchestrator LLM never paraphrases CLI commands from a workflow body
//! back to the user as a menu.
//!
//! Fails CI if a wrapper is missing the canonical phrase, or if the wrapper
//! count drifts from the expected 21.

use std::fs;
use std::path::PathBuf;

const REQUIRED_DIRECTIVE: &str =
    "never synthesize menus from CLI command listings or bash blocks";
const EXPECTED_WRAPPER_COUNT: usize = 21;

fn wrappers_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has a parent (the repo root)")
        .join("core/commands/compass")
}

fn md_wrappers() -> Vec<PathBuf> {
    fs::read_dir(wrappers_dir())
        .expect("wrappers dir exists")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
        .collect()
}

#[test]
fn all_wrappers_have_anti_menu_directive() {
    let mut missing: Vec<String> = Vec::new();
    for path in md_wrappers() {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        if !content.contains(REQUIRED_DIRECTIVE) {
            missing.push(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<?>")
                    .to_string(),
            );
        }
    }
    assert!(
        missing.is_empty(),
        "Wrappers missing anti-menu directive (expected phrase: {:?}): {:?}",
        REQUIRED_DIRECTIVE,
        missing
    );
}

#[test]
fn wrapper_count_matches_expected() {
    let count = md_wrappers().len();
    assert_eq!(
        count, EXPECTED_WRAPPER_COUNT,
        "Expected {EXPECTED_WRAPPER_COUNT} command wrappers in {}, found {count}. \
         If you intentionally added or removed one, update EXPECTED_WRAPPER_COUNT here.",
        wrappers_dir().display()
    );
}
