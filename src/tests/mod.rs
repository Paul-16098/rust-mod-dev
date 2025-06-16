//! test

#![cfg(test)]

use std::path::Path;
use crate::cofg::Cofg;

#[test]
fn test_process_file_path() {
  use crate::boot_json::process_file_path;
  assert_eq!(
    process_file_path(Path::new("c:a/b/c/d"), Path::new("c:a/b")).ok().unwrap(),
    "c/d".to_string()
  );
}
#[test]
fn test_process_error_file_path() {
  use crate::boot_json::process_file_path;
  assert_eq!(
    process_file_path(Path::new("c:a/b/c/d"), Path::new("c:a/e")).err().unwrap().to_string(),
    "Failed to strip prefix: c:a/e from path: c:a/b/c/d".to_string()
  );
}

#[test]
fn test_boot_json_in_list() {
  use crate::boot_json::BootJson;
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

#[test]
fn test_scan_and_add_files() {
  use crate::boot_json::scan_and_add_files;
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

#[test]
fn test_update_file_lists() {
  use crate::boot_json::BootJson;
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
fn remove_test_file() {
  let _ = std::fs::remove_file("./cofg.json");
}

#[test]
fn test_new_from_json_str_valid() {
  remove_test_file();
  let json =
    r#"{
            "locale": "zh-cn",
            "loglv": "debug",
            "path": {
                "tmp_path": "./tmp_test",
                "results_path": "./results_test",
                "mods_path": "./mods_test"
            },
            "pause": false,
            "ts_process": false,
            "file_name": "test.mod.zip"
        }"#;
  let cofg = Cofg::new_from_json_str(json);
  assert_eq!(cofg.locale, "zh_cn");
  assert_eq!(cofg.loglv, "debug");
  assert_eq!(cofg.path.tmp_path, "./tmp_test");
  assert!(!cofg.pause);
  assert!(!cofg.ts_process);
  assert_eq!(cofg.file_name, "test.mod.zip");
  remove_test_file();
}

#[test]
fn test_new_from_json_str_invalid_locale_and_loglv() {
  remove_test_file();
  let json =
    r#"{
            "locale": "invalid_locale",
            "loglv": "invalid_log",
            "path": {
                "tmp_path": "./tmp_test2",
                "results_path": "./results_test2",
                "mods_path": "./mods_test2"
            },
            "pause": true,
            "ts_process": true,
            "file_name": "test2.mod.zip"
        }"#;
  let cofg = Cofg::new_from_json_str(json);
  assert_eq!(cofg.locale, "en"); // fallback
  assert_eq!(cofg.loglv, "info"); // fallback
  assert_eq!(cofg.path.tmp_path, "./tmp_test2");
  assert!(cofg.pause);
  assert!(cofg.ts_process);
  assert_eq!(cofg.file_name, "test2.mod.zip");
  remove_test_file();
}

#[test]
fn test_new_from_json_str_partial_json() {
  remove_test_file();
  let json = r#"{
            "locale": "en"
        }"#;
  let cofg = Cofg::new_from_json_str(json);
  assert_eq!(cofg.locale, "en");
  assert_eq!(cofg.loglv, "info"); // default
  assert_eq!(cofg.path.tmp_path, "./tmp"); // default
  assert!(cofg.pause); // default
  assert!(cofg.ts_process); // default
  assert_eq!(cofg.file_name, "{name}.mod.zip"); // default
  remove_test_file();
}

#[test]
fn test_new_from_json_str_invalid_json() {
  remove_test_file();
  let json = r#"not a json"#;
  let cofg = Cofg::new_from_json_str(json);
  // Should fallback to default
  assert_eq!(cofg.locale, "en");
  assert_eq!(cofg.loglv, "info");
  assert_eq!(cofg.path.tmp_path, "./tmp");
  remove_test_file();
}
