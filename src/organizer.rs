use crate::duplicate_detector::DuplicateGroup;
use crate::hasher::Hash;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Organizer {
    pub root_path: PathBuf,
    skip_paths: HashSet<PathBuf>,
}

#[derive(Debug)]
pub struct OrganizedGroup {
    pub hash: Hash,
    pub group: DuplicateGroup,
    pub folder: PathBuf,
    pub file_sizes: Vec<u64>,
}

impl Organizer {
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        Organizer {
            root_path: root_path.as_ref().to_path_buf(),
            skip_paths: HashSet::new(),
        }
    }

    pub fn with_skip_paths<P: AsRef<Path>, I>(root_path: P, skip_paths: I) -> Self
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut organizer = Organizer::new(root_path);
        organizer.set_skip_paths(skip_paths);
        organizer
    }

    fn set_skip_paths<I>(&mut self, skip_paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        self.skip_paths = skip_paths
            .into_iter()
            .map(|path| fs::canonicalize(&path).unwrap_or(path))
            .collect();
    }

    pub fn organize_duplicates(
        &self,
        duplicates: HashMap<Hash, DuplicateGroup>,
    ) -> Result<Vec<OrganizedGroup>, std::io::Error> {
        let mut organized_groups = Vec::new();

        for (hash, group) in duplicates {
            if let Some(original_file) = group.files.first() {
                let original_filename = original_file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                let folder_name = format!("{hash}_{original_filename}");
                let duplicate_folder = self.root_path.join("duplicates").join(folder_name);

                // Create the duplicate folder
                fs::create_dir_all(&duplicate_folder)?;

                let mut file_sizes = Vec::new();

                // Move all files in the group to the duplicate folder
                for file_path in &group.files {
                    let size = fs::metadata(file_path)?.len();
                    file_sizes.push(size);

                    if self.should_skip(file_path) {
                        continue;
                    }

                    let file_name = file_path.file_name().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid file name")
                    })?;

                    let new_path = self.build_unique_destination(&duplicate_folder, file_name);
                    fs::rename(file_path, &new_path)?;
                }

                // Store information for comprehensive index
                organized_groups.push(OrganizedGroup {
                    hash: hash.clone(),
                    group,
                    folder: duplicate_folder,
                    file_sizes,
                });
            }
        }

        Ok(organized_groups)
    }

    pub fn create_comprehensive_index(
        &self,
        organized_groups: &[OrganizedGroup],
    ) -> Result<(), std::io::Error> {
        let index_path = self.root_path.join("duplicate_files_index.txt");

        let mut index_content = String::new();
        index_content.push_str("Duplicate Files Comprehensive Index\n");
        index_content.push_str("===================================\n\n");
        index_content.push_str(&format!(
            "Total duplicate groups found: {}\n",
            organized_groups.len()
        ));
        index_content.push_str(&format!(
            "Index created: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        index_content.push_str(&format!(
            "Scanned directory: {}\n\n",
            self.root_path.display()
        ));

        let total_files = organized_groups
            .iter()
            .map(|group| group.group.files.len())
            .sum::<usize>();

        let freed_bytes = organized_groups
            .iter()
            .map(|group| {
                let total: u64 = group.file_sizes.iter().copied().sum();
                let keep = group.file_sizes.iter().copied().min().unwrap_or(0);
                total.saturating_sub(keep)
            })
            .sum::<u64>();

        let freed_mb = freed_bytes as f64 / (1024.0 * 1024.0);

        index_content.push_str(&format!("Total files in duplicate groups: {total_files}\n"));
        index_content.push_str(&format!(
            "Space that can be freed: Approximately {freed_mb:.2} MB (estimated)\n\n"
        ));

        index_content.push_str("Duplicate Groups:\n");
        index_content.push_str("================\n\n");

        for (i, organized_group) in organized_groups.iter().enumerate() {
            let hash = &organized_group.hash;
            let group = &organized_group.group;
            let folder_path = &organized_group.folder;
            index_content.push_str(&format!("Group {}:\n", i + 1));
            index_content.push_str(&format!("  Hash: {hash}\n"));
            index_content.push_str(&format!("  Folder: {}\n", folder_path.display()));
            index_content.push_str(&format!("  Files in group: {}\n", group.files.len()));
            index_content.push_str("  File paths:\n");

            for file_path in &group.files {
                index_content.push_str(&format!("    - {}\n", file_path.display()));
            }

            index_content.push('\n');
        }

        fs::write(index_path, index_content)?;
        Ok(())
    }

    fn should_skip(&self, path: &Path) -> bool {
        if self.skip_paths.is_empty() {
            return false;
        }

        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        self.skip_paths.contains(&canonical)
    }

    fn build_unique_destination(
        &self,
        directory: &Path,
        original_name: &std::ffi::OsStr,
    ) -> PathBuf {
        let original_path = Path::new(original_name);
        let mut candidate = directory.join(original_path);
        if !candidate.exists() {
            return candidate;
        }

        let stem = original_path
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| Cow::from("file"));
        let extension = original_path.extension().map(|ext| ext.to_string_lossy());

        let mut index = 1;
        loop {
            let new_name = match &extension {
                Some(ext) if !ext.is_empty() => format!("{stem}_copy{index}.{ext}"),
                _ => format!("{stem}_copy{index}"),
            };

            candidate = directory.join(new_name);
            if !candidate.exists() {
                break candidate;
            }

            index += 1;
        }
    }
}
