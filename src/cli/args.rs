use clap::Parser;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "pr-forge",
    about = "Generate senior-grade PR descriptions from Git branches",
    version = "0.1.0",
    long_about = "A CLI tool that analyzes a branch's commits and generates a senior-grade Pull Request description and changelog, filling gaps, enforcing standards, and improving review quality."
)]
pub struct Args {
    /// The branch to analyze
    #[arg(value_name = "BRANCH")]
    pub branch: String,

    /// Base branch to compare against (default: main/master/develop)
    #[arg(short, long, value_name = "BRANCH")]
    pub base: Option<String>,

    /// Output format
    #[arg(short, long, value_name = "FORMAT", default_value = "markdown")]
    pub format: String,

    /// Generate shorter PR description
    #[arg(short, long)]
    pub short: bool,

    /// Repository path (default: current directory)
    #[arg(short, long, value_name = "PATH")]
    pub repo: Option<String>,
}

impl Args {
    /// Parse output format from string
    pub fn parse_format(&self) -> crate::error::Result<crate::output::OutputFormat> {
        crate::output::OutputFormat::from_str(&self.format)
            .map_err(|e| crate::error::PrForgeError::ConfigError(e))
    }
}
