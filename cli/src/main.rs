mod cmd;
mod helpers;
mod gdrive_client;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let is_validate = args[1].as_str() == "validate";
    let result = match args[1].as_str() {
        "validate" => cmd::validate::run(&args[2..]),
        "dag" => cmd::dag::run(&args[2..]),
        "session" => cmd::session::run(&args[2..]),
        "state" => cmd::state::run(&args[2..]),
        "version" => cmd::version::run(&args[2..]),
        "manifest" => cmd::manifest::run(&args[2..]),
        "hook" => cmd::hook::run(&args[2..]),
        "context" => cmd::context::run(&args[2..]),
        "memory" => cmd::memory::run(&args[2..]),
        "index" => cmd::index::run(&args[2..]),
        "git" => cmd::git::run(&args[2..]),
        "progress" => cmd::progress::run(&args[2..]),
        "migrate" => cmd::migrate::run(&args[2..]),
        "project" => cmd::project::run(&args[2..]),
        "gdrive" => cmd::gdrive::run(&args[2..]),
        "sheet" => cmd::sheet::run(&args[2..]),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    };

    match result {
        Ok(json) => {
            println!("{}", json);
            // For `validate`, propagate failure via exit code so scripts can
            // check success without parsing JSON. A result where `ok` or
            // `valid` is explicitly false → exit 1.
            if is_validate {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                    let ok = v.get("ok").and_then(|b| b.as_bool());
                    let valid = v.get("valid").and_then(|b| b.as_bool());
                    if ok == Some(false) || valid == Some(false) {
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            let err = serde_json::json!({"error": e.to_string()});
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: compass-cli <command> [args...]");
    eprintln!("");
    eprintln!("Commands:");
    eprintln!("  validate spec|plan|tests|prd <path>  Validate spec/plan/PRD files");
    eprintln!("  dag check|waves <path>           DAG operations");
    eprintln!("  session latest|list [dir]        Session management");
    eprintln!("  state get|update <dir> [json]    State management");
    eprintln!("  state get-config [dir]           Read cached config.json (5-min TTL)");
    eprintln!("  version                          Show version");
    eprintln!("  manifest check                   Verify installation");
    eprintln!("  hook statusline|update-checker   Lifecycle hooks");
    eprintln!("  context pack <dir> <taskId>      Generate context pack");
    eprintln!("  memory init|get|update <dir>     Project memory");
    eprintln!("  git branch|commit|status [args]  Git integration for docs");
    eprintln!("  progress save|load|clear <dir>   Progress persistence");
    eprintln!("  migrate <project_root>           Migrate v0.x state to v1.0 (idempotent)");
    eprintln!("  project resolve|list|use|add|remove|global-config  Project registry + active-project switching");
    eprintln!("  gdrive auth|status|download <id_or_url>  Google Drive OAuth + download (PKCE)");
    eprintln!("  sheet list|parse <xlsx> [name]   Parse sprint sheet from xlsx");
}
