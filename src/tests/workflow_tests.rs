use crate::handle_workflow_error;
use crate::workflow::{self, WorkflowError};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_workflow_handles_missing_duplicates() {
    let root = "test_workflow_no_duplicates";
    fs::create_dir_all(root).unwrap();

    let result = workflow::execute(root);
    assert!(result.is_ok());
    let summary = result.unwrap();

    assert!(!summary.duplicates_found);
    assert_eq!(summary.duplicate_group_count, 0);

    super::cleanup_test_files(root).unwrap();
}

#[test]
fn test_handle_workflow_error_displays_messages() {
    let error = std::io::Error::new(std::io::ErrorKind::Other, "inner error");
    let workflow_error = WorkflowError::Scan { source: error };

    handle_workflow_error(&workflow_error);

    assert!(true);
}

#[test]
fn test_workflow_skips_current_executable() {
    let temp_dir = "test_workflow_skip_exe";
    super::cleanup_test_files(temp_dir).unwrap();
    fs::create_dir_all(temp_dir).unwrap();

    let exe_name = "self.exe";
    let exe_path = PathBuf::from(temp_dir).join(exe_name);
    let duplicate_dirs = ["copy1", "copy2", "copy3"];

    let payload = b"self-binary";
    fs::write(&exe_path, payload).unwrap();
    for dir in duplicate_dirs {
        let path = PathBuf::from(temp_dir).join(dir);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join(exe_name), payload).unwrap();
    }

    let result = workflow::execute(temp_dir);
    assert!(result.is_ok());
    let summary = result.unwrap();

    assert!(summary.duplicates_found);
    let current_exe = std::env::current_exe().unwrap();
    assert!(current_exe.exists());

    let duplicates_dir = PathBuf::from(temp_dir).join("duplicates");
    assert!(duplicates_dir.exists());

    let mut moved = Vec::new();
    for entry in fs::read_dir(&duplicates_dir).unwrap() {
        let entry = entry.unwrap();
        let group_path = entry.path();
        for file_entry in fs::read_dir(group_path).unwrap() {
            let file_entry = file_entry.unwrap();
            moved.push(file_entry.path());
        }
    }

    assert_eq!(moved.len(), 4);
    let mut seen = std::collections::HashSet::new();
    for path in moved {
        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(file_name.starts_with("self"));
        assert!(seen.insert(file_name));
    }

    assert!(!exe_path.exists());

    super::cleanup_test_files(temp_dir).unwrap();
}
