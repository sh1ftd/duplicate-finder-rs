use super::cleanup_test_files;
use crate::duplicate_detector::{DuplicateDetector, DuplicateGroup};
use crate::file_scanner::FileInfo;
use crate::organizer::{OrganizedGroup, Organizer};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_organizer_creation() {
    let organizer = Organizer::new("/tmp");
    assert_eq!(organizer.root_path, PathBuf::from("/tmp"));
}

#[test]
fn test_comprehensive_index_creation() -> Result<(), std::io::Error> {
    let temp_dir = "test_comprehensive_index";
    fs::create_dir_all(temp_dir)?;

    let organizer = Organizer::new(temp_dir);

    let hash1 = "hash1_1234567890abcdef".to_string();
    let hash2 = "hash2_0987654321fedcba".to_string();

    let group1 = DuplicateGroup::new();
    let group2 = DuplicateGroup::new();

    let folder1 = PathBuf::from(temp_dir)
        .join("duplicates")
        .join(format!("{hash1}_file1.txt"));
    let folder2 = PathBuf::from(temp_dir)
        .join("duplicates")
        .join(format!("{hash2}_file3.txt"));

    fs::create_dir_all(&folder1)?;
    fs::create_dir_all(&folder2)?;

    let organized_groups = vec![
        OrganizedGroup {
            hash: hash1.clone(),
            group: group1,
            folder: folder1.clone(),
            file_sizes: vec![0],
        },
        OrganizedGroup {
            hash: hash2.clone(),
            group: group2,
            folder: folder2.clone(),
            file_sizes: vec![0],
        },
    ];

    organizer.create_comprehensive_index(&organized_groups)?;

    let index_path = PathBuf::from(temp_dir).join("duplicate_files_index.txt");
    assert!(index_path.exists());

    let index_content = fs::read_to_string(&index_path)?;
    assert!(index_content.contains("Duplicate Files Comprehensive Index"));
    assert!(index_content.contains("Total duplicate groups found: 2"));
    assert!(index_content.contains(&hash1));
    assert!(index_content.contains(&hash2));

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_organizer_generates_unique_names_for_duplicate_files() -> Result<(), std::io::Error> {
    let temp_dir = "test_unique_duplicate_names";
    cleanup_test_files(temp_dir)?;
    fs::create_dir_all(temp_dir)?;

    let duplicate1 = PathBuf::from(temp_dir).join("a").join("duplicate.bin");
    let duplicate2 = PathBuf::from(temp_dir).join("b").join("duplicate.bin");
    let duplicate3 = PathBuf::from(temp_dir).join("duplicate.bin");

    fs::create_dir_all(duplicate1.parent().unwrap())?;
    fs::create_dir_all(duplicate2.parent().unwrap())?;

    let payload = b"identical";
    fs::write(&duplicate1, payload)?;
    fs::write(&duplicate2, payload)?;
    fs::write(&duplicate3, payload)?;

    let file_infos = vec![duplicate1.clone(), duplicate2.clone(), duplicate3.clone()]
        .into_iter()
        .map(FileInfo::new)
        .collect::<Result<Vec<_>, _>>()?;

    let duplicates = DuplicateDetector::find_duplicates(file_infos)?;
    assert_eq!(duplicates.len(), 1);

    let organizer = Organizer::new(temp_dir);
    let organized_groups = organizer.organize_duplicates(duplicates)?;
    assert_eq!(organized_groups.len(), 1);

    let destination = organized_groups[0].folder.clone();
    let mut moved_files = Vec::new();
    for entry in fs::read_dir(&destination)? {
        let entry = entry?;
        moved_files.push(entry.path());
    }

    moved_files.sort();
    assert_eq!(moved_files.len(), 3);
    assert!(
        moved_files
            .iter()
            .any(|path| path.file_name().unwrap().to_string_lossy() == "duplicate.bin")
    );
    assert!(moved_files.iter().any(|path| {
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("duplicate_copy")
    }));

    cleanup_test_files(temp_dir)?;
    Ok(())
}

#[test]
fn test_comprehensive_index_reports_freed_space() -> Result<(), std::io::Error> {
    let temp_dir = "test_freed_space_reporting";
    cleanup_test_files(temp_dir)?;
    fs::create_dir_all(temp_dir)?;

    let organizer = Organizer::new(temp_dir);

    let payload = vec![0u8; 1_048_576];
    let paths: Vec<PathBuf> = ["copy1.bin", "copy2.bin", "copy3.bin"]
        .iter()
        .map(|name| PathBuf::from(temp_dir).join(name))
        .collect();

    for path in &paths {
        fs::write(path, &payload)?;
    }

    let file_infos = paths
        .iter()
        .map(|path| FileInfo::new(path.clone()))
        .collect::<Result<Vec<_>, _>>()?;

    let duplicates = DuplicateDetector::find_duplicates(file_infos)?;
    assert_eq!(duplicates.len(), 1);

    let organized_groups = organizer.organize_duplicates(duplicates)?;
    organizer.create_comprehensive_index(&organized_groups)?;

    let index_path = PathBuf::from(temp_dir).join("duplicate_files_index.txt");
    assert!(index_path.exists());
    let index_content = fs::read_to_string(&index_path)?;
    assert!(index_content.contains("Space that can be freed: Approximately 2.00 MB (estimated)"));

    cleanup_test_files(temp_dir)?;
    Ok(())
}
