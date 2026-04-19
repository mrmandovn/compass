use std::path::Path;

pub fn run(_args: &[String]) -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let version_path = Path::new(&home).join(".compass").join("VERSION");
    let version = std::fs::read_to_string(&version_path)
        .unwrap_or_else(|_| "0.4.0".to_string())
        .trim()
        .to_string();
    Ok(serde_json::json!({"version": version}).to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        // CARGO_MANIFEST_DIR points at cli/, so go up one level to reach repo root.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir)
            .parent()
            .expect("cli/ should have a parent directory (repo root)")
            .to_path_buf()
    }

    fn read_file(path: &PathBuf) -> String {
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e))
    }

    fn json_version(path: &PathBuf) -> String {
        let raw = read_file(path);
        let v: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("failed to parse JSON at {}: {}", path.display(), e));
        v.get("version")
            .and_then(|x| x.as_str())
            .unwrap_or_else(|| panic!("no string `version` field in {}", path.display()))
            .to_string()
    }

    #[test]
    fn all_sources_match() {
        let root = repo_root();

        let version_file = root.join("VERSION");
        let version_txt = read_file(&version_file)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();

        let package_json_version = json_version(&root.join("package.json"));
        let core_manifest_version = json_version(&root.join("core").join("manifest.json"));
        let colleagues_manifest_version =
            json_version(&root.join("core").join("colleagues").join("manifest.json"));
        let cargo_version = env!("CARGO_PKG_VERSION").to_string();

        assert_eq!(
            version_txt, package_json_version,
            "VERSION ({}) != package.json ({})",
            version_txt, package_json_version
        );
        assert_eq!(
            version_txt, core_manifest_version,
            "VERSION ({}) != core/manifest.json ({})",
            version_txt, core_manifest_version
        );
        assert_eq!(
            version_txt, colleagues_manifest_version,
            "VERSION ({}) != core/colleagues/manifest.json ({})",
            version_txt, colleagues_manifest_version
        );
        assert_eq!(
            version_txt, cargo_version,
            "VERSION ({}) != cli/Cargo.toml CARGO_PKG_VERSION ({})",
            version_txt, cargo_version
        );

        // Pin the expected version literal so the parity test catches
        // a partial bump (e.g. VERSION updated but Cargo.toml not)
        // even if all sources agree on the wrong value.
        assert_eq!(
            version_txt, "1.0.26",
            "VERSION pin mismatch — run ./scripts/bump-version.sh to sync; got {}",
            version_txt
        );
    }
}
