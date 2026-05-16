use crate::helpers;
use serde_json::json;
use std::path::Path;

/// Find a task by `task_id` or `id` in a plan, scanning flat `tasks[]`,
/// `colleagues[]`, then `waves[].tasks[]` in that priority order.
fn find_task<'a>(plan: &'a serde_json::Value, task_id: &str) -> Option<&'a serde_json::Value> {
    let matches = |t: &&serde_json::Value| {
        t.get("id").and_then(|v| v.as_str()) == Some(task_id)
            || t.get("task_id").and_then(|v| v.as_str()) == Some(task_id)
    };

    if let Some(arr) = plan.get("tasks").and_then(|t| t.as_array()) {
        if let Some(t) = arr.iter().find(matches) {
            return Some(t);
        }
    }
    if let Some(arr) = plan.get("colleagues").and_then(|t| t.as_array()) {
        if let Some(t) = arr.iter().find(matches) {
            return Some(t);
        }
    }
    if let Some(waves) = plan.get("waves").and_then(|w| w.as_array()) {
        for wave in waves {
            if let Some(tasks) = wave.get("tasks").and_then(|t| t.as_array()) {
                if let Some(t) = tasks.iter().find(matches) {
                    return Some(t);
                }
            }
        }
    }
    None
}

pub fn run(args: &[String]) -> Result<String, String> {
    if args.len() < 3 { return Err("Usage: compass-cli context pack <session_dir> <task_id>".into()); }
    if args[0] != "pack" && args[0] != "get" {
        return Err(format!("Unknown context command: {}", args[0]));
    }

    let session_dir = Path::new(&args[1]);
    let task_id = &args[2];
    let plan_path = session_dir.join("plan.json");

    if !plan_path.exists() { return Err("plan.json not found in session dir".into()); }
    let plan = helpers::read_json(&plan_path)?;

    let task = find_task(&plan, task_id)
        .ok_or(format!("Task {} not found", task_id))?;

    let mut files: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    if let Some(pointers) = task.get("context_pointers").and_then(|p| p.as_array()) {
        for ptr in pointers {
            if let Some(ptr_str) = ptr.as_str() {
                // Parse "path:start-end" or just "path"
                let (file_path, range) = if let Some(colon_pos) = ptr_str.rfind(':') {
                    let maybe_range = &ptr_str[colon_pos + 1..];
                    if maybe_range.contains('-') && maybe_range.chars().all(|c| c.is_ascii_digit() || c == '-') {
                        (&ptr_str[..colon_pos], Some(maybe_range))
                    } else {
                        (ptr_str, None)
                    }
                } else {
                    (ptr_str, None)
                };

                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let sliced = if let Some(range_str) = range {
                        let parts: Vec<&str> = range_str.split('-').collect();
                        if parts.len() == 2 {
                            let start: usize = parts[0].parse().unwrap_or(1);
                            let end: usize = parts[1].parse().unwrap_or(usize::MAX);
                            let lines: Vec<&str> = content.lines().collect();
                            let start_idx = start.saturating_sub(1); // 1-based to 0-based
                            let end_idx = end.min(lines.len());
                            lines[start_idx..end_idx].join("\n")
                        } else {
                            content
                        }
                    } else {
                        content
                    };
                    files.insert(ptr_str.to_string(), json!(sliced));
                }
            }
        }
    }

    let output = json!({"task_id": task_id, "files": files});

    // Write context pack file
    if args[0] == "pack" {
        let out_path = session_dir.join(format!("{}.context.json", task_id));
        helpers::write_json(&out_path, &output)?;
    }

    Ok(serde_json::to_string_pretty(&output).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_find_task_in_flat_tasks() {
        let plan = json!({
            "tasks": [
                { "task_id": "T1", "context_pointers": ["a.rs"] },
                { "task_id": "T2", "context_pointers": ["b.rs"] }
            ]
        });
        let t = find_task(&plan, "T2").expect("T2 should be found");
        assert_eq!(t["task_id"], "T2");
    }

    #[test]
    fn test_find_task_in_colleagues() {
        let plan = json!({
            "colleagues": [
                { "id": "C1", "context_pointers": ["x.md"] }
            ]
        });
        let t = find_task(&plan, "C1").expect("C1 should be found");
        assert_eq!(t["id"], "C1");
    }

    #[test]
    fn test_find_task_in_waves() {
        let plan = json!({
            "waves": [
                { "wave_id": 1, "tasks": [{ "task_id": "T1" }] },
                { "wave_id": 2, "tasks": [{ "task_id": "T2", "context_pointers": ["a.rs"] }] }
            ]
        });
        let t = find_task(&plan, "T2").expect("T2 in wave 2 should be found");
        assert_eq!(t["task_id"], "T2");
    }

    #[test]
    fn test_find_task_not_found() {
        let plan = json!({ "tasks": [{ "task_id": "X" }] });
        assert!(find_task(&plan, "Y").is_none());
    }
}
