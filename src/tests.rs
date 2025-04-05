#![cfg(test)]

use std::path::Path;

use crate::boot_json::process_file_path;

#[test]
fn test_process_file_path() {
  assert_eq!(process_file_path(Path::new("c:a/b/c/d"), Path::new("c:a/b")), Some("c/d".to_string()))
}
