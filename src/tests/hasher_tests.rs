use super::cleanup_test_files;
use crate::hasher;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_hasher_compute_hash() -> Result<(), std::io::Error> {
    let temp_dir = "test_hash_dir";
    fs::create_dir_all(temp_dir)?;
    let test_file = PathBuf::from(temp_dir).join("hash_test.txt");
    let content = "test content for hashing";
    fs::write(&test_file, content)?;

    let hash = hasher::compute_file_hash(&test_file)?;

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_identical_files_have_same_hash() -> Result<(), std::io::Error> {
    let temp_dir = "test_identical_hash";
    fs::create_dir_all(temp_dir)?;

    let content = "identical content for hash test";
    let file1 = PathBuf::from(temp_dir).join("file1.txt");
    let file2 = PathBuf::from(temp_dir).join("file2.txt");

    fs::write(&file1, content)?;
    fs::write(&file2, content)?;

    let hash1 = hasher::compute_file_hash(&file1)?;
    let hash2 = hasher::compute_file_hash(&file2)?;

    assert_eq!(hash1, hash2);

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_different_files_have_different_hashes() -> Result<(), std::io::Error> {
    let temp_dir = "test_different_hash";
    fs::create_dir_all(temp_dir)?;

    let file1 = PathBuf::from(temp_dir).join("file1.txt");
    let file2 = PathBuf::from(temp_dir).join("file2.txt");

    fs::write(&file1, "content 1")?;
    fs::write(&file2, "content 2")?;

    let hash1 = hasher::compute_file_hash(&file1)?;
    let hash2 = hasher::compute_file_hash(&file2)?;

    assert_ne!(hash1, hash2);

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_empty_file_hash() -> Result<(), std::io::Error> {
    let temp_dir = "test_empty_hash";
    fs::create_dir_all(temp_dir)?;

    let empty_file = PathBuf::from(temp_dir).join("empty.txt");
    fs::write(&empty_file, "")?;

    let hash = hasher::compute_file_hash(&empty_file)?;

    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );

    cleanup_test_files(temp_dir)?;
    Ok(())
}
