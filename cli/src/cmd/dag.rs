use crate::helpers;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.len() < 2 { return Err("Usage: compass-cli dag <check|waves> <path>".into()); }
    let data = helpers::read_json(Path::new(&args[1]))?;
    let tasks: Vec<serde_json::Value> = if let Some(arr) = data.get("colleagues").and_then(|t| t.as_array()) {
        arr.clone()
    } else if let Some(arr) = data.get("tasks").and_then(|t| t.as_array()) {
        arr.clone()
    } else if let Some(waves) = data.get("waves").and_then(|w| w.as_array()) {
        waves.iter()
            .filter_map(|w| w.get("tasks").and_then(|t| t.as_array()))
            .flat_map(|arr| arr.iter().cloned())
            .collect()
    } else {
        return Err("Missing tasks/colleagues/waves array".into());
    };

    match args[0].as_str() {
        "check" => dag_check(&tasks),
        "waves" => dag_waves(&tasks),
        _ => Err(format!("Unknown dag command: {}", args[0])),
    }
}

fn dag_check(tasks: &[serde_json::Value]) -> Result<String, String> {
    let ids: HashSet<&str> = tasks.iter().filter_map(|t| t["id"].as_str().or_else(|| t["task_id"].as_str())).collect();
    let mut dangling = vec![];
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for task in tasks {
        let id = task["id"].as_str().or_else(|| task["task_id"].as_str()).unwrap_or("");
        adj.entry(id).or_default();
        if let Some(deps) = task.get("depends_on").and_then(|d| d.as_array()) {
            for dep in deps {
                if let Some(dep_id) = dep.as_str() {
                    if !ids.contains(dep_id) { dangling.push(format!("{} -> {}", id, dep_id)); }
                    adj.entry(dep_id).or_default().push(id);
                }
            }
        }
    }

    // Cycle detection via DFS
    let mut visited: HashSet<&str> = HashSet::new();
    let mut stack: HashSet<&str> = HashSet::new();
    let mut cycles = vec![];

    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        stack: &mut HashSet<&'a str>,
        cycles: &mut Vec<String>,
    ) {
        visited.insert(node);
        stack.insert(node);
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                if !visited.contains(next) {
                    dfs(next, adj, visited, stack, cycles);
                } else if stack.contains(next) {
                    cycles.push(format!("{} -> {}", node, next));
                }
            }
        }
        stack.remove(node);
    }

    for &id in &ids {
        if !visited.contains(id) {
            dfs(id, &adj, &mut visited, &mut stack, &mut cycles);
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "valid": cycles.is_empty() && dangling.is_empty(),
        "cycles": cycles,
        "dangling": dangling,
    })).unwrap())
}

fn dag_waves(tasks: &[serde_json::Value]) -> Result<String, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut task_map: HashMap<String, &serde_json::Value> = HashMap::new();

    for task in tasks {
        let id = task["id"].as_str().or_else(|| task["task_id"].as_str()).unwrap_or("").to_string();
        in_degree.entry(id.clone()).or_insert(0);
        adj.entry(id.clone()).or_default();
        task_map.insert(id.clone(), task);

        if let Some(deps) = task.get("depends_on").and_then(|d| d.as_array()) {
            for dep in deps {
                if let Some(dep_id) = dep.as_str() {
                    adj.entry(dep_id.to_string()).or_default().push(id.clone());
                    *in_degree.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let mut waves: Vec<Vec<serde_json::Value>> = vec![];
    let mut queue: VecDeque<String> = in_degree.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    while !queue.is_empty() {
        let mut wave = vec![];
        let mut next_queue = VecDeque::new();
        while let Some(id) = queue.pop_front() {
            if let Some(task) = task_map.get(&id) {
                wave.push(json!({
                    "id": id,
                    "name": task["name"].as_str().unwrap_or(""),
                    "complexity": task["complexity"].as_str().unwrap_or(""),
                    "budget_tokens": task["budget_tokens"].as_u64().unwrap_or(0),
                }));
            }
            if let Some(neighbors) = adj.get(&id) {
                for next in neighbors {
                    if let Some(deg) = in_degree.get_mut(next) {
                        *deg -= 1;
                        if *deg == 0 { next_queue.push_back(next.clone()); }
                    }
                }
            }
        }
        if !wave.is_empty() { waves.push(wave); }
        queue = next_queue;
    }

    Ok(serde_json::to_string_pretty(&json!({
        "wave_count": waves.len(),
        "waves": waves,
    })).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;

    fn run_with(plan: &serde_json::Value, subcmd: &str) -> Result<String, String> {
        let tmp = tempfile::NamedTempFile::new().expect("tmp");
        write!(tmp.as_file(), "{}", plan).expect("write plan");
        let args = vec![subcmd.to_string(), tmp.path().to_str().unwrap().to_string()];
        run(&args)
    }

    #[test]
    fn test_dag_check_flat_tasks() {
        let plan = json!({
            "tasks": [
                { "task_id": "T1" },
                { "task_id": "T2", "depends_on": ["T1"] }
            ]
        });
        let out = run_with(&plan, "check").expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["valid"], true);
        assert_eq!(v["cycles"].as_array().unwrap().len(), 0);
        assert_eq!(v["dangling"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_dag_check_waves_based() {
        let plan = json!({
            "waves": [
                { "wave_id": 1, "tasks": [{ "task_id": "T1" }, { "task_id": "T2" }] },
                { "wave_id": 2, "tasks": [{ "task_id": "T3", "depends_on": ["T1"] }] }
            ]
        });
        let out = run_with(&plan, "check").expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["valid"], true, "expected valid:true, got: {}", out);
    }

    #[test]
    fn test_dag_check_waves_with_dangling_dep() {
        let plan = json!({
            "waves": [
                { "wave_id": 1, "tasks": [{ "task_id": "T1", "depends_on": ["MISSING"] }] }
            ]
        });
        let out = run_with(&plan, "check").expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["valid"], false);
        assert!(!v["dangling"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_dag_waves_extracts_from_waves_based_plan() {
        let plan = json!({
            "waves": [
                { "wave_id": 1, "tasks": [{ "task_id": "T1" }] },
                { "wave_id": 2, "tasks": [{ "task_id": "T2", "depends_on": ["T1"] }] }
            ]
        });
        let out = run_with(&plan, "waves").expect("ok");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["wave_count"], 2);
    }

    #[test]
    fn test_dag_run_returns_err_for_missing_arrays() {
        let plan = json!({ "plan_version": "1.0" }); // no tasks, no colleagues, no waves
        let err = run_with(&plan, "check").unwrap_err();
        assert!(
            err.contains("Missing"),
            "expected error mentioning Missing, got: {}",
            err
        );
    }
}
