use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct MarkdownSource {
    pub path: PathBuf,
    pub base_dir: PathBuf,
    pub content: String,
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("The file does not exist: {0}")]
    Missing(PathBuf),
    #[error("The path is not a file: {0}")]
    NotFile(PathBuf),
    #[error("Unable to read the file: {0}")]
    Read(#[from] std::io::Error),
    #[error("The file is too large for the MVP reader. Maximum supported size is 10 MB.")]
    TooLarge,
}

const MAX_MARKDOWN_BYTES: u64 = 10 * 1024 * 1024;

pub fn read_markdown(path: &Path) -> Result<MarkdownSource, DocumentError> {
    let path = path
        .canonicalize()
        .map_err(|_| DocumentError::Missing(path.to_path_buf()))?;

    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err(DocumentError::NotFile(path));
    }
    if metadata.len() > MAX_MARKDOWN_BYTES {
        return Err(DocumentError::TooLarge);
    }

    let content = fs::read_to_string(&path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

    Ok(MarkdownSource {
        path,
        base_dir,
        content,
    })
}
