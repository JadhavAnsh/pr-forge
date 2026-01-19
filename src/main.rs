mod ai;
mod analysis;
mod cli;
mod error;
mod git;
mod output;
mod pr;
mod rules;

use clap::Parser;
use cli::Args;
use error::Result;
use git::GitRepository;

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    // Open repository
    let repo_path = args.repo.as_deref().unwrap_or(".");
    let git_repo = GitRepository::open(repo_path)?;

    // Determine base branch
    let base_branch = if let Some(base) = &args.base {
        base.clone()
    } else {
        git::branch::get_default_branch(git_repo.inner())?
    };

    // Verify both branches exist
    if !git::branch::branch_exists(git_repo.inner(), &args.branch)? {
        return Err(error::PrForgeError::BranchNotFound(args.branch.clone()));
    }

    if !git::branch::branch_exists(git_repo.inner(), &base_branch)? {
        return Err(error::PrForgeError::BranchNotFound(base_branch.clone()));
    }

    // Get commits between branches
    let commits = git::commit::get_commits_between(
        git_repo.inner(),
        &base_branch,
        &args.branch,
    )?;

    if commits.is_empty() {
        println!("No commits found between {} and {}", base_branch, args.branch);
        return Ok(());
    }

    // Get files changed
    let files = git::commit::get_files_between(
        git_repo.inner(),
        &base_branch,
        &args.branch,
    )?;

    // Build PR description with optional AI analysis
    let enable_ai = !args.disable_ai;
    let pr = pr::build_pr_description_with_ai(
        &args.branch,
        &base_branch,
        commits,
        files,
        enable_ai,
    );

    // Format and print output
    let format = args.parse_format()?;
    let output = output::format_output(&pr, &format)?;

    println!("{}", output);

    Ok(())
}
