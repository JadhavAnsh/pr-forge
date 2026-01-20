use crate::error::{PrForgeError, Result};
use crate::pr::model::CommitInfo;
use git2::{Oid, Repository};

/// Get all commits between two references
pub fn get_commits_between(repo: &Repository, base: &str, head: &str) -> Result<Vec<CommitInfo>> {
    // Resolve references
    let base_oid = resolve_reference(repo, base)?;
    let head_oid = resolve_reference(repo, head)?;

    // Get revwalk
    let mut revwalk = repo.revwalk().map_err(|e| {
        PrForgeError::CommitAnalysisError(format!("Failed to create revwalk: {}", e))
    })?;

    // Only walk commits reachable from head but not from base
    revwalk.push(head_oid).map_err(|e| {
        PrForgeError::CommitAnalysisError(format!("Failed to push head commit: {}", e))
    })?;

    revwalk.hide(base_oid).map_err(|e| {
        PrForgeError::CommitAnalysisError(format!("Failed to hide base commit: {}", e))
    })?;

    // Collect commits
    let mut commits = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result.map_err(|e| {
            PrForgeError::CommitAnalysisError(format!("Failed to get commit OID: {}", e))
        })?;

        let commit = repo.find_commit(oid).map_err(|e| {
            PrForgeError::CommitAnalysisError(format!("Failed to find commit: {}", e))
        })?;

        let message = commit.message().unwrap_or("(empty message)").to_string();

        let author = commit.author().name().unwrap_or("Unknown").to_string();

        // Get files changed in this commit
        let files = get_files_in_commit(repo, &commit)?;

        commits.push(CommitInfo {
            hash: oid.to_string()[..8].to_string(), // Short hash
            message: message.lines().next().unwrap_or("").to_string(), // First line only
            files_changed: files,
            author,
        });
    }

    // Reverse to get chronological order
    commits.reverse();

    Ok(commits)
}

/// Resolve a git reference to OID
fn resolve_reference(repo: &Repository, reference: &str) -> Result<Oid> {
    // Try to find as branch
    let revspec = repo.revparse_single(reference).map_err(|e| {
        PrForgeError::BranchNotFound(format!(
            "Failed to resolve reference '{}': {}",
            reference, e
        ))
    })?;

    Ok(revspec.id())
}

/// Get all files changed in a commit
fn get_files_in_commit(repo: &Repository, commit: &git2::Commit) -> Result<Vec<String>> {
    let mut files = Vec::new();

    let tree = commit.tree().map_err(|e| {
        PrForgeError::CommitAnalysisError(format!("Failed to get commit tree: {}", e))
    })?;

    let parent_tree = if commit.parent_count() > 0 {
        commit.parent(0).ok().and_then(|p| p.tree().ok())
    } else {
        None
    };

    let diff = if let Some(parent_tree) = parent_tree {
        repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
    } else {
        repo.diff_tree_to_tree(None, Some(&tree), None)
    }
    .map_err(|e| PrForgeError::CommitAnalysisError(format!("Failed to compute diff: {}", e)))?;

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                files.push(path.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )
    .map_err(|e| PrForgeError::CommitAnalysisError(format!("Failed to iterate diff: {}", e)))?;

    Ok(files)
}

/// Get all files changed between two references
pub fn get_files_between(repo: &Repository, base: &str, head: &str) -> Result<Vec<String>> {
    let base_oid = resolve_reference(repo, base)?;
    let head_oid = resolve_reference(repo, head)?;

    let base_tree = repo.find_commit(base_oid).ok().and_then(|c| c.tree().ok());

    let head_tree = repo
        .find_commit(head_oid)
        .map_err(|e| {
            PrForgeError::CommitAnalysisError(format!("Failed to find head commit: {}", e))
        })?
        .tree()
        .map_err(|e| {
            PrForgeError::CommitAnalysisError(format!("Failed to get head tree: {}", e))
        })?;

    let diff = repo
        .diff_tree_to_tree(base_tree.as_ref(), Some(&head_tree), None)
        .map_err(|e| PrForgeError::CommitAnalysisError(format!("Failed to compute diff: {}", e)))?;

    let mut files = Vec::new();

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                files.push(path.to_string_lossy().to_string());
            } else if let Some(path) = delta.old_file().path() {
                files.push(path.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    )
    .map_err(|e| PrForgeError::CommitAnalysisError(format!("Failed to iterate diff: {}", e)))?;

    // Deduplicate
    files.sort();
    files.dedup();

    Ok(files)
}
