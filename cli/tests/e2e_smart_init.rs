//! E2E — `/compass:init` Mode A/B/C mechanics. The init workflow itself is
//! LLM-interpreted (AskUserQuestion driven); this test exercises the exact
//! CLI surfaces that workflow invokes for each mode branch. Ported from
//! `tests/integration/e2e_smart_init.sh`.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{run_cli_ok, HomeGuard};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("invalid JSON: {e}\n---\n{out}"))
}

// ---------------------------------------------------------------------------
// Mode A — first-time global wizard writes ~/.compass/global-config.json
// ---------------------------------------------------------------------------

#[test]
fn mode_a_global_wizard_creates_global_config() {
    let g = HomeGuard::new();
    let home = g.home_path().to_path_buf();
    let global_path = home.join(".compass").join("global-config.json");

    // Pre-condition: no global config yet in a fresh $HOME.
    assert!(
        !global_path.exists(),
        "precondition: global-config should not exist yet at {}",
        global_path.display()
    );

    // The 4 CLI calls the workflow makes after the AskUserQuestion block.
    run_cli_ok(&["project", "global-config", "set", "--key", "lang", "--value", "vi"]);
    run_cli_ok(&[
        "project", "global-config", "set", "--key", "default_tech_stack",
        "--value", "[\"typescript\",\"python\"]",
    ]);
    run_cli_ok(&[
        "project", "global-config", "set", "--key", "default_review_style",
        "--value", "whole_document",
    ]);
    run_cli_ok(&["project", "global-config", "set", "--key", "default_domain", "--value", "ard"]);

    assert!(
        global_path.exists(),
        "global-config should exist at {}",
        global_path.display()
    );

    let v = json(&fs::read_to_string(&global_path).expect("read global-config"));
    assert_eq!(v["lang"], "vi", "lang mismatch");
    assert_eq!(
        v["default_tech_stack"].as_array().expect("array").len(),
        2,
        "tech_stack should have 2 entries"
    );
    assert_eq!(v["default_review_style"], "whole_document");
    assert_eq!(v["default_domain"], "ard");
    assert!(v["version"].is_string(), "version string expected");
    assert!(v["created_at"].is_string(), "created_at expected");
    assert!(v["updated_at"].is_string(), "updated_at expected");

    // Round-trip via `get` to confirm the persisted file is readable.
    let out = run_cli_ok(&["project", "global-config", "get", "--key", "lang"]);
    assert!(out.trim().contains("vi"), "get round-trip should return 'vi'; got {out:?}");
}

// ---------------------------------------------------------------------------
// Mode B — cwd has no config → workflow creates project skeleton + registers
// ---------------------------------------------------------------------------

fn seed_beta_project(root: &Path) {
    // Folder skeleton the Mode B step creates via `mkdir -p`.
    for d in [
        "prd",
        "epics",
        "wiki",
        "prototype",
        "technical",
        "release-notes",
        "research",
        ".compass/.state/sessions",
    ] {
        fs::create_dir_all(root.join(d)).expect("mkdir");
    }

    // Mode B writes config.json merging global defaults with project-specific
    // answers. Simulate that merged output.
    let cfg = serde_json::json!({
        "version": "1.2.0",
        "project": {"name": "Beta", "po": "@me"},
        "lang": "vi", "spec_lang": "vi", "mode": "standalone",
        "prefix": "BT", "domain": "ard",
        "output_paths": {"prd": "prd", "story": "epics/{EPIC}/user-stories", "epic": "epics"},
        "naming": {"prd": "{slug}.md"}
    });
    fs::write(
        root.join(".compass").join(".state").join("config.json"),
        serde_json::to_vec_pretty(&cfg).expect("serialize config"),
    )
    .expect("write config.json");

    // Mode B also writes a CLAUDE.md header.
    fs::write(
        root.join("CLAUDE.md"),
        "# Claude Context — Beta\n- product: Beta\n- domain: ard\n- po: @me\n",
    )
    .expect("write CLAUDE.md");
}

#[test]
fn mode_b_project_create_full_structure() {
    let _g = HomeGuard::new();

    let tmp = TempDir::new().expect("tempdir");
    let dir_b = tmp.path().join("project-beta");
    fs::create_dir_all(&dir_b).expect("mkdir dir_b");

    // Mode B pre-condition: cwd has no config yet.
    let cfg_path = dir_b.join(".compass").join(".state").join("config.json");
    assert!(!cfg_path.exists(), "precondition: no config at {}", cfg_path.display());

    seed_beta_project(&dir_b);

    // Register + activate (the final step of the Mode B flow).
    run_cli_ok(&["project", "add", dir_b.to_str().unwrap()]);
    run_cli_ok(&["project", "use", dir_b.to_str().unwrap()]);

    // Every directory the workflow promises must exist.
    for d in ["prd", "epics", "wiki", "prototype", "technical", "release-notes", "research", ".compass/.state"] {
        assert!(dir_b.join(d).is_dir(), "expected directory {d} at {}", dir_b.display());
    }

    // Resolve should pick Beta.
    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok");
    assert_eq!(r["name"], "Beta");

    // Registry should contain the Beta path.
    let home = std::env::var("HOME").expect("HOME");
    let registry = Path::new(&home).join(".compass").join("projects.json");
    assert!(registry.exists(), "registry should exist after project add");
    let reg = json(&fs::read_to_string(&registry).expect("read registry"));
    let projects = reg["projects"].as_array().expect("projects array");
    let dir_b_canonical = fs::canonicalize(&dir_b).unwrap_or(dir_b.clone());
    let found = projects.iter().any(|p| {
        let path = p["path"].as_str().unwrap_or("");
        Path::new(path) == dir_b_canonical
    });
    assert!(found, "registry should contain Beta path; got {}", reg);
}

// ---------------------------------------------------------------------------
// Mode C — cwd has config → targeted field update preserves the rest
// ---------------------------------------------------------------------------

#[test]
fn mode_c_config_update_preserves_other_fields() {
    let _g = HomeGuard::new();

    let tmp = TempDir::new().expect("tempdir");
    let dir_b = tmp.path().join("project-beta-c");
    seed_beta_project(&dir_b);
    run_cli_ok(&["project", "add", dir_b.to_str().unwrap()]);
    run_cli_ok(&["project", "use", dir_b.to_str().unwrap()]);

    let cfg_path = dir_b.join(".compass").join(".state").join("config.json");
    let original = json(&fs::read_to_string(&cfg_path).expect("read config"));

    // Simulate the Mode C "change prefix to BZ" edit the workflow performs
    // via Read/Write (jq-equivalent in Rust).
    let mut updated = original.clone();
    updated["prefix"] = serde_json::json!("BZ");
    fs::write(&cfg_path, serde_json::to_vec_pretty(&updated).expect("serialize"))
        .expect("rewrite config");

    let after = json(&fs::read_to_string(&cfg_path).expect("re-read config"));
    assert_eq!(after["prefix"], "BZ", "prefix should be updated to BZ");
    // Every untouched field must remain as it was.
    assert_eq!(after["project"]["name"], "Beta", "project.name preserved");
    assert_eq!(after["domain"], "ard", "domain preserved");
    assert_eq!(after["lang"], "vi", "lang preserved");
    assert_eq!(after["mode"], "standalone", "mode preserved");
    assert_eq!(after["version"], "1.2.0", "version preserved");

    // Resolve should still find the project and reflect the new prefix.
    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok");
    assert_eq!(r["config"]["prefix"], "BZ");
}
