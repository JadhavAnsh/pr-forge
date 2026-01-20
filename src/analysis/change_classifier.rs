use crate::pr::model::ChangeType;
use std::collections::HashMap;

/// Statistics about changes grouped by type
pub struct ChangeStats {
    pub by_type: HashMap<ChangeType, Vec<String>>,
    #[allow(dead_code)]
    pub total: usize,
}

/// Classify and group changes
pub fn classify_changes(commits: &[crate::pr::model::ClassifiedCommit]) -> ChangeStats {
    let mut by_type: HashMap<ChangeType, Vec<String>> = HashMap::new();

    for commit in commits {
        let entry = by_type.entry(commit.change_type.clone()).or_default();
        entry.push(commit.description.clone());
    }

    let total = commits.len();

    ChangeStats { by_type, total }
}

/// Generate change summary text
pub fn generate_summary(stats: &ChangeStats) -> String {
    let mut parts = Vec::new();

    if let Some(features) = stats.by_type.get(&ChangeType::Feature) {
        parts.push(format!(
            "Added {} new feature{}",
            features.len(),
            if features.len() == 1 { "" } else { "s" }
        ));
    }

    if let Some(fixes) = stats.by_type.get(&ChangeType::Fix) {
        parts.push(format!(
            "fixed {} issue{}",
            fixes.len(),
            if fixes.len() == 1 { "" } else { "s" }
        ));
    }

    if let Some(_refactors) = stats.by_type.get(&ChangeType::Refactor) {
        parts.push("refactored code for maintainability".to_string());
    }

    if parts.is_empty() {
        return "Updated codebase".to_string();
    }

    // Capitalize first part
    if let Some(first) = parts.first_mut() {
        if let Some(c) = first.chars().next() {
            *first = c.to_uppercase().to_string() + &first[1..];
        }
    }

    parts.join(", with ")
}
