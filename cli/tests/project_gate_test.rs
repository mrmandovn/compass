//! E2E tests for `compass-cli project gate` and the extended
//! `compass-cli project resolve` (shared_root). Covers the T-05 test spec for
//! the agent-dispatch-gate-port session.
//!
//! Approach: every test is **black-box** — it shells out to the compiled
//! `compass-cli` binary via `run_cli`. The Jaccard scorer is exercised
//! indirectly by seeding a fixture pipeline with a controlled `context.json`
//! title and asserting the `relevance` field that the binary emits. This
//! keeps the tests strictly at the CLI contract and lets us cover the
//! stopword list, case-insensitivity, and empty-after-filter edge cases
//! without touching private module internals.
//!
//! All fixtures are built inside `tempfile::TempDir`s so tests do not pollute
//! the checked-in `cli/tests/fixtures/` tree; `HomeGuard` isolates
//! `$HOME` per test so the user's real `~/.compass/projects.json` registry is
//! never touched.
//!
//! Covers:
//! - REQ-03 (resolve.shared_root) — 2 tests
//! - REQ-05 (Jaccard scorer)      — 5 tests
//! - REQ-06 (case selection + stale detection) — 7 tests
//!
//! Total: 14 tests.

#[path = "e2e_common.rs"]
mod e2e_common;

use e2e_common::{run_cli, run_cli_ok, HomeGuard};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("invalid JSON: {e}\n---\n{out}"))
}

/// Build a minimal Compass project rooted at `root` — matches what
/// `compass-cli project add` expects (must have `.compass/.state/config.json`).
fn make_project(root: &Path, name: &str) {
    let state = root.join(".compass").join(".state");
    fs::create_dir_all(&state).expect("mkdir .compass/.state");
    let cfg = serde_json::json!({
        "version": "1.2.0",
        "project": {"name": name, "po": "@t"},
        "lang": "en",
        "spec_lang": "en",
        "mode": "standalone",
        "prefix": "PX",
        "domain": "ard",
        "output_paths": {
            "prd": "prd",
            "story": "epics/{EPIC}/user-stories",
            "epic": "epics",
        },
        "naming": {"prd": "{slug}.md"},
    });
    fs::write(
        state.join("config.json"),
        serde_json::to_vec_pretty(&cfg).expect("serialize config"),
    )
    .expect("write config.json");
}

/// Format epoch seconds as `YYYY-MM-DDTHH:MM:SSZ` — matches the format
/// `compass-cli`'s `now_iso` / `parse_iso_utc_to_epoch` speak (Howard Hinnant
/// civil_from_days algorithm, dependency-free).
fn format_iso_utc(total_secs: u64) -> String {
    let days = (total_secs / 86_400) as i64;
    let tod = total_secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    let hh = tod / 3600;
    let mm = (tod % 3600) / 60;
    let ss = tod % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hh, mm, ss
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

/// Return an ISO timestamp for `days_ago` days in the past, computed from the
/// current clock. Used to build pipeline `created_at` values that straddle
/// the 14-day stale threshold deterministically.
fn iso_days_ago(days_ago: u64) -> String {
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let past = now_secs.saturating_sub(days_ago * 86_400);
    format_iso_utc(past)
}

/// Write a `sessions/<slug>/pipeline.json` + `context.json` pair inside
/// `project_root`. Used to seed `project gate`'s scan input.
fn make_pipeline(
    project_root: &Path,
    slug: &str,
    title: &str,
    created_at_iso: &str,
    artifacts_count: usize,
    active: bool,
) {
    let session_dir = project_root
        .join(".compass")
        .join(".state")
        .join("sessions")
        .join(slug);
    fs::create_dir_all(&session_dir).expect("mkdir session dir");

    let artifacts: Vec<Value> = (0..artifacts_count)
        .map(|i| serde_json::json!({"id": format!("art-{}", i), "type": "prd"}))
        .collect();

    let pipeline = serde_json::json!({
        "status": if active { "active" } else { "closed" },
        "created_at": created_at_iso,
        "artifacts": artifacts,
    });
    fs::write(
        session_dir.join("pipeline.json"),
        serde_json::to_vec_pretty(&pipeline).expect("serialize pipeline"),
    )
    .expect("write pipeline.json");

    let context = serde_json::json!({"title": title});
    fs::write(
        session_dir.join("context.json"),
        serde_json::to_vec_pretty(&context).expect("serialize context"),
    )
    .expect("write context.json");
}

/// Register + activate `project_root` via the CLI — the same path a real
/// user walks through after `compass-cli project add` + `project use`.
fn register_and_use(project_root: &Path) {
    let p = project_root.to_str().expect("project_root utf8");
    run_cli_ok(&["project", "add", p]);
    run_cli_ok(&["project", "use", p]);
}

/// Build a standalone project (no sibling `shared/`) and register it. Returns
/// the TempDir owning the filesystem so callers can keep it alive for the
/// duration of the test.
fn standalone_project(name: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let parent = tmp.path().to_path_buf();
    let root = parent.join("proj");
    make_project(&root, name);
    register_and_use(&root);
    (tmp, root)
}

/// Convenience: run `project gate` and return the parsed JSON output.
fn gate(args_text: &str) -> Value {
    let out = run_cli_ok(&[
        "project",
        "gate",
        "--args",
        args_text,
        "--artifact-type",
        "prd",
    ]);
    json(&out)
}

/// Pull the `relevance` value out of `active_pipelines[0]`. Panics if absent —
/// we only ever call this on fixtures that seeded a pipeline.
fn top_relevance(gate_out: &Value) -> f64 {
    gate_out["active_pipelines"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|p| p["relevance"].as_f64())
        .unwrap_or_else(|| panic!("no active_pipelines[0].relevance in {gate_out}"))
}

/// Build a project with a single active pipeline whose `context.json.title` is
/// the supplied title. Caller then runs `gate(args)` and inspects
/// `active_pipelines[0].relevance` to read the Jaccard score the CLI computed.
fn jaccard_fixture(tag: &str, title: &str) -> (TempDir, PathBuf) {
    let (tmp, root) = standalone_project(tag);
    // created_at = "now" keeps stale=false regardless of artifacts_count; we
    // don't care about the options_spec shape for pure scoring tests.
    make_pipeline(
        &root,
        "some-slug",
        title,
        &iso_days_ago(0),
        /*artifacts*/ 1,
        /*active*/ true,
    );
    (tmp, root)
}

/// Assert |a - b| <= eps with a helpful panic message when not.
fn assert_close(a: f64, b: f64, eps: f64, label: &str) {
    assert!(
        (a - b).abs() <= eps,
        "{label}: expected ≈{b}, got {a} (|Δ|={})",
        (a - b).abs()
    );
}

// ---------------------------------------------------------------------------
// REQ-05: Jaccard scoring
// ---------------------------------------------------------------------------

/// "auth refactor" vs "auth flow refactor": tokens a={auth,refactor},
/// b={auth,flow,refactor}. |∩|=2, |∪|=3 → 2/3 ≈ 0.667.
#[test]
fn jaccard_basic_overlap() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = jaccard_fixture("jaccard_basic", "auth flow refactor");
    let out = gate("auth refactor");
    assert_close(top_relevance(&out), 2.0 / 3.0, 0.01, "jaccard_basic_overlap");
}

/// "add new auth for app" vs "auth flow" — after stopword filter
/// ({a,an,the,for,and,or,of,in,on,to,app,new,old}):
///   a = {add, auth}        (drops: new, for, app)
///   b = {auth, flow}
/// |∩|={auth}=1, |∪|={add,auth,flow}=3 → 1/3 ≈ 0.333.
#[test]
fn jaccard_stopwords_removed() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = jaccard_fixture("jaccard_stop", "auth flow");
    let out = gate("add new auth for app");
    assert_close(
        top_relevance(&out),
        1.0 / 3.0,
        0.01,
        "jaccard_stopwords_removed",
    );
}

/// Completely disjoint token sets → 0.0.
#[test]
fn jaccard_no_overlap() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = jaccard_fixture("jaccard_none", "billing invoice");
    let out = gate("auth login");
    assert_close(top_relevance(&out), 0.0, 1e-6, "jaccard_no_overlap");
}

/// Both sides empty after stopword filter → spec returns 0.0 (no signal).
///   a = "the new app" → {} (all stopwords)
///   b = "for the old app" → {} (all stopwords)
#[test]
fn jaccard_empty_after_stopwords() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = jaccard_fixture("jaccard_empty", "for the old app");
    let out = gate("the new app");
    assert_close(
        top_relevance(&out),
        0.0,
        1e-6,
        "jaccard_empty_after_stopwords",
    );
}

/// Case must be folded before comparing; identical sets → 1.0.
#[test]
fn jaccard_case_insensitive() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = jaccard_fixture("jaccard_case", "auth refactor");
    let out = gate("Auth REFACTOR");
    assert_close(top_relevance(&out), 1.0, 1e-6, "jaccard_case_insensitive");
}

// ---------------------------------------------------------------------------
// REQ-06: Case selection
// ---------------------------------------------------------------------------

/// 1 active pipeline with relevance ≥ 0.2 (here 1.0) → case 1,
/// suggested_action = "continue:<slug>".
#[test]
fn case_1_one_relevant_pipeline() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("case1");
    make_pipeline(&root, "auth-refactor", "auth refactor", &iso_days_ago(0), 1, true);

    let out = gate("auth refactor");
    assert_eq!(out["case"], 1, "expected case=1, got {out}");
    assert_eq!(
        out["suggested_action"], "continue:auth-refactor",
        "expected suggested_action='continue:auth-refactor', got {out}"
    );
    // Sanity: the pipeline surfaced and relevance ≥ 0.2.
    let rel = top_relevance(&out);
    assert!(rel >= 0.2, "expected relevance ≥ 0.2, got {rel}");
}

/// 1 active pipeline with relevance < 0.2 (here 0.0, disjoint tokens) →
/// case 2, suggested_action = "standalone".
#[test]
fn case_2_one_unrelated_pipeline() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("case2");
    make_pipeline(&root, "billing-rework", "billing invoice", &iso_days_ago(0), 1, true);

    let out = gate("auth login");
    assert_eq!(out["case"], 2, "expected case=2, got {out}");
    assert_eq!(
        out["suggested_action"], "standalone",
        "expected suggested_action='standalone', got {out}"
    );
    assert_close(top_relevance(&out), 0.0, 1e-6, "case_2 relevance");
}

/// No active pipelines → case 3, suggested_action = "current_project".
/// Sessions dir is absent entirely, which `gate` must handle via a tolerant
/// `read_dir`.
#[test]
fn case_3_no_active_pipelines() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = standalone_project("case3");
    // No pipelines seeded.

    let out = gate("anything at all");
    assert_eq!(out["case"], 3, "expected case=3, got {out}");
    assert_eq!(
        out["suggested_action"], "current_project",
        "expected suggested_action='current_project', got {out}"
    );
    assert_eq!(
        out["active_pipelines"].as_array().map(|a| a.len()),
        Some(0),
        "expected empty active_pipelines, got {out}"
    );
}

/// 2+ active pipelines → case 4, suggested_action = "continue:<top_slug>".
/// Top pipeline is the one with the highest Jaccard relevance. Here
/// `auth-refactor` (title "auth refactor") dominates `billing-flow`
/// (title "billing invoice") for args "auth refactor".
#[test]
fn case_4_multiple_pipelines() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("case4");
    make_pipeline(&root, "auth-refactor", "auth refactor", &iso_days_ago(1), 1, true);
    make_pipeline(&root, "billing-flow", "billing invoice", &iso_days_ago(2), 1, true);
    // Closed pipelines must not count.
    make_pipeline(&root, "old-closed", "some other work", &iso_days_ago(30), 0, false);

    let out = gate("auth refactor");
    assert_eq!(out["case"], 4, "expected case=4, got {out}");
    assert_eq!(
        out["suggested_action"], "continue:auth-refactor",
        "expected suggested_action='continue:auth-refactor', got {out}"
    );

    let pipelines = out["active_pipelines"].as_array().expect("array");
    assert_eq!(
        pipelines.len(),
        2,
        "expected 2 active pipelines (closed filtered out), got {}",
        pipelines.len()
    );
    // Top pipeline has highest relevance.
    assert_eq!(pipelines[0]["slug"], "auth-refactor");
    let r0 = pipelines[0]["relevance"].as_f64().unwrap_or(-1.0);
    let r1 = pipelines[1]["relevance"].as_f64().unwrap_or(-1.0);
    assert!(
        r0 >= r1,
        "expected active_pipelines sorted desc by relevance; got r0={r0} r1={r1}"
    );
}

// ---------------------------------------------------------------------------
// REQ-06: Stale detection (age_days > 14 AND artifacts_count == 0)
// ---------------------------------------------------------------------------

#[test]
fn stale_true_when_old_and_empty() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("stale_true");
    make_pipeline(
        &root,
        "old-empty",
        "legacy cleanup",
        &iso_days_ago(20),
        /*artifacts*/ 0,
        /*active*/ true,
    );

    let out = gate("anything");
    let p = &out["active_pipelines"][0];
    assert_eq!(
        p["stale"], true,
        "expected stale=true for 20d-old empty pipeline, got {p}"
    );
    let age = p["age_days"].as_u64().unwrap_or(0);
    assert!(age >= 14, "expected age_days ≥ 14, got {age}");
    assert_eq!(p["artifacts_count"], 0);
}

#[test]
fn stale_false_when_has_artifacts() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("stale_artifacts");
    make_pipeline(
        &root,
        "old-but-productive",
        "ongoing epic",
        &iso_days_ago(20),
        /*artifacts*/ 3,
        /*active*/ true,
    );

    let out = gate("anything");
    let p = &out["active_pipelines"][0];
    assert_eq!(
        p["stale"], false,
        "expected stale=false when artifacts_count>0, got {p}"
    );
    assert_eq!(p["artifacts_count"], 3);
}

#[test]
fn stale_false_when_recent() {
    let _g = HomeGuard::new();
    let (_tmp, root) = standalone_project("stale_recent");
    make_pipeline(
        &root,
        "fresh",
        "new work",
        &iso_days_ago(5),
        /*artifacts*/ 0,
        /*active*/ true,
    );

    let out = gate("anything");
    let p = &out["active_pipelines"][0];
    assert_eq!(
        p["stale"], false,
        "expected stale=false for 5-day-old pipeline, got {p}"
    );
    let age = p["age_days"].as_u64().unwrap_or(99);
    assert!(age <= 14, "expected age_days ≤ 14, got {age}");
}

// ---------------------------------------------------------------------------
// REQ-03: resolve.shared_root
// ---------------------------------------------------------------------------

/// Parent dir contains both `proj/` and `shared/` → resolve emits
/// `shared_root` as the absolute canonicalized path to `shared/`.
#[test]
fn resolve_shared_root_set_when_sibling_exists() {
    let _g = HomeGuard::new();
    let tmp = TempDir::new().expect("tempdir");
    let parent = tmp.path().to_path_buf();
    let proj = parent.join("proj");
    let shared = parent.join("shared");
    fs::create_dir_all(&shared).expect("mkdir shared");
    make_project(&proj, "proj_with_shared");
    register_and_use(&proj);

    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok", "expected status=ok, got {r}");

    let sr = r["shared_root"]
        .as_str()
        .unwrap_or_else(|| panic!("shared_root must be a string, got {r}"));
    // Compare via canonicalize to survive macOS /var ↔ /private/var
    // symlink games and any trailing-slash normalization.
    let want = fs::canonicalize(&shared).expect("canonicalize shared");
    let got = fs::canonicalize(sr).expect("canonicalize shared_root");
    assert_eq!(got, want, "shared_root path mismatch: got {got:?}, want {want:?}");
}

/// Parent dir contains only `proj/` (no sibling `shared/`) →
/// `shared_root` is JSON null.
#[test]
fn resolve_shared_root_null_when_no_sibling() {
    let _g = HomeGuard::new();
    let tmp = TempDir::new().expect("tempdir");
    let parent = tmp.path().to_path_buf();
    let proj = parent.join("proj");
    make_project(&proj, "proj_alone");
    register_and_use(&proj);

    let r = json(&run_cli_ok(&["project", "resolve"]));
    assert_eq!(r["status"], "ok", "expected status=ok, got {r}");
    assert!(
        r["shared_root"].is_null(),
        "expected shared_root=null, got {:?}",
        r["shared_root"]
    );
}

// ---------------------------------------------------------------------------
// Sanity: the `project gate` flag parsing (belt-and-braces — a single
// check so a future refactor that breaks flag parsing fails loudly, not
// silently).
// ---------------------------------------------------------------------------

#[test]
fn gate_rejects_missing_args_flag() {
    let _g = HomeGuard::new();
    let (_tmp, _root) = standalone_project("gate_missing_args");

    let out = run_cli(&["project", "gate", "--artifact-type", "prd"]);
    assert!(
        !out.status.success(),
        "gate must fail when --args is absent; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("--args") || combined.to_uppercase().contains("MISSING_FLAG"),
        "expected error to mention --args / MISSING_FLAG, got: {combined}"
    );
}
