/// mod boot_json

use glob::glob;
use log::trace;
use serde::{ Deserialize, Serialize };
use nest_struct::nest_struct;

/// boot.json的主要數據結構
/// 包含mod的所有元數據和資源文件列表
#[nest_struct]
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct BootJson {
  /// mod的唯一標識名稱
  pub name: String,
  /// mod版本號
  version: Option<String>,
  /// 額外文件列表(如README, License等)
  additionFile: Option<Vec<String>>,
  /// 圖片資源文件列表
  imgFileList: Option<Vec<String>>,
  /// JavaScript腳本文件列表
  scriptFileList: Option<Vec<String>>,
  /// Twee故事腳本文件列表
  tweeFileList: Option<Vec<String>>,
  /// CSS樣式文件列表
  styleFileList: Option<Vec<String>>,
  /// 插件配置列表
  addonPlugin: Option<Vec<nest! {
    /// 目標mod名稱
    modName: String,
    /// 插件名稱
    addonName: String,
    /// 目標mod版本
    modVersion: String,
    /// 插件參數列表
    params: Vec<ParamEntry! {
      passage: String,
      findString: String,
      replace: String,
    }>,
  }>>,
  /// mod依賴信息列表
  dependenceInfo: Option<Vec<nest! {
    /// 被依賴的mod名稱
    modName: String,
    /// 被依賴的mod版本要求
    version: String,
  }>>,
}

/// BootJson結構體的方法實現
impl BootJson {
  /// 從文件路徑創建BootJson實例
  /// * `path` - boot.json文件的路徑
  /// # 示例
  /// ```rust
  /// let boot_json = BootJson::new("path/to/boot.json")?;
  /// ```
  /// # 錯誤處理
  /// - 返回錯誤如果文件不存在或格式錯誤
  pub fn new(path: &str) -> Result<BootJson, Box<dyn std::error::Error>> {
    let file_content = std::fs::read(path).map_err(|e| format!("無法讀取boot.json文件: {}", e))?;

    let mut json: BootJson = serde_json
      ::from_slice(&file_content)
      .map_err(|e| format!("解析boot.json失敗: {}", e))?;

    // 初始化所有Option字段
    // json.name = Some(json.name.unwrap_or_else(|| "unknown".to_string()));
    json.version = Some(json.version.unwrap_or_else(|| "1.0.0".to_string()));
    json.additionFile = Some(json.additionFile.unwrap_or_default());
    json.imgFileList = Some(json.imgFileList.unwrap_or_default());
    json.scriptFileList = Some(json.scriptFileList.unwrap_or_default());
    json.styleFileList = Some(json.styleFileList.unwrap_or_default());
    json.tweeFileList = Some(json.tweeFileList.unwrap_or_default());

    Ok(json)
  }

  /// 更新文件列表
  /// * `cwd` - 當前工作目錄路徑
  /// * 返回 Result
  ///
  /// 該函數會掃描工作目錄下的所有相關文件並更新到對應的文件列表中
  pub fn update_file_lists(
    &mut self,
    cwd: &std::path::Path
  ) -> Result<(), Box<dyn std::error::Error>> {
    let show_cwd = cwd.display();

    // 確保所有列表已初始化
    let addition_files = self.additionFile.get_or_insert_with(Vec::new);
    let img_files = self.imgFileList.get_or_insert_with(Vec::new);
    let script_files = self.scriptFileList.get_or_insert_with(Vec::new);
    let style_files = self.styleFileList.get_or_insert_with(Vec::new);
    let twee_files = self.tweeFileList.get_or_insert_with(Vec::new);
    self.addonPlugin.get_or_insert_with(Vec::new);
    self.dependenceInfo.get_or_insert_with(Vec::new);

    // 處理附加文件
    for file in ["README.md", "README.txt", "License.txt", "License"].iter() {
      let file_path = format!("{}/{}", show_cwd, file);
      if std::path::Path::new(&file_path).exists() && !addition_files.contains(&file.to_string()) {
        addition_files.push(file.to_string());
      }
    }

    // 處理各類型文件
    scan_and_add_files(&format!("{}/**/*.png", show_cwd), img_files, cwd)?;
    scan_and_add_files(&format!("{}/**/*.js", show_cwd), script_files, cwd)?;
    scan_and_add_files(&format!("{}/**/*.css", show_cwd), style_files, cwd)?;
    scan_and_add_files(&format!("{}/**/*.twee", show_cwd), twee_files, cwd)?;
    scan_and_add_files(&format!("{}/**/*.js.map", show_cwd), addition_files, cwd)?;

    Ok(())
  }

  /// 檢查文件是否在任何列表中
  /// # 參數
  /// * `value` - 要檢查的文件路徑
  /// # 返回
  /// * `bool` - 文件是否存在於任何列表中
  pub fn in_list(&self, value: &str) -> bool {
    // boot.json 總是包含在內
    if value == "boot.json" {
      return true;
    }

    let normalized_path = value.replace("\\", "/");
    let lists = [
      &self.imgFileList,
      &self.scriptFileList,
      &self.tweeFileList,
      &self.styleFileList,
      &self.additionFile,
    ];

    trace!("檢查路徑: {}", normalized_path);
    lists.iter().any(|list| list.as_ref().unwrap().contains(&normalized_path))
  }
}

/// 掃描並添加特定類型的文件到文件列表中
/// * `pattern` - 文件匹配模式
/// * `file_list` - 文件列表
/// * `cwd` - 當前工作目錄
pub fn scan_and_add_files(
  pattern: &str,
  file_list: &mut Vec<String>,
  cwd: &std::path::Path
) -> Result<(), Box<dyn std::error::Error>> {
  for path in glob(pattern)?.flatten() {
    if let Some(rel_path) = process_file_path(&path, cwd) {
      if !file_list.contains(&rel_path) {
        file_list.push(rel_path.replace("\\", "/"));
      }
    }
  }
  Ok(())
}

/// 處理文件路徑，將絕對路徑轉換為相對路徑
/// * `path` - 要處理的文件路徑
/// * `cwd` - 當前工作目錄
pub fn process_file_path(path: &std::path::Path, cwd: &std::path::Path) -> Option<String> {
  path
    .strip_prefix(cwd)
    .ok()?
    .to_str()
    .map(|s| s.to_string())
}
