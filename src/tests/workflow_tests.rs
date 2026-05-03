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
    let error = std::io::Error::other("inner error");
    let workflow_error = WorkflowError::Scan { source: error };

    workflow::handle_workflow_error(&workflow_error);
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

#[test]
fn test_workflow_index_content_readable_when_no_duplicates() {
    let root = "test_workflow_index_readable";
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).unwrap();

    let summary = workflow::execute(root).expect("workflow should succeed");
    let index = summary
        .index_content
        .expect("index file should have been readable");
    assert!(index.contains("Duplicate Files Comprehensive Index"));
    assert!(index.contains("Total duplicate groups found: 0"));

    super::cleanup_test_files(root).unwrap();
}

#[test]
fn workflow_error_display_includes_variant_context() {
    let scan = WorkflowError::Scan {
        source: std::io::Error::other("s"),
    };
    assert!(scan.to_string().contains("Error scanning files"), "{scan}");

    let detect = WorkflowError::Detect {
        source: std::io::Error::other("d"),
    };
    assert!(
        detect.to_string().contains("Error finding duplicates"),
        "{detect}"
    );

    let organize = WorkflowError::Organize {
        source: std::io::Error::other("o"),
    };
    assert!(
        organize.to_string().contains("Error organizing duplicates"),
        "{organize}"
    );

    let index_creation = WorkflowError::IndexCreation {
        source: std::io::Error::other("i"),
    };
    assert!(
        index_creation
            .to_string()
            .contains("Error creating comprehensive index"),
        "{index_creation}"
    );
}

#[test]
fn workflow_error_source_points_at_io_error() {
    let inner_msg = "nested-io";
    let inner = std::io::Error::other(inner_msg);
    let err = WorkflowError::Detect { source: inner };
    let source = std::error::Error::source(&err).expect("source");
    assert_eq!(format!("{}", source), inner_msg);
}
