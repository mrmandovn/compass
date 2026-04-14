//! PRD taste validator: R-FLOW and R-XREF.
//!
//! Two rules only, per SCHEMAS-v1.md §4:
//!   * R-FLOW  — every non-blank, non-heading line inside a `## User Flows`
//!               section must match `^\s*\d+\.\s`.
//!   * R-XREF  — every `[LINK-…]`, `[EPIC-…]`, `[REQ-…]` token must resolve
//!               either to a heading anchor in the same file or to a file
//!               under PRDs/, Stories/, Backlog/, or epics/ (relative to an
//!               inferred project root). `[LINK-EXT: https://…]` is skipped.

use crate::helpers;
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn validate_prd(path: &Path) -> Result<String, String> {
    let content = helpers::read_file(path)?;
    let project_root = find_project_root(path);

    let mut violations: Vec<Value> = Vec::new();
    check_flow(&content, &mut violations);
    check_xref(&content, path, project_root.as_deref(), &mut violations);

    Ok(serde_json::to_string_pretty(&json!({
        "ok": violations.is_empty(),
        "violations": violations,
    })).unwrap())
}

/// Walk up from the PRD file until we find a directory that contains one
/// of the known output folders (PRDs/, Stories/, Backlog/, epics/). That
/// directory is treated as the project root for R-XREF file lookups.
fn find_project_root(prd_path: &Path) -> Option<PathBuf> {
    let start = prd_path
        .canonicalize()
        .ok()
        .or_else(|| Some(prd_path.to_path_buf()))?;
    let mut cursor: Option<&Path> = start.parent();
    while let Some(dir) = cursor {
        for marker in ["PRDs", "Stories", "Backlog", "epics"] {
            if dir.join(marker).is_dir() {
                return Some(dir.to_path_buf());
            }
        }
        cursor = dir.parent();
    }
    None
}

// ---- R-FLOW -----------------------------------------------------------------

fn check_flow(content: &str, out: &mut Vec<Value>) {
    // Ordered list line (allows leading indent for nested steps).
    let ordered = Regex::new(r"^\s*\d+\.\s").unwrap();
    // Heading lines (any level) — we use them to know when sections end.
    let heading = Regex::new(r"^\s{0,3}#{1,6}\s+(.*)$").unwrap();

    let mut in_flows = false;
    let mut flows_level: usize = 0;
    let mut current_section = String::from("User Flows");

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        // A fenced code block inside User Flows would be unusual; we keep
        // the rule simple and treat code fences as normal content lines.

        if let Some(caps) = heading.captures(raw_line) {
            let hashes = raw_line.trim_start().chars().take_while(|c| *c == '#').count();
            let title = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");

            if !in_flows {
                // Enter the section only on a literal `## User Flows` heading
                // (case-insensitive on the title).
                if hashes == 2 && title.eq_ignore_ascii_case("User Flows") {
                    in_flows = true;
                    flows_level = hashes;
                    current_section = title.to_string();
                }
            } else {
                // Inside the Flows section. A sub-heading updates the label
                // (for nicer error messages); a heading at the same-or-higher
                // level as the opening `## User Flows` closes the section.
                if hashes <= flows_level {
                    in_flows = false;
                    // Re-enter if this heading itself is another User Flows.
                    if hashes == 2 && title.eq_ignore_ascii_case("User Flows") {
                        in_flows = true;
                        flows_level = hashes;
                        current_section = title.to_string();
                    }
                } else {
                    current_section = format!("User Flows > {}", title);
                }
            }
            continue;
        }

        if !in_flows {
            continue;
        }
        if raw_line.trim().is_empty() {
            continue;
        }
        if ordered.is_match(raw_line) {
            continue;
        }

        out.push(json!({
            "rule": "R-FLOW",
            "section": current_section,
            "line": line_no,
            "message": format!(
                "Section '{}' line {}: expected ordered numeric list item (e.g. `1. step`), found: {:?}",
                current_section,
                line_no,
                raw_line.trim_end()
            ),
        }));
    }
}

// ---- R-XREF -----------------------------------------------------------------

fn check_xref(
    content: &str,
    prd_path: &Path,
    project_root: Option<&Path>,
    out: &mut Vec<Value>,
) {
    // Matches [LINK-foo], [EPIC-bar], [REQ-baz]. LINK-EXT is filtered out
    // before lookup.
    let token = Regex::new(r"\[(LINK-[A-Z0-9_\-]+|EPIC-[A-Z0-9_\-]+|REQ-[A-Z0-9_\-]+)\]").unwrap();
    let heading = Regex::new(r"^\s{0,3}#{1,6}\s+(.*)$").unwrap();

    // Collect heading anchors in the same file. Slug = lowercase, non-word
    // runs → `-`. We also treat the raw token text as a direct anchor so an
    // author can simply write a heading called `LINK-foo` and reference
    // `[LINK-foo]`.
    let mut anchors: Vec<String> = Vec::new();
    for line in content.lines() {
        if let Some(caps) = heading.captures(line) {
            let raw = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            anchors.push(raw.to_lowercase());
            anchors.push(slugify(raw));
        }
    }

    let search_dirs = ["PRDs", "Stories", "Backlog", "epics"];
    let prd_dir = prd_path.parent().map(|p| p.to_path_buf());

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        for caps in token.captures_iter(line) {
            let raw_ref = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let full = caps.get(0).map(|m| m.as_str()).unwrap_or("");

            // Skip explicit external links, which use the literal prefix
            // `LINK-EXT:` inside the brackets (e.g. `[LINK-EXT: https://…]`).
            // Our token regex already rejects `:`, but we also look at the
            // surrounding bracket contents on the line to catch that form.
            if raw_ref.starts_with("LINK-EXT") {
                continue;
            }
            if line.contains("[LINK-EXT:") {
                // Only skip the LINK-EXT occurrence on this line, but our
                // token regex won't match `[LINK-EXT: …]` anyway since `:`
                // isn't in the char class, so any non-EXT token on the same
                // line still needs to resolve. Fall through.
            }

            if resolve_xref(raw_ref, &anchors, project_root, prd_dir.as_deref(), &search_dirs) {
                continue;
            }

            out.push(json!({
                "rule": "R-XREF",
                "line": line_no,
                "message": format!(
                    "Dangling reference {}: no matching anchor in file; not found under PRDs/, Stories/, Backlog/, epics/.",
                    full
                ),
            }));
        }
    }
}

fn resolve_xref(
    raw_ref: &str,
    anchors: &[String],
    project_root: Option<&Path>,
    prd_dir: Option<&Path>,
    search_dirs: &[&str; 4],
) -> bool {
    let lower = raw_ref.to_lowercase();
    let slug = slugify(raw_ref);
    // Accept exact match, or a heading whose slug begins with the token
    // followed by `-` — e.g. `## REQ-42 Something` resolves `[REQ-42]`.
    let prefix = format!("{}-", slug);
    if anchors
        .iter()
        .any(|a| a == &lower || a == &slug || a.starts_with(&prefix))
    {
        return true;
    }

    // File lookup: try `<root>/<dir>/**` with extensions .md / .markdown
    // and also as a directory name. We do a shallow walk — the spec only
    // requires that *some* file under those dirs has a matching stem.
    let roots: Vec<PathBuf> = match project_root {
        Some(r) => vec![r.to_path_buf()],
        None => prd_dir.map(|d| vec![d.to_path_buf()]).unwrap_or_default(),
    };

    for root in &roots {
        for dir in search_dirs {
            let base = root.join(dir);
            if !base.is_dir() {
                continue;
            }
            if file_with_stem_exists(&base, raw_ref) {
                return true;
            }
        }
    }
    false
}

/// Recursively check whether any file under `base` has a name containing
/// `needle` (case-insensitive), with a `.md` / `.markdown` extension, OR a
/// directory whose name contains `needle`. Bounded depth to keep this O(n)
/// on typical project layouts.
fn file_with_stem_exists(base: &Path, needle: &str) -> bool {
    let needle_l = needle.to_lowercase();
    walk_contains(base, &needle_l, 0, 6)
}

fn walk_contains(dir: &Path, needle_l: &str, depth: usize, max_depth: usize) -> bool {
    if depth > max_depth {
        return false;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_lowercase(),
            None => continue,
        };
        if path.is_dir() {
            if file_name.contains(needle_l) {
                return true;
            }
            if walk_contains(&path, needle_l, depth + 1, max_depth) {
                return true;
            }
        } else {
            let is_md = file_name.ends_with(".md") || file_name.ends_with(".markdown");
            if is_md {
                let stem = file_name
                    .trim_end_matches(".markdown")
                    .trim_end_matches(".md");
                if stem.contains(needle_l) {
                    return true;
                }
            }
        }
    }
    false
}

fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_accepts_ordered_list() {
        let md = "\
# Title
## User Flows
1. Open the app
2. Tap Sign up
3. Enter email
";
        let mut v = vec![];
        check_flow(md, &mut v);
        assert!(v.is_empty(), "expected no violations, got {:?}", v);
    }

    #[test]
    fn flow_rejects_bullets() {
        let md = "\
## User Flows
- open app
- tap sign up
";
        let mut v = vec![];
        check_flow(md, &mut v);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0]["rule"], "R-FLOW");
    }

    #[test]
    fn flow_rejects_prose() {
        let md = "\
## User Flows
The user opens the app and signs up.
";
        let mut v = vec![];
        check_flow(md, &mut v);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn flow_allows_nested_ordered_items() {
        let md = "\
## User Flows
1. step one
    1. sub step
2. step two
";
        let mut v = vec![];
        check_flow(md, &mut v);
        assert!(v.is_empty(), "got {:?}", v);
    }

    #[test]
    fn flow_closes_on_next_h2() {
        let md = "\
## User Flows
1. one
## Other
- bullet here is fine
";
        let mut v = vec![];
        check_flow(md, &mut v);
        assert!(v.is_empty(), "got {:?}", v);
    }

    #[test]
    fn xref_skips_link_ext() {
        // LINK-EXT uses `:` so the token regex already won't match it.
        let md = "See [LINK-EXT: https://example.com] for details.\n";
        let mut v = vec![];
        check_xref(md, Path::new("/tmp/does-not-exist.md"), None, &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn xref_reports_dangling_when_no_root() {
        let md = "See [REQ-42] please.\n";
        let mut v = vec![];
        check_xref(md, Path::new("/tmp/does-not-exist.md"), None, &mut v);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0]["rule"], "R-XREF");
    }

    #[test]
    fn xref_resolves_to_same_file_heading() {
        let md = "\
# Title
## REQ-42 Something
See [REQ-42] please.
";
        let mut v = vec![];
        check_xref(md, Path::new("/tmp/does-not-exist.md"), None, &mut v);
        assert!(v.is_empty(), "got {:?}", v);
    }

    #[test]
    fn slugify_handles_spaces_and_punct() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("REQ-42 Something"), "req-42-something");
    }
}

#[cfg(test)]
mod prd_tests {
    //! Fixture-backed end-to-end tests for `validate_prd`. Each test loads
    //! a real markdown fixture under `cli/tests/fixtures/`, runs the public
    //! `validate_prd` entry point, and asserts on the JSON result.
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn run_prd(name: &str) -> serde_json::Value {
        let path = fixture(name);
        let out = validate_prd(&path).expect("validate_prd returned Err");
        serde_json::from_str(&out).expect("validate_prd output was not JSON")
    }

    #[test]
    fn flow_pass() {
        let result = run_prd("prd_good_flow.md");
        let violations = result["violations"].as_array().expect("violations must be array");
        let flow_hits: Vec<_> = violations.iter()
            .filter(|v| v["rule"] == "R-FLOW")
            .collect();
        assert!(flow_hits.is_empty(),
            "expected no R-FLOW violations, got: {:?}", flow_hits);
    }

    #[test]
    fn flow_fail_prose() {
        let result = run_prd("prd_bad_flow_prose.md");
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "R-FLOW")
            .expect("expected R-FLOW violation");
        let section = hit["section"].as_str().unwrap_or("");
        assert!(section.contains("User Flows"),
            "expected section to reference 'User Flows', got: {:?}", section);
    }

    #[test]
    fn flow_fail_bullet() {
        let result = run_prd("prd_bad_flow_bullet.md");
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "R-FLOW")
            .expect("expected R-FLOW violation");
        let section = hit["section"].as_str().unwrap_or("");
        assert!(section.contains("User Flows"),
            "expected section to reference 'User Flows', got: {:?}", section);
    }

    #[test]
    fn xref_pass() {
        let result = run_prd("prd_xref_valid.md");
        let violations = result["violations"].as_array().expect("violations must be array");
        let xref_hits: Vec<_> = violations.iter()
            .filter(|v| v["rule"] == "R-XREF")
            .collect();
        assert!(xref_hits.is_empty(),
            "expected no R-XREF violations, got: {:?}", xref_hits);
    }

    #[test]
    fn xref_dangling() {
        let result = run_prd("prd_xref_dangling.md");
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "R-XREF")
            .expect("expected R-XREF violation");
        let msg = hit["message"].as_str().unwrap_or("");
        // The error message embeds the full token, e.g. "[EPIC-999]" or "[REQ-404]".
        assert!(msg.contains("[EPIC-") || msg.contains("[REQ-"),
            "expected message to reference an EPIC-/REQ- target, got: {:?}", msg);
    }
}
