use crate::helpers;
use serde_json::json;
use std::path::Path;

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

    let tasks_key = if plan.get("colleagues").is_some() { "colleagues" } else { "tasks" };
    let tasks = plan.get(tasks_key).and_then(|t| t.as_array())
        .ok_or("No tasks array in plan")?;

    let task = tasks.iter()
        .find(|t| t["id"].as_str() == Some(task_id) || t["task_id"].as_str() == Some(task_id))
        .ok_or(format!("Task {} not found", task_id))?;

    let mut files: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    if let Some(pointers) = task.get("context_pointers").and_then(|p| p.as_array()) {
        for ptr in pointers {
            if let Some(ptr_str) = ptr.as_str() {
                // Parse "path:start-end" or just "path"
                let (file_path, _range) = if let Some(colon_pos) = ptr_str.rfind(':') {
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
                    files.insert(file_path.to_string(), json!(content));
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
