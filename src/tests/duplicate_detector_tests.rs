use super::{cleanup_test_files, create_test_files};
use crate::duplicate_detector::{DuplicateDetector, DuplicateGroup};
use crate::file_scanner::FileInfo;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_duplicate_group_operations() {
    let mut group = DuplicateGroup::new();
    let file1 = PathBuf::from("file1.txt");
    let file2 = PathBuf::from("file2.txt");

    assert_eq!(group.len(), 0);
    assert!(group.is_empty());

    group.add_file(file1.clone());
    assert_eq!(group.len(), 1);
    assert!(!group.is_empty());

    group.add_file(file2);
    assert_eq!(group.len(), 2);
    assert!(!group.is_empty());

    assert!(group.files.contains(&file1));
}

#[test]
fn test_duplicate_detector_no_duplicates() -> Result<(), std::io::Error> {
    let temp_dir = "test_no_dupes";
    fs::create_dir_all(temp_dir)?;

    let files = vec![
        ("unique1.txt", "content 1"),
        ("unique2.txt", "content 2"),
        ("unique3.txt", "content 3"),
    ];

    for (filename, content) in files {
        let path = PathBuf::from(temp_dir).join(filename);
        fs::write(&path, content)?;
    }

    let file_infos: Vec<FileInfo> = fs::read_dir(temp_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|entry| FileInfo::new(entry.path()).unwrap())
        .collect();

    let duplicates = DuplicateDetector::find_duplicates(file_infos)?;

    assert!(duplicates.is_empty());

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_duplicate_detector_with_duplicates() -> Result<(), std::io::Error> {
    let temp_dir = "test_with_dupes";
    let test_files = create_test_files(temp_dir)?;

    let file_infos: Vec<FileInfo> = test_files
        .into_iter()
        .map(|path| FileInfo::new(path).unwrap())
        .collect();

    let duplicates = DuplicateDetector::find_duplicates(file_infos)?;

    assert_eq!(duplicates.len(), 2);

    for (_hash, group) in duplicates {
        assert_eq!(group.len(), 2);
    }

    cleanup_test_files(temp_dir)?;
    Ok(())
}
