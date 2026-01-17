use crate::error::{PrForgeError, Result};
use git2::Repository;

/// Verify a branch exists
pub fn branch_exists(repo: &Repository, branch_name: &str) -> Result<bool> {
    let revspec = repo.revparse_single(branch_name);
    Ok(revspec.is_ok())
}

/// Get default branch name
pub fn get_default_branch(repo: &Repository) -> Result<String> {
    // Try to find default from HEAD
    let head = repo.head().map_err(|e| {
        PrForgeError::GitError(format!("Failed to read HEAD: {}", e))
    })?;

    if head.is_branch() {
        if let Some(branch_name) = head.shorthand() {
            return Ok(branch_name.to_string());
        }
    }

    // Fallback to common defaults
    for default_name in &["main", "master", "develop"] {
        if branch_exists(repo, default_name)? {
            return Ok(default_name.to_string());
        }
    }

    Err(PrForgeError::GitError(
        "Could not determine default branch".to_string(),
    ))
}
