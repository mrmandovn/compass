use serde_json::json;
use std::process::Command;

const DOC_DIRS: &[&str] = &["prd", "epics", "research", "technical", "wiki", "compass"];

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_git(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())
}

pub fn run(args: &[String]) -> Result<String, String> {
    if !git_available() {
        return Ok(json!({"available": false}).to_string());
    }

    if args.is_empty() {
        return Err("Usage: compass-cli git <branch|commit|status> [args...]".into());
    }

    match args[0].as_str() {
        "branch" => cmd_branch(args),
        "commit" => cmd_commit(args),
        "status" => cmd_status(),
        _ => Err(format!("Unknown git command: {}", args[0])),
    }
}

fn cmd_branch(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err("Usage: compass-cli git branch <name>".into());
    }
    let name = &args[1];
    let branch = format!("docs/{}", name);

    let out = run_git(&["checkout", "-b", &branch])
        .map_err(|e| e.to_string())?;

    if out.status.success() {
        Ok(json!({
            "success": true,
            "branch": branch,
            "action": "created_and_checked_out"
        })
        .to_string())
    } else {
        // Branch may already exist — try just checking out
        let out2 = run_git(&["checkout", &branch]).map_err(|e| e.to_string())?;
        if out2.status.success() {
            Ok(json!({
                "success": true,
                "branch": branch,
                "action": "checked_out_existing"
            })
            .to_string())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Err(format!("git checkout failed: {}", stderr))
        }
    }
}

fn cmd_commit(args: &[String]) -> Result<String, String> {
    if args.len() < 2 {
        return Err("Usage: compass-cli git commit <message>".into());
    }
    let message = &args[1];

    // Stage only files inside designated doc dirs
    let mut staged_count = 0usize;
    for dir in DOC_DIRS {
        let out = run_git(&["add", "--", dir]).map_err(|e| e.to_string())?;
        if out.status.success() {
            staged_count += 1;
        }
    }

    // Check if there is anything staged
    let status_out = run_git(&["diff", "--cached", "--name-only"]).map_err(|e| e.to_string())?;
    let staged_files: Vec<String> = String::from_utf8_lossy(&status_out.stdout)
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if staged_files.is_empty() {
        return Ok(json!({
            "success": false,
            "reason": "nothing_to_commit",
            "staged": staged_count
        })
        .to_string());
    }

    let commit_out = run_git(&["commit", "-m", message]).map_err(|e| e.to_string())?;

    if commit_out.status.success() {
        Ok(json!({
            "success": true,
            "message": message,
            "files": staged_files
        })
        .to_string())
    } else {
        let stderr = String::from_utf8_lossy(&commit_out.stderr).to_string();
        Err(format!("git commit failed: {}", stderr))
    }
}

fn cmd_status() -> Result<String, String> {
    // Current branch
    let branch_out = run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).map_err(|e| e.to_string())?;
    let branch = String::from_utf8_lossy(&branch_out.stdout)
        .trim()
        .to_string();

    // Changed files in doc dirs only
    let status_out = run_git(&["status", "--porcelain"]).map_err(|e| e.to_string())?;
    let changed: Vec<String> = String::from_utf8_lossy(&status_out.stdout)
        .lines()
        .filter_map(|line| {
            let path = line.get(3..).unwrap_or("").trim().to_string();
            let in_doc_dir = DOC_DIRS.iter().any(|d| path.starts_with(d));
            if in_doc_dir {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    Ok(json!({
        "available": true,
        "branch": branch,
        "changed_files": changed,
        "changed_count": changed.len()
    })
    .to_string())
}
