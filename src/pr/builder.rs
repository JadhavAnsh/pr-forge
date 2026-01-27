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

    // Prepare commit context for Groq
    let commits_summary = classified_commits
        .iter()
        .map(|c| format!("{}: {}", c.change_type, c.description))
        .collect::<Vec<_>>()
        .join("\n");

    // Analyze files into high-level categories so the AI can describe
    // how frontend, backend, APIs, docs, tests, and configs are affected.
    let analyzed_files = analyze_files(files);

    let mut frontend_files: Vec<String> = Vec::new();
    let mut api_files: Vec<String> = Vec::new();
    let mut backend_files: Vec<String> = Vec::new();
    let mut docs_files: Vec<String> = Vec::new();
    let mut test_files: Vec<String> = Vec::new();
    let mut config_files: Vec<String> = Vec::new();
    let mut other_files: Vec<String> = Vec::new();

    for file in &analyzed_files {
        let path = &file.path;
        let mut classified = false;

        if is_docs_file(path) {
            docs_files.push(path.clone());
            classified = true;
        }
        if file.is_test {
            test_files.push(path.clone());
            classified = true;
        }
        if file.is_config {
            config_files.push(path.clone());
            classified = true;
        }
        if file.is_api {
            api_files.push(path.clone());
            classified = true;
        }
        if is_frontend_file(path) {
            frontend_files.push(path.clone());
            classified = true;
        }
        if is_backend_file(path) {
            backend_files.push(path.clone());
            classified = true;
        }

        if !classified {
            other_files.push(path.clone());
        }
    }

    let summarize_category = |name: &str, files: &Vec<String>, max: usize| -> String {
        if files.is_empty() {
            format!("- {}: none detected", name)
        } else if files.len() > max {
            format!(
                "- {}: {} files changed (showing first {}):\n  - {}",
                name,
                files.len(),
                max,
                files
                    .iter()
                    .take(max)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n  - ")
            )
        } else {
            format!(
                "- {} ({}):\n  - {}",
                name,
                files.len(),
                files.join("\n  - ")
            )
        }
    };

    let file_categories_overview = format!(
        "File categories detected:\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        summarize_category("Frontend / UI", &frontend_files, 30),
        summarize_category("Backend / Core", &backend_files, 30),
        summarize_category("API / Routes / Handlers", &api_files, 30),
        summarize_category("Documentation", &docs_files, 30),
        summarize_category("Tests", &test_files, 30),
        summarize_category("Config / Infra", &config_files, 30),
        summarize_category("Other", &other_files, 30),
    );

    // High-level commit type breakdown to help the AI
    let mut feature_commits = 0usize;
    let mut fix_commits = 0usize;
    let mut refactor_commits = 0usize;
    let mut docs_commits = 0usize;
    let mut test_commits = 0usize;
    let mut perf_commits = 0usize;
    let mut style_commits = 0usize;
    let mut other_commits = 0usize;

    for c in classified_commits {
        match c.change_type {
            ChangeType::Feature => feature_commits += 1,
            ChangeType::Fix => fix_commits += 1,
            ChangeType::Refactor => refactor_commits += 1,
            ChangeType::Docs => docs_commits += 1,
            ChangeType::Test => test_commits += 1,
            ChangeType::Perf => perf_commits += 1,
            ChangeType::Style => style_commits += 1,
            ChangeType::Chore | ChangeType::Unknown => other_commits += 1,
        }
    }

    let commit_type_overview = format!(
        "Commit type summary:\n- Features: {}\n- Fixes: {}\n- Refactors: {}\n- Docs: {}\n- Tests: {}\n- Perf: {}\n- Style: {}\n- Other/Chore/Unknown: {}",
        feature_commits,
        fix_commits,
        refactor_commits,
        docs_commits,
        test_commits,
        perf_commits,
        style_commits,
        other_commits
    );

    let system_prompt = r#"You are an expert software engineer analyzing pull request changes.
Your task is to generate a professional, well-organized changelog categorizing all changes.

Your analysis must adapt to the types of files and commits involved:
- When frontend or UI-related files are present (e.g. React/Next/Vue components, JS/TS/TSX/JSX files under `src/components` or `src/pages`, CSS/SCSS, HTML templates, design system files), clearly describe user-facing behavior, visual changes, layout adjustments, UX flows, and any accessibility or responsiveness implications.
- When backend or core server code changes, explain how business logic, domain behavior, data processing, or background jobs are affected, including potential performance or reliability implications.
- When API/route/handler files change, identify which endpoints or resources are impacted, what new capabilities are added, whether any request/response shapes changed, and how this might break or upgrade existing consumers.
- When documentation files change, describe which areas of documentation were updated and how they align with the underlying code changes (new features, breaking changes, migration notes, etc.).
- When tests are added or updated, summarize the new coverage or scenarios being tested and how they protect the changed behavior.
- When configuration or infrastructure files change (e.g. env, Docker, CI, TOML/YAML/JSON configs), call out deployment, environment, or operational impacts.

If the branch introduces an entirely new resource, module, endpoint, or major feature area (for example new API routes, new top-level modules, or new UI screens/components), explicitly treat this as a **new feature** and call it out as such, not just as a refactor.

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

{}

{}

Categories to use:
- 🎨 UI/UX Improvements
- 🧱 Backend & Core Logic
- 🌐 API & Integrations
- 🔧 Build & Configuration Fixes
- 🐛 Bug Fixes
- 📦 Dependencies
- 🚀 Performance Improvements
- 📝 Documentation
- 🧪 Testing
- ⚙️ Refactoring
- 🎯 Breaking Changes

Please provide a comprehensive changelog with all changes properly categorized."#,
        base_branch, branch_name, commits_summary, commit_type_overview, file_categories_overview
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

/// Heuristic to detect likely frontend/UI files from a path.
/// This is intentionally conservative: it tries to capture common
/// web/frontend stacks (React/Vue/Next/etc.) without being tied
/// to any one framework.
fn is_frontend_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();

    // Common component / page / UI directories
    let ui_dirs = [
        "src/components",
        "src/component",
        "src/pages",
        "src/views",
        "src/ui",
        "src/frontend",
        "frontend/",
        "web/",
        "client/",
    ];

    let is_ui_dir = ui_dirs.iter().any(|dir| lower.contains(dir));

    // Common frontend file extensions
    let ui_exts = [
        ".tsx", ".jsx", ".vue", ".svelte", ".astro", ".html", ".hbs", ".handlebars", ".twig",
        ".css", ".scss", ".sass", ".less", ".styl", ".stylus",
    ];

    let is_ui_ext = ui_exts.iter().any(|ext| lower.ends_with(ext));

    // JS/TS files that live under obvious UI dirs
    let is_js_ts_ui = (lower.ends_with(".ts") || lower.ends_with(".js"))
        && (lower.contains("src/components")
            || lower.contains("src/pages")
            || lower.contains("src/ui")
            || lower.contains("src/frontend")
            || lower.contains("/ui/")
            || lower.contains("/components/"));

    is_ui_dir || is_ui_ext || is_js_ts_ui
}

/// Heuristic to detect documentation files (Markdown, docs directories, etc.)
fn is_docs_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md")
        || lower.ends_with(".rst")
        || lower.contains("docs/")
        || lower.starts_with("docs/")
        || lower.contains("/docs/")
}

/// Heuristic to detect backend/core code files.
/// This is kept broad so that we can talk about how server-side or
/// core business logic is impacted, without being tied to a specific
/// language or framework.
fn is_backend_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();

    // Likely backend directories
    let backend_dirs = [
        "src/backend",
        "backend/",
        "server/",
        "services/",
        "src/services",
        "src/domain",
        "src/core",
    ];

    let in_backend_dir = backend_dirs.iter().any(|dir| lower.contains(dir));

    // Common backend/server-side extensions (excluding obvious UI ones)
    let backend_exts = [".rs", ".go", ".py", ".java", ".kt", ".kts", ".cs", ".rb"];
    let is_backend_ext = backend_exts.iter().any(|ext| lower.ends_with(ext));

    // Treat generic source files under src/ as backend/core when they haven't
    // already been classified as frontend or docs/config/test (handled earlier).
    let is_generic_src = lower.starts_with("src/") || lower.contains("/src/");

    in_backend_dir || is_backend_ext || is_generic_src
}
