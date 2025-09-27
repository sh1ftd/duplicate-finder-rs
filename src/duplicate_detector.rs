use crate::file_scanner::FileInfo;
use crate::hasher::{Hash, compute_file_hash};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DuplicateGroup {
    pub files: Vec<PathBuf>,
}

impl Default for DuplicateGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateGroup {
    pub fn new() -> Self {
        DuplicateGroup { files: Vec::new() }
    }

    pub fn add_file(&mut self, file_path: PathBuf) {
        self.files.push(file_path);
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

pub struct DuplicateDetector;

impl DuplicateDetector {
    pub fn find_duplicates(
        files: Vec<FileInfo>,
    ) -> Result<HashMap<Hash, DuplicateGroup>, std::io::Error> {
        let mut file_hashes: HashMap<Hash, DuplicateGroup> = HashMap::new();

        for file_info in files {
            let hash = compute_file_hash(&file_info.path)?;

            file_hashes
                .entry(hash)
                .or_default()
                .add_file(file_info.path);
        }

        // Keep only groups with multiple files (actual duplicates)
        // Single-file groups are filtered out but files remain untouched in their original locations
        file_hashes.retain(|_, group| !group.is_empty() && group.len() > 1);

        Ok(file_hashes)
    }
}
