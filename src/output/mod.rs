pub mod json;
pub mod markdown;
pub mod plain;

use crate::error::Result;
use crate::pr::model::PRDescription;

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Markdown,
    Plain,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "plain" | "txt" => Ok(OutputFormat::Plain),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Format PR description according to selected format
pub fn format_output(pr: &PRDescription, format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Markdown => Ok(markdown::format_markdown(pr)),
        OutputFormat::Plain => Ok(plain::format_plain(pr)),
        OutputFormat::Json => json::format_json(pr),
    }
}
