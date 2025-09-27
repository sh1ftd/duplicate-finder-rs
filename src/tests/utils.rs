use std::fs;
use std::path::PathBuf;

pub fn create_test_files(test_dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    fs::create_dir_all(test_dir)?;

    let files = vec![
        ("file1.txt", "test content 1"),
        ("file2.txt", "test content 1"),
        ("file3.txt", "test content 2"),
        ("file4.txt", "test content 2"),
        ("unique.txt", "unique content"),
    ];

    let mut paths = Vec::new();
    for (filename, content) in files {
        let path = PathBuf::from(test_dir).join(filename);
        fs::write(&path, content)?;
        paths.push(path);
    }

    Ok(paths)
}

pub fn cleanup_test_files(test_dir: &str) -> Result<(), std::io::Error> {
    if fs::metadata(test_dir).is_ok() {
        fs::remove_dir_all(test_dir)?;
    }
    Ok(())
}
