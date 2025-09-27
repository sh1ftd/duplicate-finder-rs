use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct FileInfo {
    pub path: PathBuf,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        let _metadata = fs::metadata(&path)?;

        Ok(FileInfo { path })
    }
}

pub struct FileScanner {
    pub root_path: PathBuf,
}

impl FileScanner {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        FileScanner {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub fn scan_files(&self) -> Result<Vec<FileInfo>, std::io::Error> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let file_info = FileInfo::new(entry.path().to_path_buf())?;
                files.push(file_info);
            }
        }

        Ok(files)
    }
}
