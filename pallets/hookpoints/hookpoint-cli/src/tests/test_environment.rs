#[cfg(test)]
use std::fs;
use tempfile::tempdir;
use crate::environment::get_pallets;

#[test]
fn test_get_pallets() {
    // Create a temporary directory.
    let dir = tempdir().unwrap();

    // Create a "pallets" directory inside the temporary directory.
    let pallets_path = dir.path().join("pallets");
    fs::create_dir(&pallets_path).unwrap();

    // Create some directories and files inside the "pallets" directory.
    let pallet1 = pallets_path.join("pallet1");
    fs::create_dir(&pallet1).unwrap();

    let pallet2 = pallets_path.join("pallet2");
    fs::create_dir(&pallet2).unwrap();

    let file1 = pallets_path.join("file1.txt");
    fs::write(&file1, b"Some content").unwrap();

    // Call the function and check its output.
    let result = get_pallets(&Some(dir.path())).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains_key("pallet1"));
    assert!(result.contains_key("pallet2"));
    assert!(!result.contains_key("file1.txt"));
}

