use crate::helpers;
use serde_json::json;
use std::path::Path;
use std::fs;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("Usage: compass-cli index <build|search|get> [args]".into());
    }
    match args[0].as_str() {
        "build" => build_index(args.get(1).map(|s| s.as_str())),
        "add" => {
            if args.len() < 3 { return Err("Usage: compass-cli index add <file> <type>".into()); }
            add_to_index(&args[1], &args[2])
        }
        "remove" => {
            if args.len() < 2 { return Err("Usage: compass-cli index remove <file>".into()); }
            remove_from_index(&args[1])
        }
        "search" => {
            if args.len() < 2 { return Err("Usage: compass-cli index search <keywords>".into()); }
            search_index(&args[1..].join(" "))
        }
        "get" => get_index(),
        _ => Err(format!("Unknown index command: {}", args[0])),
    }
}

/// Scan all 8 document types and build a lightweight index
fn build_index(project_dir: Option<&str>) -> Result<String, String> {
    let base = project_dir.unwrap_or(".");
    let base_path = Path::new(base);

    let scan_paths = vec![
        ("prd", vec!["prd", ".compass/PRDs"]),
        ("epic", vec!["epics"]),
        ("story", vec![".compass/Stories"]),
        ("idea", vec!["research"]),
        ("research", vec!["research"]),
        ("backlog", vec!["research"]),
        ("technical", vec!["technical", ".compass/Technical"]),
        ("wiki", vec!["wiki"]),
    ];

    let mut entries: Vec<serde_json::Value> = vec![];

    for (doc_type, dirs) in &scan_paths {
        for dir in dirs {
            let dir_path = base_path.join(dir);
            if !dir_path.exists() { continue; }

            scan_directory(&dir_path, doc_type, &mut entries, base_path);
        }
    }

    // Also scan epics subdirectories for stories
    let epics_path = base_path.join("epics");
    if epics_path.exists() {
        if let Ok(epic_dirs) = fs::read_dir(&epics_path) {
            for entry in epic_dirs.filter_map(|e| e.ok()) {
                let stories_dir = entry.path().join("user-stories");
                if stories_dir.exists() {
                    scan_directory(&stories_dir, "story", &mut entries, base_path);
                }
                // Also index epic.md itself
                let epic_file = entry.path().join("epic.md");
                if epic_file.exists() {
                    if let Some(e) = index_file(&epic_file, "epic", base_path) {
                        entries.push(e);
                    }
                }
            }
        }
    }

    let index = json!({
        "version": "1.0",
        "built_at": format!("{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs()),
        "count": entries.len(),
        "entries": entries,
    });

    // Write index to .compass/.state/index.json
    let index_path = base_path.join(".compass").join(".state").join("index.json");
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    helpers::write_json(&index_path, &index)?;

    Ok(serde_json::to_string_pretty(&json!({
        "success": true,
        "count": entries.len(),
        "path": index_path.to_string_lossy(),
    })).unwrap())
}

/// Scan a directory for .md files and extract title + keywords
fn scan_directory(dir: &Path, doc_type: &str, entries: &mut Vec<serde_json::Value>, base: &Path) {
    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };

    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "md") { continue; }
        if path.file_name().map_or(false, |n| n == "README.md") { continue; }

        if let Some(e) = index_file(&path, doc_type, base) {
            entries.push(e);
        }
    }
}

/// Index a single .md file — extract title, type, keywords from filename + first heading
fn index_file(path: &Path, doc_type: &str, base: &Path) -> Option<serde_json::Value> {
    let content = fs::read_to_string(path).ok()?;
    let rel_path = path.strip_prefix(base).unwrap_or(path).to_string_lossy().to_string();
    let filename = path.file_stem()?.to_string_lossy().to_string();

    // Extract title from first heading or frontmatter
    let title = content.lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .or_else(|| {
            // Try frontmatter title field
            helpers::parse_frontmatter(&content)
                .and_then(|fm| fm.get("title").cloned())
        })
        .unwrap_or_else(|| filename.clone());

    // Extract keywords: split filename + title into words, lowercase, deduplicate
    let mut keywords: Vec<String> = vec![];
    for word in filename.split(|c: char| c == '-' || c == '_' || c == '.') {
        let w = word.to_lowercase();
        if w.len() > 2 && !keywords.contains(&w) { keywords.push(w); }
    }
    for word in title.split_whitespace() {
        let w = word.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "");
        if w.len() > 2 && !keywords.contains(&w) { keywords.push(w); }
    }

    Some(json!({
        "path": rel_path,
        "type": doc_type,
        "title": title,
        "keywords": keywords,
        "filename": path.file_name()?.to_string_lossy(),
    }))
}

/// Add or update a single file in the index (O(1) instead of full rebuild)
fn add_to_index(file_path: &str, doc_type: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let base = Path::new(".");
    let entry = index_file(path, doc_type, base)
        .ok_or_else(|| format!("Could not index file: {}", file_path))?;

    let index_path = Path::new(".compass/.state/index.json");

    let mut index = if index_path.exists() {
        helpers::read_json(index_path)?
    } else {
        json!({
            "version": "1.0",
            "built_at": "0",
            "count": 0,
            "entries": [],
        })
    };

    // Remove existing entry with same path (if updating), then add new
    let rel_path = path.strip_prefix(base).unwrap_or(path).to_string_lossy().to_string();
    {
        let entries = index.get_mut("entries").and_then(|e| e.as_array_mut());
        if let Some(arr) = entries {
            arr.retain(|e| e.get("path").and_then(|p| p.as_str()) != Some(&rel_path));
            arr.push(entry);
        }
    }
    // Update count and timestamp
    let count = index.get("entries").and_then(|e| e.as_array()).map_or(0, |a| a.len());
    if let Some(obj) = index.as_object_mut() {
        obj.insert("count".to_string(), json!(count));
        obj.insert("built_at".to_string(), json!(format!("{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs())));
    }

    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    helpers::write_json(index_path, &index)?;

    Ok(json!({"success": true, "added": rel_path}).to_string())
}

/// Remove a file from the index
fn remove_from_index(file_path: &str) -> Result<String, String> {
    let index_path = Path::new(".compass/.state/index.json");
    if !index_path.exists() {
        return Ok(json!({"success": true, "note": "no index to update"}).to_string());
    }

    let mut index = helpers::read_json(index_path)?;
    let base = Path::new(".");
    let rel_path = Path::new(file_path).strip_prefix(base)
        .unwrap_or(Path::new(file_path)).to_string_lossy().to_string();

    let mut removed = false;
    if let Some(entries) = index.get_mut("entries").and_then(|e| e.as_array_mut()) {
        let before = entries.len();
        entries.retain(|e| e.get("path").and_then(|p| p.as_str()) != Some(&rel_path));
        removed = entries.len() < before;
    }

    let count = index.get("entries").and_then(|e| e.as_array()).map_or(0, |a| a.len());
    if let Some(obj) = index.as_object_mut() {
        obj.insert("count".to_string(), json!(count));
    }

    helpers::write_json(index_path, &index)?;
    Ok(json!({"success": true, "removed": removed, "path": rel_path}).to_string())
}

/// Check if index is outdated by comparing index built_at vs newest file mtime
fn is_index_outdated() -> bool {
    let index_path = Path::new(".compass/.state/index.json");
    if !index_path.exists() { return true; }

    // Get index mtime
    let index_mtime = match fs::metadata(index_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };

    // Scan all doc folders for any file newer than index
    let scan_dirs = vec![
        "prd", "epics", "research", "technical", "wiki",
        ".compass/PRDs", ".compass/Stories", ".compass/Ideas",
        ".compass/Research", ".compass/Technical", ".compass/Backlog",
    ];

    for dir in scan_dirs {
        let dir_path = Path::new(dir);
        if !dir_path.exists() { continue; }
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(meta) = entry.metadata() {
                    if let Ok(mtime) = meta.modified() {
                        if mtime > index_mtime { return true; }
                    }
                }
            }
        }
    }

    // Also check epic subdirectories
    if let Ok(entries) = fs::read_dir("epics") {
        for entry in entries.filter_map(|e| e.ok()) {
            let stories_dir = entry.path().join("user-stories");
            if stories_dir.exists() {
                if let Ok(files) = fs::read_dir(&stories_dir) {
                    for f in files.filter_map(|e| e.ok()) {
                        if let Ok(meta) = f.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                if mtime > index_mtime { return true; }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Search the index for matching entries — auto-rebuilds if outdated
fn search_index(query: &str) -> Result<String, String> {
    let index_path = Path::new(".compass/.state/index.json");

    // Auto-rebuild if missing or outdated
    if !index_path.exists() || is_index_outdated() {
        let _ = build_index(Some("."));
    }

    if !index_path.exists() {
        return Ok(json!({"matches": [], "hint": "Could not build index"}).to_string());
    }

    let index = helpers::read_json(index_path)?;
    let entries = index.get("entries").and_then(|e| e.as_array())
        .ok_or("Invalid index format")?;

    let query_words: Vec<String> = query.split_whitespace()
        .map(|w| w.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
        .filter(|w| w.len() > 2)
        .collect();

    if query_words.is_empty() {
        return Ok(json!({"matches": [], "hint": "Query too short"}).to_string());
    }

    let mut matches: Vec<(usize, &serde_json::Value)> = vec![];

    for entry in entries {
        // Skip entries whose file no longer exists (deleted)
        let entry_path = entry.get("path").and_then(|p| p.as_str()).unwrap_or("");
        if !entry_path.is_empty() && !Path::new(entry_path).exists() { continue; }

        let keywords = entry.get("keywords")
            .and_then(|k| k.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let title = entry.get("title").and_then(|t| t.as_str()).unwrap_or("").to_lowercase();

        let mut score = 0usize;
        for qw in &query_words {
            if keywords.iter().any(|k| k.contains(qw.as_str())) { score += 2; }
            if title.contains(qw.as_str()) { score += 1; }
        }

        if score >= 2 { matches.push((score, entry)); }
    }

    matches.sort_by(|a, b| b.0.cmp(&a.0));

    let results: Vec<serde_json::Value> = matches.iter().take(10)
        .map(|(score, entry)| {
            let mut e = (*entry).clone();
            if let Some(obj) = e.as_object_mut() {
                obj.insert("score".to_string(), json!(score));
            }
            e
        })
        .collect();

    Ok(serde_json::to_string_pretty(&json!({
        "query": query,
        "matches": results,
        "total": results.len(),
    })).unwrap())
}

/// Get the full index
fn get_index() -> Result<String, String> {
    let index_path = Path::new(".compass/.state/index.json");
    if !index_path.exists() {
        return Ok(json!({"exists": false, "hint": "Run compass-cli index build first"}).to_string());
    }
    let index = helpers::read_json(index_path)?;
    Ok(serde_json::to_string_pretty(&index).unwrap())
}
