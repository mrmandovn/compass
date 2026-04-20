// Parse xlsx sprint sheets for compass:sprint import.
// Detects sheet structure: header row with "Key" in col A,
// extracts tasks + member allocations + priority.

use calamine::{open_workbook_auto, Data, Reader};
use serde_json::{json, Value};
use std::path::Path;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("Usage: compass-cli sheet <list|parse> <xlsx_path> [sheet_name]".into());
    }
    match args[0].as_str() {
        "list" => {
            if args.len() < 2 {
                return Err("Usage: compass-cli sheet list <xlsx_path>".into());
            }
            list_sheets(&args[1])
        }
        "parse" => {
            if args.len() < 2 {
                return Err(
                    "Usage: compass-cli sheet parse <xlsx_path> [sheet_name]".into(),
                );
            }
            let sheet = args.get(2).map(|s| s.as_str());
            parse_sheet(&args[1], sheet)
        }
        other => Err(format!("Unknown sheet subcommand: {}", other)),
    }
}

fn list_sheets(path: &str) -> Result<String, String> {
    let wb = open_workbook_auto(Path::new(path))
        .map_err(|e| format!("Open workbook: {}", e))?;
    let names = wb.sheet_names();
    let sheets: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

    // Detect latest sprint sheet (Sprint N pattern, highest N)
    let mut latest_sprint: Option<(i32, String)> = None;
    for name in &sheets {
        if let Some(n) = parse_sprint_number(name) {
            if latest_sprint.as_ref().map_or(true, |(prev, _)| n > *prev) {
                latest_sprint = Some((n, name.to_string()));
            }
        }
    }

    Ok(json!({
        "sheets": sheets,
        "latest_sprint": latest_sprint.map(|(n, name)| json!({"number": n, "name": name})),
    })
    .to_string())
}

fn parse_sprint_number(name: &str) -> Option<i32> {
    // Match "Sprint 20", "Sprint 16 - Tết", etc.
    let lower = name.to_lowercase();
    if !lower.starts_with("sprint") {
        return None;
    }
    let rest = &lower[6..].trim();
    let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

fn parse_sheet(path: &str, sheet_name: Option<&str>) -> Result<String, String> {
    let mut wb = open_workbook_auto(Path::new(path))
        .map_err(|e| format!("Open workbook: {}", e))?;

    let name = match sheet_name {
        Some(n) => n.to_string(),
        None => {
            // Auto-pick latest sprint
            let mut latest: Option<(i32, String)> = None;
            for n in wb.sheet_names() {
                if let Some(num) = parse_sprint_number(&n) {
                    if latest.as_ref().map_or(true, |(p, _)| num > *p) {
                        latest = Some((num, n.clone()));
                    }
                }
            }
            latest
                .ok_or("No 'Sprint N' sheet found. Specify sheet name.")?
                .1
        }
    };

    let range = wb
        .worksheet_range(&name)
        .map_err(|e| format!("Sheet not found: {}", e))?;

    // Find header row: first row with "Key" in column A
    let mut header_row_idx: Option<usize> = None;
    for (i, row) in range.rows().enumerate() {
        if let Some(Data::String(s)) = row.first() {
            if s.trim().eq_ignore_ascii_case("key") {
                header_row_idx = Some(i);
                break;
            }
        }
    }
    let header_idx = header_row_idx
        .ok_or("Header row not found — expected 'Key' in column A")?;

    let header_row: Vec<String> = range
        .rows()
        .nth(header_idx)
        .ok_or("header row missing")?
        .iter()
        .map(cell_to_string)
        .collect();

    // Detect columns:
    // A=Key, B=Task, C=Story point, then member columns,
    // then Goal, Com_Goal, Completed, Remark, Priority
    let key_idx = 0;
    let task_idx = 1;
    let points_idx = 2;

    // Member cols: from index 3 until we hit "Goal" or similar
    let goal_col_idx = header_row
        .iter()
        .position(|h| h.eq_ignore_ascii_case("Goal"))
        .unwrap_or(header_row.len());
    let member_cols: Vec<(usize, String)> = (3..goal_col_idx)
        .filter_map(|i| {
            header_row
                .get(i)
                .filter(|h| !h.is_empty())
                .map(|h| (i, h.clone()))
        })
        .collect();

    let priority_idx = header_row
        .iter()
        .position(|h| h.eq_ignore_ascii_case("Priority"))
        .unwrap_or(header_row.len() - 1);
    let status_idx = header_row
        .iter()
        .position(|h| h.eq_ignore_ascii_case("Com_Goal"))
        .unwrap_or(0);

    // Parse task rows (below header)
    let mut tasks: Vec<Value> = Vec::new();
    for (i, row) in range.rows().enumerate() {
        if i <= header_idx {
            continue;
        }
        let key = cell_to_string(&row.get(key_idx).cloned().unwrap_or(Data::Empty));
        let task_name = cell_to_string(&row.get(task_idx).cloned().unwrap_or(Data::Empty));
        // Skip only truly empty rows (no key AND no task name). Rows with task name
        // but no Jira key are valid — they represent work items not yet ticketed.
        // The sync workflow can surface them as "create new Jira ticket" candidates.
        if key.is_empty() && task_name.is_empty() {
            continue;
        }
        if task_name.is_empty() {
            continue;
        }

        let story_point =
            cell_to_f64(&row.get(points_idx).cloned().unwrap_or(Data::Empty));

        // Member allocations
        let mut members: Vec<Value> = Vec::new();
        for (col_idx, member_name) in &member_cols {
            let cell = row.get(*col_idx).cloned().unwrap_or(Data::Empty);
            if let Some(pts) = cell_to_points(&cell) {
                if pts > 0.0 {
                    members.push(json!({
                        "name": member_name,
                        "points": pts,
                    }));
                }
            }
        }

        let priority = cell_to_string(&row.get(priority_idx).cloned().unwrap_or(Data::Empty));
        let status = if status_idx > 0 {
            cell_to_string(&row.get(status_idx).cloned().unwrap_or(Data::Empty))
        } else {
            String::new()
        };

        let needs_subtasks = members.len() >= 2;
        // Keyless rows represent tasks that need Jira tickets created before sync.
        // Emit `key: null` so the sync workflow can surface them explicitly.
        let key_value: Value = if key.is_empty() {
            Value::Null
        } else {
            Value::String(key)
        };

        tasks.push(json!({
            "key": key_value,
            "name": task_name,
            "story_points": story_point,
            "priority": priority,
            "status": status,
            "members": members,
            "needs_subtasks": needs_subtasks,
        }));
    }

    Ok(json!({
        "sheet": name,
        "header_row": header_idx + 1,
        "member_columns": member_cols.iter().map(|(_, n)| n.clone()).collect::<Vec<_>>(),
        "task_count": tasks.len(),
        "tasks": tasks,
    })
    .to_string())
}

fn cell_to_string(c: &Data) -> String {
    match c {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => format!("{}", dt),
        _ => String::new(),
    }
}

fn cell_to_f64(c: &Data) -> f64 {
    match c {
        Data::Float(f) => *f,
        Data::Int(i) => *i as f64,
        Data::String(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn cell_to_points(c: &Data) -> Option<f64> {
    match c {
        Data::Float(f) => Some(*f),
        Data::Int(i) => Some(*i as f64),
        Data::String(s) => {
            let trimmed = s.trim();
            if trimmed == "-" || trimmed.is_empty() {
                None
            } else {
                trimmed.parse().ok()
            }
        }
        _ => None,
    }
}
