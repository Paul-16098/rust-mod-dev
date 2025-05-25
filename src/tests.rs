//! test

#![cfg(test)]

use std::path::Path;

use crate::boot_json::{ process_file_path, BootJson, scan_and_add_files };

#[test]
fn test_process_file_path() {
  assert_eq!(
    process_file_path(Path::new("c:a/b/c/d"), Path::new("c:a/b")).ok().unwrap(),
    "c/d".to_string()
  );
}
#[test]
fn test_process_error_file_path() {
  assert_eq!(
    process_file_path(Path::new("c:a/b/c/d"), Path::new("c:a/e")).err().unwrap().to_string(),
    "Failed to strip prefix: c:a/e from path: c:a/b/c/d".to_string()
  );
}

// 新增測試: BootJson::in_list
#[test]
fn test_boot_json_in_list() {
  let boot_json = BootJson {
    name: "testmod".to_string(),
    version: Some("1.0.0".to_string()),
    additionFile: Some(vec!["README.md".to_string()]),
    imgFileList: Some(vec!["img/a.png".to_string()]),
    scriptFileList: Some(vec!["main.js".to_string()]),
    styleFileList: Some(vec!["main.css".to_string()]),
    tweeFileList: Some(vec!["story.twee".to_string()]),
    addonPlugin: Some(vec![]),
    dependenceInfo: Some(vec![]),
  };
  assert!(boot_json.in_list("boot.json"));
  assert!(boot_json.in_list("README.md"));
  assert!(boot_json.in_list("img/a.png"));
  assert!(!boot_json.in_list("not_exist.txt"));
}

// 新增測試: scan_and_add_files
#[test]
fn test_scan_and_add_files() {
  use std::fs::{ create_dir_all, File };
  use tempfile::tempdir;

  let dir = tempdir().unwrap();
  let dir_path = dir.path();
  create_dir_all(dir_path.join("img")).unwrap();
  let img_file = dir_path.join("img/test.png");
  File::create(&img_file).unwrap();

  let mut img_files = vec![];
  let pattern = format!("{}/**/*.png", dir_path.display());
  scan_and_add_files(&pattern, &mut img_files, dir_path).unwrap();
  assert!(img_files.iter().any(|f| (f.ends_with("img/test.png") || f.ends_with("img\\test.png"))));
}

// 新增測試: BootJson::update_file_lists
#[test]
fn test_update_file_lists() {
  use std::fs::{ create_dir_all, File };
  use tempfile::tempdir;

  let dir = tempdir().unwrap();
  let dir_path = dir.path();
  create_dir_all(dir_path.join("img")).unwrap();
  File::create(dir_path.join("img/a.png")).unwrap();
  File::create(dir_path.join("README.md")).unwrap();
  let mut boot_json = BootJson {
    name: "testmod".to_string(),
    version: None,
    additionFile: None,
    imgFileList: None,
    scriptFileList: None,
    styleFileList: None,
    tweeFileList: None,
    addonPlugin: None,
    dependenceInfo: None,
  };
  boot_json.update_file_lists(dir_path).unwrap();
  assert!(
    boot_json.imgFileList
      .as_ref()
      .unwrap()
      .iter()
      .any(|f| (f.ends_with("img/a.png") || f.ends_with("img\\a.png")))
  );
  assert!(boot_json.additionFile.as_ref().unwrap().contains(&"README.md".to_string()));
}
