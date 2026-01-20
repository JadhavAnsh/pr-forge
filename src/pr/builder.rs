use crate::ai::GroqClient;
use crate::analysis::change_classifier::{classify_changes, generate_summary};
use crate::analysis::commit_analyzer::analyze_commits;
use crate::analysis::file_analyzer::analyze_files;
use crate::pr::model::{BranchAnalysis, ChangeType, ClassifiedCommit, PRDescription};
use crate::rules::ruleset::create_default_ruleset;
use log::{debug, info, warn};

/// Build a complete PR description from branch analysis with optional AI enhancement
#[allow(dead_code)]
pub fn build_pr_description(
    branch_name: &str,
    base_branch: &str,
    commits: Vec<crate::pr::model::CommitInfo>,
    files: Vec<String>,
) -> PRDescription {
    build_pr_description_with_ai(branch_name, base_branch, commits, files, true)
}

/// Build PR description with optional AI analysis
pub fn build_pr_description_with_ai(
    branch_name: &str,
    base_branch: &str,
    commits: Vec<crate::pr::model::CommitInfo>,
    files: Vec<String>,
    enable_ai: bool,
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
        findings: vec![],                // Will be populated by rules
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

    // Try to enhance with Groq AI analysis if enabled
    let (ai_changelog, ai_enabled, ai_error) = if enable_ai {
        match generate_ai_changelog(&classified, &files, branch_name, base_branch) {
            Ok(changelog) => {
                info!("Successfully generated AI-enhanced changelog");
                (Some(changelog), Some(true), None)
            }
            Err(e) => {
                warn!(
                    "Failed to generate AI changelog, falling back to rule-based analysis: {}",
                    e
                );
                let error_msg = format!("{}", e);
                (None, Some(true), Some(error_msg)) // AI was enabled but failed
            }
        }
    } else {
        debug!("AI analysis disabled");
        (None, Some(false), None)
    };

    PRDescription {
        summary,
        key_changes,
        files_touched,
        commits_analyzed,
        impact,
        risks_and_notes,
        checklist,
        ai_changelog,
        ai_enabled,
        ai_error,
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
            .or_default()
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
        if !matches!(
            change_type,
            ChangeType::Feature | ChangeType::Fix | ChangeType::Refactor
        ) {
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
    let mut parts: Vec<String> =
        vec!["This PR improves code quality and maintainability.".to_string()];

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
        (
            "Code changes are logical and well-organized".to_string(),
            false,
        ),
        (
            "Tests are present and cover new functionality".to_string(),
            analysis.has_tests,
        ),
        ("Documentation is updated if needed".to_string(), false),
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

/// Generate AI-enhanced changelog using Groq API
fn generate_ai_changelog(
    classified_commits: &[ClassifiedCommit],
    files: &[String],
    branch_name: &str,
    base_branch: &str,
) -> crate::error::Result<crate::ai::AiGeneratedChangelog> {
    // Create Groq client with default config
    let config = crate::ai::GroqConfig::default();
    let client = GroqClient::new(config)?;

    // Prepare context for Groq
    let commits_summary = classified_commits
        .iter()
        .map(|c| format!("{}: {}", c.change_type, c.description))
        .collect::<Vec<_>>()
        .join("\n");

    let files_summary = if files.len() > 50 {
        format!("{} files changed (truncated)", files.len())
    } else {
        files.join(", ")
    };

    let system_prompt = r#"You are an expert software engineer analyzing pull request changes. 
Your task is to generate a professional, well-organized changelog categorizing all changes.

Respond ONLY with valid JSON in this exact format, no markdown code blocks:
{
  "sections": [
    {
      "category": "UI/UX Improvements",
      "emoji": "🎨",
      "entries": [
        {
          "title": "Component Name",
          "description": "Detailed description of what changed",
          "files": ["path/to/file1", "path/to/file2"]
        }
      ]
    }
  ],
  "summary": "Brief overall summary",
  "breaking_changes": ["List of breaking changes if any"]
}"#;

    let user_message = format!(
        r#"Analyze this PR from {} to {} and categorize the changes:

Commits:
{}

Files Changed:
{}

Categories to use:
- 🎨 UI/UX Improvements
- 🔧 Build & Configuration Fixes
- 🐛 Bug Fixes
- 📦 Dependencies
- 🚀 Performance Improvements
- 📝 Documentation
- 🧪 Testing
- ⚙️ Refactoring
- 🎯 Breaking Changes

Please provide a comprehensive changelog with all changes properly categorized."#,
        base_branch, branch_name, commits_summary, files_summary
    );

    let response = client.chat_completion(system_prompt.to_string(), user_message)?;

    debug!("Groq response: {}", response);

    // Parse JSON response
    let changelog =
        serde_json::from_str::<crate::ai::AiGeneratedChangelog>(&response).map_err(|e| {
            warn!("Failed to parse Groq response as JSON: {}", e);
            crate::error::PrForgeError::ApiParseError(format!(
                "Invalid changelog JSON from Groq: {}",
                e
            ))
        })?;

    Ok(changelog)
}
