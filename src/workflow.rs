use crate::duplicate_detector::DuplicateDetector;
use crate::file_scanner::FileScanner;
use crate::organizer::Organizer;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WorkflowSummary {
    pub files_scanned: usize,
    pub duplicate_group_count: usize,
    pub duplicates_found: bool,
    pub index_path: PathBuf,
    pub index_content: Option<String>,
    pub index_read_error: Option<io::Error>,
}

#[derive(Debug)]
pub enum WorkflowError {
    Scan { source: io::Error },
    Detect { source: io::Error },
    Organize { source: io::Error },
    IndexCreation { source: io::Error },
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowError::Scan { source } => write!(f, "Error scanning files: {source}"),
            WorkflowError::Detect { source } => write!(f, "Error finding duplicates: {source}"),
            WorkflowError::Organize { source } => {
                write!(f, "Error organizing duplicates: {source}")
            }
            WorkflowError::IndexCreation { source } => {
                write!(f, "Error creating comprehensive index: {source}")
            }
        }
    }
}

impl std::error::Error for WorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WorkflowError::Scan { source }
            | WorkflowError::Detect { source }
            | WorkflowError::Organize { source }
            | WorkflowError::IndexCreation { source } => Some(source),
        }
    }
}

pub fn execute(root_path: &str) -> Result<WorkflowSummary, WorkflowError> {
    let scanner = FileScanner::new(root_path);
    let files = scanner
        .scan_files()
        .map_err(|source| WorkflowError::Scan { source })?;
    let files_scanned = files.len();

    let duplicates = DuplicateDetector::find_duplicates(files)
        .map_err(|source| WorkflowError::Detect { source })?;

    let duplicate_group_count = duplicates.len();
    let organizer = Organizer::new(root_path);

    if duplicate_group_count == 0 {
        organizer
            .create_comprehensive_index(&[])
            .map_err(|source| WorkflowError::IndexCreation { source })?;

        let index_path = build_index_path(root_path);
        let (index_content, index_read_error) = read_index(&index_path);

        return Ok(WorkflowSummary {
            files_scanned,
            duplicate_group_count,
            duplicates_found: false,
            index_path,
            index_content,
            index_read_error,
        });
    }

    let skip_paths = build_skip_paths();
    let organizer = Organizer::with_skip_paths(root_path, skip_paths);

    let organized_groups = organizer
        .organize_duplicates(duplicates)
        .map_err(|source| WorkflowError::Organize { source })?;

    organizer
        .create_comprehensive_index(&organized_groups)
        .map_err(|source| WorkflowError::IndexCreation { source })?;

    let index_path = build_index_path(root_path);
    let (index_content, index_read_error) = read_index(&index_path);

    Ok(WorkflowSummary {
        files_scanned,
        duplicate_group_count: organized_groups.len(),
        duplicates_found: true,
        index_path,
        index_content,
        index_read_error,
    })
}

fn build_index_path(root_path: &str) -> PathBuf {
    Path::new(root_path).join("duplicate_files_index.txt")
}

fn read_index(index_path: &Path) -> (Option<String>, Option<io::Error>) {
    match fs::read_to_string(index_path) {
        Ok(content) => (Some(content), None),
        Err(error) => (None, Some(error)),
    }
}

fn build_skip_paths() -> Vec<PathBuf> {
    if let Ok(current_exe) = std::env::current_exe() {
        let canonical = fs::canonicalize(&current_exe).unwrap_or(current_exe);
        vec![canonical]
    } else {
        Vec::new()
    }
}
