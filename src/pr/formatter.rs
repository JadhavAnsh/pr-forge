use crate::pr::model::PRDescription;

/// Format PR description as Markdown
pub fn format_markdown(pr: &PRDescription) -> String {
    let mut output = String::new();

    output.push_str("## Summary\n");
    output.push_str(&pr.summary);
    output.push_str("\n\n");

    // Add AI-generated changelog if available
    if let Some(changelog) = &pr.ai_changelog {
        output.push_str("## Changelog\n\n");
        
        for section in &changelog.sections {
            output.push_str(&format!("{} {}\n", section.emoji, section.category));
            
            for entry in &section.entries {
                output.push_str(&format!("**{}** - {}\n", entry.title, entry.description));
                
                if !entry.files.is_empty() {
                    for file in &entry.files {
                        output.push_str(&format!("  - `{}`\n", file));
                    }
                }
                output.push_str("\n");
            }
        }

        if !changelog.breaking_changes.is_empty() {
            output.push_str("### 🚨 Breaking Changes\n");
            for breaking_change in &changelog.breaking_changes {
                output.push_str(&format!("- {}\n", breaking_change));
            }
            output.push_str("\n");
        }

        output.push_str("\n");
    }

    output.push_str("## Key Changes\n");
    for change in &pr.key_changes {
        output.push_str(change);
        output.push_str("\n");
    }
    output.push_str("\n");

    if !pr.files_touched.is_empty() {
        output.push_str("## Files / Areas Touched\n");
        for file in &pr.files_touched {
            output.push_str(&format!("- {}\n", file));
        }
        output.push_str("\n");
    }

    output.push_str("## Commits Analyzed\n");
    for commit in &pr.commits_analyzed {
        output.push_str(commit);
        output.push_str("\n");
    }
    output.push_str("\n");

    output.push_str("## Impact\n");
    output.push_str(&pr.impact);
    output.push_str("\n\n");

    if !pr.risks_and_notes.is_empty() {
        output.push_str("## Risks & Notes\n");
        for risk in &pr.risks_and_notes {
            output.push_str("- ");
            output.push_str(risk);
            output.push_str("\n");
        }
        output.push_str("\n");
    }

    output.push_str("## Reviewer Checklist\n");
    for (item, checked) in &pr.checklist {
        let checkbox = if *checked { "✅" } else { "☐" };
        output.push_str(&format!("{} {}\n", checkbox, item));
    }

    output
}

/// Format PR description as plain text
pub fn format_plain(pr: &PRDescription) -> String {
    let mut output = String::new();

    output.push_str("SUMMARY\n");
    output.push_str("-------\n");
    output.push_str(&pr.summary);
    output.push_str("\n\n");

    output.push_str("KEY CHANGES\n");
    output.push_str("----------\n");
    for change in &pr.key_changes {
        output.push_str(change);
        output.push_str("\n");
    }
    output.push_str("\n");

    output.push_str("FILES TOUCHED\n");
    output.push_str("-------------\n");
    for file in &pr.files_touched {
        output.push_str(&format!("{}\n", file));
    }
    output.push_str("\n");

    output.push_str("IMPACT\n");
    output.push_str("------\n");
    output.push_str(&pr.impact);
    output.push_str("\n\n");

    output
}
