use crate::pr::model::{BranchAnalysis, RuleFinding, Severity};
use crate::rules::Rule;

/// Rule: Detects missing test changes for code modifications
pub struct TestsRule;

impl Rule for TestsRule {
    fn id(&self) -> &'static str {
        "tests_rule"
    }

    fn description(&self) -> &'static str {
        "Detects when code changes lack corresponding test changes"
    }

    fn evaluate(&self, analysis: &BranchAnalysis) -> Option<RuleFinding> {
        if !analysis.has_tests {
            // Check if there are actual code changes (non-test, non-config files)
            let has_code_changes = analysis.files_changed.iter().any(|f| {
                !f.contains("test")
                    && !f.contains("spec")
                    && !f.contains("__tests__")
                    && !f.ends_with(".md")
                    && !f.contains("config")
                    && !f.contains(".yml")
                    && !f.contains(".yaml")
            });

            if has_code_changes {
                return Some(RuleFinding {
                    rule_id: self.id().to_string(),
                    severity: Severity::Warning,
                    message: "⚠️  No test changes detected for code modifications.".to_string(),
                });
            }
        }
        None
    }
}

/// Rule: Detects potential breaking changes
pub struct BreakingChangeRule;

impl Rule for BreakingChangeRule {
    fn id(&self) -> &'static str {
        "breaking_change_rule"
    }

    fn description(&self) -> &'static str {
        "Detects signals of potential breaking changes"
    }

    fn evaluate(&self, analysis: &BranchAnalysis) -> Option<RuleFinding> {
        let has_breaking_signals = analysis.files_changed.iter().any(|f| {
            // Check for public API deletions
            f.contains("src/lib.rs")
                || f.contains("src/api")
                || f.contains("src/public")
                || (f.ends_with(".rs")
                    && analysis.commits.iter().any(|c| {
                        c.message.to_lowercase().contains("breaking")
                            || c.message.to_lowercase().contains("delete")
                            || c.message.to_lowercase().contains("remove public")
                    }))
        });

        if has_breaking_signals {
            return Some(RuleFinding {
                rule_id: self.id().to_string(),
                severity: Severity::Critical,
                message: "🚨 Potential breaking changes detected. Consider adding migration notes."
                    .to_string(),
            });
        }

        if analysis.possible_breaking_change {
            return Some(RuleFinding {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message: "⚠️  Review commits for potential breaking changes.".to_string(),
            });
        }

        None
    }
}

/// Rule: Detects configuration file changes
pub struct ConfigChangeRule;

impl Rule for ConfigChangeRule {
    fn id(&self) -> &'static str {
        "config_change_rule"
    }

    fn description(&self) -> &'static str {
        "Detects configuration file modifications"
    }

    fn evaluate(&self, analysis: &BranchAnalysis) -> Option<RuleFinding> {
        if analysis.has_config_changes {
            return Some(RuleFinding {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message:
                    "⚠️  Configuration files changed. Verify environment updates are documented."
                        .to_string(),
            });
        }

        // Also check for specific config file patterns
        let config_patterns = [
            ".env",
            ".yml",
            ".yaml",
            ".json",
            "docker-compose",
            "config/",
        ];
        let has_config_file = analysis
            .files_changed
            .iter()
            .any(|f| config_patterns.iter().any(|pattern| f.contains(pattern)));

        if has_config_file {
            return Some(RuleFinding {
                rule_id: self.id().to_string(),
                severity: Severity::Warning,
                message:
                    "⚠️  Configuration files changed. Verify environment updates are documented."
                        .to_string(),
            });
        }

        None
    }
}

/// Rule: Detects low-quality commit messages
pub struct CommitQualityRule;

impl Rule for CommitQualityRule {
    fn id(&self) -> &'static str {
        "commit_quality_rule"
    }

    fn description(&self) -> &'static str {
        "Detects non-descriptive commit messages"
    }

    fn evaluate(&self, analysis: &BranchAnalysis) -> Option<RuleFinding> {
        let low_quality_patterns = ["^fix$", "^update$", "^wip$", "^changes$", "^stuff$"];
        let mut poor_commits_count = 0;

        for commit in &analysis.commits {
            let msg = commit.message.trim().to_lowercase();
            if low_quality_patterns
                .iter()
                .any(|pattern| msg == *pattern || msg.starts_with(&format!("{} ", pattern)))
            {
                poor_commits_count += 1;
            }
        }

        if poor_commits_count > 0 {
            let count = poor_commits_count;
            return Some(RuleFinding {
                rule_id: self.id().to_string(),
                severity: Severity::Info,
                message: format!(
                    "ℹ️  {} commit message(s) are non-descriptive; PR summary compensates for this.",
                    count
                ),
            });
        }

        None
    }
}
