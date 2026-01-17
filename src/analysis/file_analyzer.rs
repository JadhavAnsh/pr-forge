use crate::pr::model::FileChange;

/// Analyze files for patterns
pub fn analyze_files(files: &[String]) -> Vec<FileChange> {
    files
        .iter()
        .map(|path| FileChange {
            is_test: is_test_file(path),
            is_config: is_config_file(path),
            is_api: is_api_file(path),
            is_deleted: false, // TODO: track from diff
            path: path.clone(),
        })
        .collect()
}

/// Check if file is a test file
fn is_test_file(path: &str) -> bool {
    path.contains("test")
        || path.contains("spec")
        || path.contains("__tests__")
        || path.ends_with(".test.rs")
        || path.ends_with(".test.ts")
        || path.ends_with(".spec.ts")
}

/// Check if file is a config file
fn is_config_file(path: &str) -> bool {
    path.ends_with(".env")
        || path.ends_with(".yml")
        || path.ends_with(".yaml")
        || path.ends_with(".json")
        || path.contains("docker-compose")
        || path.contains("config/")
        || path.ends_with("Dockerfile")
        || path.ends_with(".toml")
}

/// Check if file is API-related
fn is_api_file(path: &str) -> bool {
    path.contains("src/api")
        || path.contains("src/handlers")
        || path.contains("src/routes")
        || path.ends_with("lib.rs")
        || path.ends_with("mod.rs") && path.starts_with("src/")
}

/// Detect if there are breaking changes
#[allow(dead_code)]
pub fn detect_breaking_changes(files: &[FileChange]) -> bool {
    files.iter().any(|f| {
        f.is_deleted && f.is_api
    })
}

/// Summary of file categories
pub fn categorize_files(files: &[FileChange]) -> (usize, usize) {
    let tests = files.iter().filter(|f| f.is_test).count();
    let configs = files.iter().filter(|f| f.is_config).count();

    (tests, configs)
}
