//! Regression guard — every slash-command wrapper at
//! `core/commands/compass/*.md` must use the plain-markdown template
//! introduced in v1.0.2: `## Workflow` + `## Instructions` headings,
//! no Claude-specific XML tags (`<output>`, `<objective>`,
//! `<execution_context>`, `<process>`) that render poorly on other hosts
//! like OpenCode.
//!
//! Also guards the anti-menu-synthesis directive introduced in v1.0.1.
//!
//! Fails CI if a wrapper is missing the canonical structure, contains
//! forbidden XML tags, or the wrapper count drifts from 21.

use std::fs;
use std::path::PathBuf;

const FORBIDDEN_TAGS: &[&str] = &[
    "<output>",
    "</output>",
    "<objective>",
    "</objective>",
    "<execution_context>",
    "</execution_context>",
    "<process>",
    "</process>",
];

const REQUIRED_HEADINGS: &[&str] = &["## Workflow", "## Instructions"];

const REQUIRED_ANTI_MENU_PHRASE: &str =
    "Never synthesize menus from bash/CLI command listings";

const EXPECTED_WRAPPER_COUNT: usize = 24;

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

fn file_name(p: &PathBuf) -> String {
    p.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("<?>")
        .to_string()
}

#[test]
fn all_wrappers_have_plain_markdown_structure() {
    let mut missing_heading: Vec<(String, &str)> = Vec::new();
    for path in md_wrappers() {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        for h in REQUIRED_HEADINGS {
            if !content.contains(h) {
                missing_heading.push((file_name(&path), h));
            }
        }
    }
    assert!(
        missing_heading.is_empty(),
        "Wrappers missing required heading(s): {:?}",
        missing_heading
    );
}

#[test]
fn no_wrapper_has_xml_tags() {
    let mut offenders: Vec<(String, &str)> = Vec::new();
    for path in md_wrappers() {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        for tag in FORBIDDEN_TAGS {
            if content.contains(tag) {
                offenders.push((file_name(&path), tag));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "Wrappers contain forbidden XML tags (plain markdown required for multi-host compat): {:?}",
        offenders
    );
}

#[test]
fn all_wrappers_have_anti_menu_directive() {
    let mut missing: Vec<String> = Vec::new();
    for path in md_wrappers() {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
        if !content.contains(REQUIRED_ANTI_MENU_PHRASE) {
            missing.push(file_name(&path));
        }
    }
    assert!(
        missing.is_empty(),
        "Wrappers missing anti-menu directive (expected phrase: {:?}): {:?}",
        REQUIRED_ANTI_MENU_PHRASE,
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
