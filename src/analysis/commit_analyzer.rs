use crate::pr::model::{ChangeType, ClassifiedCommit, CommitInfo};

/// Analyze commits and classify them
pub fn analyze_commits(
    commits: Vec<CommitInfo>,
) -> Vec<ClassifiedCommit> {
    commits
        .into_iter()
        .map(|commit| {
            let (change_type, description) = classify_commit(&commit.message);
            ClassifiedCommit {
                info: commit,
                change_type,
                description,
            }
        })
        .collect()
}

/// Classify a commit by its message
fn classify_commit(message: &str) -> (ChangeType, String) {
    let lower = message.to_lowercase();

    let change_type = if lower.starts_with("feat") || lower.starts_with("feature") {
        ChangeType::Feature
    } else if lower.starts_with("fix") {
        ChangeType::Fix
    } else if lower.starts_with("refactor") {
        ChangeType::Refactor
    } else if lower.starts_with("chore") {
        ChangeType::Chore
    } else if lower.starts_with("docs") {
        ChangeType::Docs
    } else if lower.starts_with("test") {
        ChangeType::Test
    } else if lower.starts_with("perf") {
        ChangeType::Perf
    } else if lower.starts_with("style") {
        ChangeType::Style
    } else {
        ChangeType::Unknown
    };

    // Extract description (everything after the type and colon)
    let description = if let Some(colon_pos) = message.find(':') {
        message[colon_pos + 1..].trim().to_string()
    } else {
        message.to_string()
    };

    (change_type, description)
}
