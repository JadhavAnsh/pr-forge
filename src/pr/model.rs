use serde::{Deserialize, Serialize};

/// Core commit information extracted from Git
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub files_changed: Vec<String>,
    pub author: String,
}

/// Severity level for rule findings
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "Info"),
            Severity::Warning => write!(f, "Warning"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

/// Finding from rule evaluation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleFinding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
}

/// Complete analysis of a branch
#[derive(Debug)]
pub struct BranchAnalysis {
    #[allow(dead_code)]
    pub branch_name: String,
    #[allow(dead_code)]
    pub base_branch: String,
    pub commits: Vec<CommitInfo>,
    pub files_changed: Vec<String>,
    pub has_tests: bool,
    pub has_config_changes: bool,
    pub possible_breaking_change: bool,
    #[allow(dead_code)]
    pub findings: Vec<RuleFinding>,
}

/// Change type classification
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChangeType {
    Feature,
    Fix,
    Refactor,
    Chore,
    Docs,
    Test,
    Perf,
    Style,
    Unknown,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Feature => write!(f, "feat"),
            ChangeType::Fix => write!(f, "fix"),
            ChangeType::Refactor => write!(f, "refactor"),
            ChangeType::Chore => write!(f, "chore"),
            ChangeType::Docs => write!(f, "docs"),
            ChangeType::Test => write!(f, "test"),
            ChangeType::Perf => write!(f, "perf"),
            ChangeType::Style => write!(f, "style"),
            ChangeType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Classified commit with extracted intent
#[derive(Debug, Clone)]
pub struct ClassifiedCommit {
    pub info: CommitInfo,
    pub change_type: ChangeType,
    pub description: String,
}

/// File change information
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub is_test: bool,
    pub is_config: bool,
    pub is_api: bool,
    pub is_deleted: bool,
}

/// Complete PR description output
#[derive(Debug, Serialize, Deserialize)]
pub struct PRDescription {
    pub summary: String,
    pub key_changes: Vec<String>,
    pub files_touched: Vec<String>,
    pub commits_analyzed: Vec<String>,
    pub impact: String,
    pub risks_and_notes: Vec<String>,
    pub checklist: Vec<(String, bool)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_changelog: Option<crate::ai::AiGeneratedChangelog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_error: Option<String>,
}
