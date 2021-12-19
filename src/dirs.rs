use crate::settings::SETTINGS;
use glob::{glob, GlobError};
use std::{error::Error, fmt::Display, io, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DirItem {
    pub path: String,
    pub display: String,
    pub readme: Option<String>,
}

fn get_readme(path: PathBuf) -> Result<Option<String>, io::Error> {
    for glob_pattern in SETTINGS.preview_files.iter() {
        let mut preview_file_pattern = path.clone();
        preview_file_pattern.push(glob_pattern);

        let preview_file_pattern = preview_file_pattern
            .to_str()
            .expect("Failed to expand preview file path");

        let matched_preview_file = glob(preview_file_pattern)
            .expect("Failed to expand preview file globbing pattern")
            .flatten()
            .next();

        if let Some(file) = matched_preview_file {
            return Ok(Some(
                file.to_str()
                    .expect("Failed to expand preview file path")
                    .to_string(),
            ));
        }
    }

    Ok(None)
}

#[derive(Debug)]
pub enum GetDirsError {
    IoError(io::Error),
    GlobError(GlobError),
}

impl Error for GetDirsError {}

impl From<io::Error> for GetDirsError {
    fn from(e: io::Error) -> Self {
        GetDirsError::IoError(e)
    }
}

impl From<GlobError> for GetDirsError {
    fn from(e: GlobError) -> Self {
        GetDirsError::GlobError(e)
    }
}

impl Display for GetDirsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetDirsError::IoError(e) => writeln!(f, "I/O Error: {}", e),
            GetDirsError::GlobError(e) => writeln!(f, "Error expanding globbing pattern: {}", e),
        }
    }
}

pub fn get_dirs() -> Result<Vec<DirItem>, GetDirsError> {
    let mut items = Vec::new();
    for dir in SETTINGS.search_dirs.iter() {
        let dir = shellexpand::tilde(dir);
        for child in glob(&dir).expect("Failed to resolve globbing pattern") {
            let path = child?;
            if path.is_dir() {
                items.push(DirItem {
                    path: path.to_str().unwrap().to_string(),
                    display: path
                        .file_name()
                        .expect("Failed to trim directory name")
                        .to_str()
                        .unwrap()
                        .to_string(),
                    readme: get_readme(path)?,
                });
            }
        }
    }

    items.sort_unstable_by_key(|item| item.display.to_string());

    Ok(items)
}
