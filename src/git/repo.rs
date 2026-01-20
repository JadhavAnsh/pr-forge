use crate::error::{PrForgeError, Result};
use git2::Repository;
use std::path::Path;

/// Repository wrapper
pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    /// Open a repository at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(&path).map_err(|e| {
            PrForgeError::RepoNotFound(format!("Failed to open repo at {:?}: {}", path.as_ref(), e))
        })?;

        Ok(GitRepository { repo })
    }

    /// Open repository in current directory
    #[allow(dead_code)]
    pub fn open_current() -> Result<Self> {
        let repo = Repository::discover(".").map_err(|e| {
            PrForgeError::RepoNotFound(format!(
                "No Git repository found in current directory: {}",
                e
            ))
        })?;

        Ok(GitRepository { repo })
    }

    /// Get internal repository reference
    pub fn inner(&self) -> &Repository {
        &self.repo
    }

    /// Get mutable internal repository reference
    #[allow(dead_code)]
    pub fn inner_mut(&mut self) -> &mut Repository {
        &mut self.repo
    }
}
