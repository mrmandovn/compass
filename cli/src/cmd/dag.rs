use crate::helpers;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.len() < 2 { return Err("Usage: compass-cli dag <check|waves> <path>".into()); }
    let data = helpers::read_json(Path::new(&args[1]))?;
    let tasks_key = if data.get("colleagues").is_some() { "colleagues" } else { "tasks" };
    let tasks = data.get(tasks_key).and_then(|t| t.as_array())
        .ok_or("Missing tasks/colleagues array")?;

    match args[0].as_str() {
        "check" => dag_check(tasks),
        "waves" => dag_waves(tasks),
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
