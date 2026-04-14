use crate::helpers;
use serde_json::json;
use std::path::Path;

pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() || args[0] != "check" {
        return Err("Usage: compass-cli manifest check".into());
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let compass_dir = Path::new(&home).join(".compass");
    let manifest = helpers::read_json(&compass_dir.join("core").join("manifest.json"))?;

    let mut errors: Vec<String> = vec![];
    if let Some(commands) = manifest.get("commands").and_then(|c| c.as_array()) {
        for cmd in commands {
            let name = cmd.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let workflow = cmd.get("workflow").and_then(|w| w.as_str()).unwrap_or("");

            // Check workflow exists
            if !compass_dir.join("core").join("workflows").join(workflow).exists() {
                errors.push(format!("Missing workflow: core/workflows/{}", workflow));
            }
            // Check command adapter
            if !compass_dir.join("core/commands/compass").join(format!("{}.md", name)).exists() {
                errors.push(format!("Missing command: {}.md", name));
            }
        }
    }

    Ok(serde_json::to_string_pretty(&json!({
        "valid": errors.is_empty(),
        "errors": errors,
    })).unwrap())
}
