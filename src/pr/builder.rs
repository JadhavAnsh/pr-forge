use crate::analysis::change_classifier::{classify_changes, generate_summary};
use crate::analysis::commit_analyzer::analyze_commits;
use crate::analysis::file_analyzer::analyze_files;
use crate::pr::model::{BranchAnalysis, ClassifiedCommit, ChangeType, PRDescription};
use crate::rules::ruleset::create_default_ruleset;

/// Build a complete PR description from branch analysis
pub fn build_pr_description(
    branch_name: &str,
    base_branch: &str,
    commits: Vec<crate::pr::model::CommitInfo>,
    files: Vec<String>,
) -> PRDescription {
    // Classify commits
    let classified = analyze_commits(commits.clone());

    // Analyze files
    let file_analysis = analyze_files(&files);
    let (has_tests, has_configs) = crate::analysis::file_analyzer::categorize_files(&file_analysis);

    // Classify changes
    let change_stats = classify_changes(&classified);

    // Build initial analysis
    let analysis = BranchAnalysis {
        branch_name: branch_name.to_string(),
        base_branch: base_branch.to_string(),
        commits: commits.clone(),
        files_changed: files.clone(),
        has_tests: has_tests > 0,
        has_config_changes: has_configs > 0,
        possible_breaking_change: false, // TODO: detect from diffs
        findings: vec![], // Will be populated by rules
    };

    // Evaluate rules
    let ruleset = create_default_ruleset();
    let findings = ruleset.evaluate(&analysis);

    // Generate sections
    let summary = generate_summary(&change_stats);
    let key_changes = generate_key_changes(&classified);
    let impact = generate_impact(&change_stats);
    let risks_and_notes = generate_risks(&findings);
    let checklist = generate_checklist(&analysis);

    let files_touched = files
        .iter()
        .take(10) // Limit to first 10 files in PR
        .cloned()
        .collect();

    let commits_analyzed = commits
        .iter()
        .map(|c| format!("- {}: {}", c.hash, c.message))
        .collect();

    PRDescription {
        summary,
        key_changes,
        files_touched,
        commits_analyzed,
        impact,
        risks_and_notes,
        checklist,
    }
}

/// Generate key changes section
fn generate_key_changes(commits: &[ClassifiedCommit]) -> Vec<String> {
    let mut changes = Vec::new();

    // Group by type
    let mut by_type: std::collections::HashMap<ChangeType, Vec<&ClassifiedCommit>> =
        std::collections::HashMap::new();

    for commit in commits {
        by_type
            .entry(commit.change_type.clone())
            .or_insert_with(Vec::new)
            .push(commit);
    }

    // Features
    if let Some(features) = by_type.get(&ChangeType::Feature) {
        for feature in features {
            changes.push(format!("• {}", feature.description));
        }
    }

    // Fixes
    if let Some(fixes) = by_type.get(&ChangeType::Fix) {
        for fix in fixes {
            changes.push(format!("• Fixed: {}", fix.description));
        }
    }

    // Refactors
    if let Some(refactors) = by_type.get(&ChangeType::Refactor) {
        for refactor in refactors {
            changes.push(format!("• Refactored: {}", refactor.description));
        }
    }

    // Others
    for (change_type, items) in &by_type {
        if !matches!(change_type, ChangeType::Feature | ChangeType::Fix | ChangeType::Refactor) {
            for item in items {
                changes.push(format!("• {}", item.description));
            }
        }
    }

    if changes.is_empty() {
        changes.push("• Code improvements and updates".to_string());
    }

    changes
}

/// Generate impact section
fn generate_impact(change_stats: &crate::analysis::change_classifier::ChangeStats) -> String {
    let mut parts: Vec<String> = vec![
        "This PR improves code quality and maintainability.".to_string(),
    ];
    
    if change_stats.by_type.contains_key(&ChangeType::Feature) {
        parts.push("New functionality is available for end users.".to_string());
    }
    
    if change_stats.by_type.contains_key(&ChangeType::Fix) {
        parts.push("Bug fixes enhance reliability.".to_string());
    }

    parts.join(" ")
}

/// Generate risks from findings
fn generate_risks(findings: &[crate::pr::model::RuleFinding]) -> Vec<String> {
    findings.iter().map(|f| f.message.clone()).collect()
}

/// Generate reviewer checklist
fn generate_checklist(analysis: &BranchAnalysis) -> Vec<(String, bool)> {
    vec![
        ("Code changes are logical and well-organized".to_string(), false),
        (
            "Tests are present and cover new functionality".to_string(),
            analysis.has_tests,
        ),
        (
            "Documentation is updated if needed".to_string(),
            false,
        ),
        (
            "No breaking changes introduced".to_string(),
            !analysis.possible_breaking_change,
        ),
        (
            "Configuration changes are documented".to_string(),
            !analysis.has_config_changes,
        ),
    ]
}
