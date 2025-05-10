//! 文件系統操作相關的輔助函數

use super::{ fs, Path, error, t };

/// 遞迴複製目錄
/// # 參數
/// * `src` - 源目錄路徑
/// * `dst` - 目標目錄路徑
pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
  if !dst.exists() {
    fs::create_dir_all(dst)?;
  }

  for entry in fs::read_dir(src)? {
    let entry = entry?;
    let src_path = entry.path();
    let dst_path = dst.join(entry.file_name());

    if entry.file_type()?.is_dir() {
      if entry.file_name() == ".git" {
        continue;
      }
      copy_dir_all(&src_path, &dst_path)?;
    } else {
      fs::copy(&src_path, &dst_path)?;
    }
  }
  Ok(())
}

/// 檢查目錄是否為空
///
/// # 參數
/// * `path` - 要檢查的目錄路徑
///
/// # 返回值
/// * `bool` - true表示目錄為空，false表示非空
pub(crate) fn check_empty_dirs(path: &std::path::PathBuf) -> bool {
  if !path.is_dir() {
    return false;
  }

  match fs::read_dir(path) {
    Ok(entries) => {
      let has_valid_entries = entries.flatten().any(|entry| {
        let entry_path = entry.path();
        if entry_path.is_dir() {
          !check_empty_dirs(&entry_path)
        } else {
          true
        }
      });
      !has_valid_entries
    }
    Err(e) => {
      error!("{}", t!("filesystem.read_dir_failed", path = path.display(), e = e));
      false
    }
  }
}
