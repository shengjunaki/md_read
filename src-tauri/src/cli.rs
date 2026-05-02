use std::{env, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Only .md and .markdown files are supported.")]
    UnsupportedExtension,
}

pub fn startup_markdown_path() -> Result<Option<PathBuf>, CliError> {
    let Some(raw) = env::args_os().nth(1) else {
        return Ok(None);
    };

    let path = PathBuf::from(raw);
    let is_markdown = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "md" | "markdown"))
        .unwrap_or(false);

    if !is_markdown {
        return Err(CliError::UnsupportedExtension);
    }

    Ok(Some(path))
}
