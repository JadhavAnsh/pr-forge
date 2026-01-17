use crate::pr::model::PRDescription;
use serde_json::json;

/// Format PR as JSON (for CI/automation)
pub fn format_json(pr: &PRDescription) -> crate::error::Result<String> {
    let json_value = json!({
        "summary": pr.summary,
        "key_changes": pr.key_changes,
        "files_touched": pr.files_touched,
        "commits_analyzed": pr.commits_analyzed,
        "impact": pr.impact,
        "risks_and_notes": pr.risks_and_notes,
        "checklist": pr.checklist.iter().map(|(item, checked)| {
            json!({"item": item, "checked": checked})
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&json_value)?)
}
