use std::path::Path;
use std::fs;

/// Read file contents, return error if missing
pub fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))
}

/// Read and parse JSON file
pub fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let content = read_file(path)?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))
}

/// Write JSON to file with pretty printing
pub fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| format!("JSON serialize error: {}", e))?;
    fs::write(path, content)
        .map_err(|e| format!("Cannot write {}: {}", path.display(), e))
}

/// Parse YAML-like frontmatter from markdown (between --- delimiters)
pub fn parse_frontmatter(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first()? != &"---" { return None; }
    let end = lines.iter().skip(1).position(|l| *l == "---")?;
    let mut map = std::collections::HashMap::new();
    for line in &lines[1..=end] {
        if let Some((key, val)) = line.split_once(':') {
            map.insert(
                key.trim().to_string(),
                val.trim().trim_matches('"').to_string(),
            );
        }
    }
    Some(map)
}
