use crate::helpers;
use serde_json::json;
use std::path::Path;

pub mod prd;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("Usage: compass-cli validate <spec|plan|tests|prd> <path>".into());
    }
    match args[0].as_str() {
        "--help" | "-h" | "help" => Ok(help_text()),
        "prd" => {
            if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h") {
                return Ok(prd_help_text());
            }
            if args.len() < 2 {
                return Err("Usage: compass-cli validate prd <path>".into());
            }
            prd::validate_prd(Path::new(&args[1]))
        }
        _ => {
            if args.len() < 2 {
                return Err("Usage: compass-cli validate <spec|plan|tests|prd> <path>".into());
            }
            match args[0].as_str() {
                "spec" => validate_spec(Path::new(&args[1])),
                "plan" => validate_plan(Path::new(&args[1])),
                "tests" => validate_tests(Path::new(&args[1])),
                other => Err(format!("Unknown validate target: {}", other)),
            }
        }
    }
}

fn help_text() -> String {
    "compass-cli validate <spec|plan|tests|prd> <path>\n\
     \n\
     Subcommands:\n  \
       spec  <path>   Validate a spec markdown file\n  \
       plan  <path>   Validate a plan.json (v1.0 schema)\n  \
       tests <path>   Validate a tests markdown file\n  \
       prd   <path>   Validate a PRD markdown file (R-FLOW, R-XREF)\n"
        .to_string()
}

fn prd_help_text() -> String {
    "compass-cli validate prd <path>\n\
     \n\
     Runs PRD taste rules on the given markdown file:\n  \
       R-FLOW  User Flows sections must be ordered numeric lists\n  \
       R-XREF  [LINK-*], [EPIC-*], [REQ-*] tokens must resolve\n\
     \n\
     Emits JSON: {\"ok\": bool, \"violations\": [...]}\n\
     Exit code: 0 if ok, 1 if violations.\n"
        .to_string()
}

fn validate_spec(path: &Path) -> Result<String, String> {
    let content = helpers::read_file(path)?;
    let fm = helpers::parse_frontmatter(&content);
    let mut errors: Vec<String> = vec![];
    let mut warnings: Vec<String> = vec![];
    let mut component = String::new();

    match fm {
        Some(map) => {
            for field in ["spec_version", "project", "component", "task_type", "category", "status"] {
                if !map.contains_key(field) || map[field].is_empty() {
                    errors.push(format!("Missing frontmatter field: {}", field));
                }
            }
            component = map.get("component").cloned().unwrap_or_default();
            // Check for required sections
            if !content.contains("## Overview") { warnings.push("Missing ## Overview section".into()); }
            if !content.contains("## Acceptance") && !content.contains("## Acceptance Criteria") {
                warnings.push("Missing ## Acceptance Criteria section".into());
            }
        }
        None => errors.push("No YAML frontmatter found (expected --- delimiters)".into()),
    }

    Ok(serde_json::to_string_pretty(&json!({
        "valid": errors.is_empty(),
        "component": component,
        "errors": errors,
        "warnings": warnings,
    })).unwrap())
}

/// v1.0 plan validator. Enforces the schema documented in
/// `core/shared/SCHEMAS-v1.md`. Legacy plans (no `plan_version` or a
/// pre-1.0 string) get a backward-compatible pass through the old checks
/// plus a guidance error pointing to `compass-cli migrate`.
fn validate_plan(path: &Path) -> Result<String, String> {
    let data = helpers::read_json(path)?;
    let mut errors: Vec<serde_json::Value> = vec![];
    let mut warnings: Vec<String> = vec![];

    let plan_version = data.get("plan_version").and_then(|v| v.as_str());

    // Dispatch on plan_version. v1.0 is the new schema. Anything else
    // falls through to the legacy checks but flags an upgrade hint.
    match plan_version {
        Some("1.0") => {
            validate_plan_v1(&data, &mut errors, &mut warnings);
        }
        Some(other) => {
            errors.push(violation(
                "UNSUPPORTED_PLAN_VERSION",
                None,
                Some("plan_version"),
                &format!(
                    "Unsupported plan_version '{}'. Run `compass-cli migrate` to upgrade to 1.0.",
                    other
                ),
            ));
            validate_plan_legacy(&data, &mut errors, &mut warnings);
        }
        None => {
            errors.push(violation(
                "MISSING_PLAN_VERSION",
                None,
                Some("plan_version"),
                "Missing plan_version. Run `compass-cli migrate` to upgrade to 1.0.",
            ));
            validate_plan_legacy(&data, &mut errors, &mut warnings);
        }
    }

    let task_count = count_tasks(&data);

    Ok(serde_json::to_string_pretty(&json!({
        "valid": errors.is_empty(),
        "ok": errors.is_empty(),
        "task_count": task_count,
        "violations": errors,
        "warnings": warnings,
    })).unwrap())
}

fn count_tasks(data: &serde_json::Value) -> usize {
    // v1.0: sum of tasks across waves; legacy: length of tasks/colleagues.
    if let Some(waves) = data.get("waves").and_then(|w| w.as_array()) {
        return waves
            .iter()
            .filter_map(|w| w.get("tasks").and_then(|t| t.as_array()))
            .map(|a| a.len())
            .sum();
    }
    let tasks_key = if data.get("colleagues").is_some() { "colleagues" } else { "tasks" };
    data.get(tasks_key)
        .and_then(|t| t.as_array())
        .map(|a| a.len())
        .unwrap_or(0)
}

fn violation(
    rule: &str,
    task_id: Option<&str>,
    field: Option<&str>,
    message: &str,
) -> serde_json::Value {
    json!({
        "rule": rule,
        "task_id": task_id.unwrap_or(""),
        "field": field.unwrap_or(""),
        "message": message,
    })
}

fn validate_plan_v1(
    data: &serde_json::Value,
    errors: &mut Vec<serde_json::Value>,
    _warnings: &mut Vec<String>,
) {
    // Top-level required fields per SCHEMAS-v1.md §1.
    for field in ["session_id", "colleagues_selected", "memory_ref", "waves"] {
        if data.get(field).is_none() {
            errors.push(violation(
                "MISSING_FIELD",
                None,
                Some(field),
                &format!("Missing top-level field: {}", field),
            ));
        }
    }

    // Waves + tasks.
    if let Some(waves) = data.get("waves").and_then(|w| w.as_array()) {
        if waves.is_empty() {
            errors.push(violation(
                "EMPTY_WAVES",
                None,
                Some("waves"),
                "waves must be a non-empty array",
            ));
        }
        for wave in waves {
            let tasks = wave.get("tasks").and_then(|t| t.as_array());
            match tasks {
                Some(arr) if !arr.is_empty() => {
                    for task in arr {
                        validate_task_v1(task, errors);
                    }
                }
                _ => errors.push(violation(
                    "EMPTY_TASKS",
                    None,
                    Some("waves[].tasks"),
                    "Each wave must have a non-empty tasks array",
                )),
            }
        }
    }
}

fn validate_task_v1(task: &serde_json::Value, errors: &mut Vec<serde_json::Value>) {
    let task_id = task.get("task_id").and_then(|v| v.as_str()).unwrap_or("");

    // context_pointers: REQUIRED, non-empty, ≤ 30, each non-empty string.
    match task.get("context_pointers") {
        None => errors.push(violation(
            "MISSING_CONTEXT_POINTERS",
            Some(task_id),
            Some("context_pointers"),
            "Missing required field: context_pointers",
        )),
        Some(v) => match v.as_array() {
            None => errors.push(violation(
                "MISSING_CONTEXT_POINTERS",
                Some(task_id),
                Some("context_pointers"),
                "context_pointers must be an array",
            )),
            Some(arr) if arr.is_empty() => errors.push(violation(
                "EMPTY_CONTEXT_POINTERS",
                Some(task_id),
                Some("context_pointers"),
                "context_pointers must be a non-empty array (1..30 items)",
            )),
            Some(arr) if arr.len() > 30 => errors.push(violation(
                "TOO_MANY_POINTERS",
                Some(task_id),
                Some("context_pointers"),
                &format!(
                    "context_pointers has {} items; maximum is 30. Split the task.",
                    arr.len()
                ),
            )),
            Some(arr) => {
                for (i, item) in arr.iter().enumerate() {
                    match item.as_str() {
                        Some(s) if !s.trim().is_empty() => {}
                        _ => errors.push(violation(
                            "EMPTY_CONTEXT_POINTERS",
                            Some(task_id),
                            Some(&format!("context_pointers[{}]", i)),
                            "Each context_pointer must be a non-empty string",
                        )),
                    }
                }
            }
        },
    }
}

fn validate_plan_legacy(
    data: &serde_json::Value,
    errors: &mut Vec<serde_json::Value>,
    warnings: &mut Vec<String>,
) {
    // Preserve the previous (pre-1.0) behavior so existing callers still
    // get their familiar error surface on top of the upgrade hint.
    for field in ["name", "workspace_dir", "budget_tokens"] {
        if data.get(field).is_none() {
            errors.push(violation(
                "MISSING_FIELD",
                None,
                Some(field),
                &format!("Missing top-level field: {}", field),
            ));
        }
    }

    let tasks_key = if data.get("colleagues").is_some() { "colleagues" } else { "tasks" };
    match data.get(tasks_key).and_then(|t| t.as_array()) {
        Some(arr) => {
            for (i, task) in arr.iter().enumerate() {
                for field in ["id", "name", "complexity", "budget_tokens"] {
                    if task.get(field).is_none() {
                        errors.push(violation(
                            "MISSING_FIELD",
                            task.get("id").and_then(|v| v.as_str()),
                            Some(field),
                            &format!("Task {}: missing field '{}'", i, field),
                        ));
                    }
                }
                match task.get("files").or_else(|| task.get("output_files")) {
                    Some(f) if f.as_array().map_or(true, |a| a.is_empty()) => {
                        if task.get("files").is_some() {
                            errors.push(violation(
                                "EMPTY_FILES",
                                task.get("id").and_then(|v| v.as_str()),
                                Some("files"),
                                &format!(
                                    "Task {}: files must be non-empty array",
                                    task.get("id").and_then(|v| v.as_str()).unwrap_or("?")
                                ),
                            ));
                        }
                    }
                    None => {
                        if tasks_key == "tasks" {
                            errors.push(violation(
                                "MISSING_FIELD",
                                task.get("id").and_then(|v| v.as_str()),
                                Some("files"),
                                &format!("Task {}: missing 'files' field", i),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            let declared = data.get("budget_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let sum: u64 = arr
                .iter()
                .filter_map(|t| t.get("budget_tokens").and_then(|v| v.as_u64()))
                .sum();
            if declared != 0 && declared != sum {
                warnings.push(format!(
                    "Budget mismatch: declared {}, tasks sum to {}",
                    declared, sum
                ));
            }
        }
        None => errors.push(violation(
            "MISSING_FIELD",
            None,
            Some(tasks_key),
            &format!("Missing '{}' array", tasks_key),
        )),
    }
}

fn validate_tests(path: &Path) -> Result<String, String> {
    let content = helpers::read_file(path)?;
    let fm = helpers::parse_frontmatter(&content);
    let mut errors: Vec<String> = vec![];
    let warnings: Vec<String> = vec![];
    let mut component = String::new();

    match fm {
        Some(map) => {
            for field in ["tests_version", "spec_ref", "component", "category", "strategy"] {
                if !map.contains_key(field) || map[field].is_empty() {
                    errors.push(format!("Missing frontmatter field: {}", field));
                }
            }
            component = map.get("component").cloned().unwrap_or_default();
        }
        None => errors.push("No YAML frontmatter found".into()),
    }

    Ok(serde_json::to_string_pretty(&json!({
        "valid": errors.is_empty(),
        "component": component,
        "errors": errors,
        "warnings": warnings,
    })).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plan_ok() -> serde_json::Value {
        json!({
            "plan_version": "1.0",
            "session_id": "s1",
            "colleagues_selected": ["writer"],
            "memory_ref": ".compass/.state/project-memory.json",
            "waves": [{
                "wave_id": 1,
                "tasks": [{
                    "task_id": "C-01",
                    "colleague": "writer",
                    "budget": 1000,
                    "depends_on": [],
                    "briefing_notes": "x",
                    "context_pointers": ["PRDs/*.md"],
                    "output_pattern": "PRDs/out.md"
                }]
            }]
        })
    }

    #[test]
    fn v1_happy_path_has_no_violations() {
        let mut errs = vec![];
        let mut warns = vec![];
        validate_plan_v1(&plan_ok(), &mut errs, &mut warns);
        assert!(errs.is_empty(), "expected no violations, got {:?}", errs);
    }

    #[test]
    fn v1_empty_context_pointers_fails() {
        let mut p = plan_ok();
        p["waves"][0]["tasks"][0]["context_pointers"] = json!([]);
        let mut errs = vec![];
        let mut warns = vec![];
        validate_plan_v1(&p, &mut errs, &mut warns);
        assert!(errs.iter().any(|e| e["rule"] == "EMPTY_CONTEXT_POINTERS"));
    }

    #[test]
    fn v1_missing_context_pointers_fails() {
        let mut p = plan_ok();
        p["waves"][0]["tasks"][0]
            .as_object_mut()
            .unwrap()
            .remove("context_pointers");
        let mut errs = vec![];
        let mut warns = vec![];
        validate_plan_v1(&p, &mut errs, &mut warns);
        assert!(errs.iter().any(|e| e["rule"] == "MISSING_CONTEXT_POINTERS"));
    }

    #[test]
    fn v1_too_many_pointers_fails() {
        let mut p = plan_ok();
        let many: Vec<String> = (0..31).map(|i| format!("p-{}.md", i)).collect();
        p["waves"][0]["tasks"][0]["context_pointers"] = json!(many);
        let mut errs = vec![];
        let mut warns = vec![];
        validate_plan_v1(&p, &mut errs, &mut warns);
        assert!(errs.iter().any(|e| e["rule"] == "TOO_MANY_POINTERS"));
    }

}

#[cfg(test)]
mod plan_tests {
    //! Fixture-backed end-to-end tests for `validate_plan`. Each test loads
    //! a real JSON fixture under `cli/tests/fixtures/`, runs the public
    //! `validate_plan` entry point, and asserts on the JSON result.
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn run_plan(name: &str) -> serde_json::Value {
        let path = fixture(name);
        let out = validate_plan(&path).expect("validate_plan returned Err");
        serde_json::from_str(&out).expect("validate_plan output was not JSON")
    }

    #[test]
    fn plan_v1_valid() {
        let result = run_plan("plan_v1_valid.json");
        assert_eq!(result["ok"], serde_json::Value::Bool(true),
            "expected ok=true, got: {}", result);
        let violations = result["violations"].as_array().expect("violations must be array");
        assert!(violations.is_empty(),
            "expected no violations, got: {:?}", violations);
    }

    #[test]
    fn plan_missing_pointers() {
        let result = run_plan("plan_missing_pointers.json");
        assert_eq!(result["ok"], serde_json::Value::Bool(false));
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "MISSING_CONTEXT_POINTERS")
            .expect("expected MISSING_CONTEXT_POINTERS violation");
        assert_eq!(hit["task_id"], "C-02",
            "task_id should be set on the offending task: {:?}", hit);
    }

    #[test]
    fn empty_pointers() {
        let result = run_plan("plan_empty_pointers.json");
        assert_eq!(result["ok"], serde_json::Value::Bool(false));
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "EMPTY_CONTEXT_POINTERS")
            .expect("expected EMPTY_CONTEXT_POINTERS violation");
        assert_eq!(hit["task_id"], "C-02");
    }

    #[test]
    fn too_many_pointers() {
        let result = run_plan("plan_too_many_pointers.json");
        assert_eq!(result["ok"], serde_json::Value::Bool(false));
        let violations = result["violations"].as_array().expect("violations must be array");
        let hit = violations.iter().find(|v| v["rule"] == "TOO_MANY_POINTERS")
            .expect("expected TOO_MANY_POINTERS violation");
        assert_eq!(hit["task_id"], "C-02");
    }
}
