use super::cleanup_test_files;
use crate::file_scanner::{FileInfo, FileScanner};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_file_scanner_creation() {
    let scanner = FileScanner::new("/tmp");
    assert_eq!(scanner.root_path, PathBuf::from("/tmp"));
}

#[test]
fn test_file_info_creation() -> Result<(), std::io::Error> {
    let temp_dir = "test_temp_dir";
    fs::create_dir_all(temp_dir)?;
    let test_file = PathBuf::from(temp_dir).join("test.txt");
    fs::write(&test_file, "test content")?;

    let file_info = FileInfo::new(test_file.clone())?;
    assert_eq!(file_info.path, test_file);

    cleanup_test_files(temp_dir)?;
    Ok(())
}
