//! test

#![cfg(test)]

use std::path::Path;

use crate::boot_json::process_file_path;

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
