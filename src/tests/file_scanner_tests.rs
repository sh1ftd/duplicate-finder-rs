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
fn test_file_scanner_finds_nested_files() -> Result<(), std::io::Error> {
    let temp_dir = "test_nested_scan_dir";
    cleanup_test_files(temp_dir)?;
    let nested = PathBuf::from(temp_dir).join("a").join("b");
    fs::create_dir_all(&nested)?;
    let root_txt = PathBuf::from(temp_dir).join("root.txt");
    let nested_txt = nested.join("leaf.txt");
    fs::write(&root_txt, "root")?;
    fs::write(&nested_txt, "leaf")?;

    let scanner = FileScanner::new(temp_dir);
    let files = scanner.scan_files()?;

    assert_eq!(files.len(), 2);

    cleanup_test_files(temp_dir)?;
    Ok(())
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
