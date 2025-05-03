//! main

mod tests;
// 引入模塊和依賴
pub mod boot_json;
use boot_json::BootJson;
use clap::{ Parser, ArgAction };
use config::Config;
use glob::glob;
use log::{ debug, error, info, trace, warn };
use rust_i18n::t;
use serde::{ Deserialize, Serialize };
use zip_extensions::ZipWriterExtensions;
use std::fs::{ self, remove_dir_all, remove_file, File };
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;
use nest_struct::nest_struct;

// 設定i18n
rust_i18n::i18n!();

const VERSION: &str = env!("VERSION");

/// 配置相關結構體和實現
#[nest_struct]
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Cofg {
  /// 程序使用的語言環境(zh_cn/zh_tw/en)
  locale: String,
  /// 日誌級別(warn/info/debug/trace)
  loglv: String,
  /// 路徑相關配置
  path: PathCofg! {
    /// 臨時文件存放路徑
    tmp_path: String,
    /// 輸出結果存放路徑
    results_path: String,
    /// mod源文件路徑
    mods_path: String,
  },
  /// 最後暫停?
  pause: bool,
  /// 處理 ts 文件?
  ts_process: bool,
  /// file name
  file_name: String,
}

impl Cofg {
  /// 配置初始化函數
  /// * 讀取並解析cofg.json文件
  fn new() -> Cofg {
    let settings = Config::builder()
      .add_source(config::File::with_name("./cofg.json"))
      .build()
      .unwrap();
    let mut cofg: Cofg = settings.try_deserialize().unwrap_or_default();

    cofg.locale = cofg.normalize_locale();
    cofg.loglv = cofg.validate_log_level().unwrap_or(cofg.loglv);
    cofg.write_file();
    cofg
  }

  /// 正規化語言環境
  fn normalize_locale(&self) -> String {
    match self.locale.to_lowercase().as_str() {
      "zh_cn" | "zh-cn" | "cn" | "zh" => "zh_cn".to_string(),
      "zh_tw" | "zh-tw" | "tw" => "zh_tw".to_string(),
      "en" | "en_us" | "en-us" => "en".to_string(),
      o => {
        warn!("{}", t!("config.invalid_locale", msg = o));
        "en".to_string()
      }
    }
  }

  /// 驗證日誌級別
  fn validate_log_level(&self) -> Option<String> {
    match self.loglv.as_str() {
      "warn" | "info" | "debug" | "trace" => None,
      o => {
        warn!("{}", t!("config.invalid_log_level", msg = o));
        Some("info".to_string())
      }
    }
  }

  /// form cli load args
  fn load_cli(mut self, cli: Cli) {
    if let Some(v) = cli.locale {
      self.locale = v;
    }
    if let Some(v) = cli.loglv {
      self.loglv = v;
    }
    self.pause = cli.pause;
    self.ts_process = cli.ts_process;
  }

  /// Returns the write file of this [`Cofg`].
  fn write_file(&self) {
    match serde_json::to_string_pretty(self) {
      Ok(json_string) => {
        if let Err(e) = std::fs::write("./cofg.json", json_string) {
          warn!("{}", t!("filesystem.write_file_failed", path = "cofg.json", e = e));
        }
      }
      Err(e) => {
        warn!("{}", t!("json.serialize_error", msg = e.to_string()));
      }
    }
  }

  /// 初始化路徑和日誌系統
  /// * 設置程序語言環境
  /// * 初始化日誌系統
  fn init(&self) {
    self.clone().load_cli(Cli::parse());

    for path in [&self.path.tmp_path, &self.path.results_path].iter() {
      let path_obj = std::path::Path::new(path);
      if path_obj.exists() {
        remove_dir_all(path_obj).unwrap();
      }
      fs::create_dir(path_obj).unwrap();
    }
    if !std::path::Path::new(&self.path.mods_path).exists() {
      fs::create_dir(&self.path.mods_path).unwrap();
    }

    rust_i18n::set_locale(&self.locale);
    let mut colog_cofg = colog::default_builder();
    match self.loglv.as_str() {
      "warn" => {
        colog_cofg.filter_level(log::LevelFilter::Warn);
      }
      "info" => {
        colog_cofg.filter_level(log::LevelFilter::Info);
      }
      "debug" => {
        colog_cofg.filter_level(log::LevelFilter::Debug);
      }
      "trace" => {
        colog_cofg.filter_level(log::LevelFilter::Trace);
      }
      o => warn!("{}", t!("config.invalid_log_level", msg = o)),
    }
    colog_cofg.init();
  }
}

impl Default for Cofg {
  fn default() -> Self {
    Cofg {
      locale: "en".to_string(),
      loglv: "info".to_string(),
      path: PathCofg {
        tmp_path: "./tmp".to_string(),
        results_path: "./results".to_string(),
        mods_path: "./mods".to_string(),
      },
      pause: true,
      ts_process: true,
      file_name: "{name}.mod.zip".to_string(),
    }
  }
}
/*
impl std::fmt::Display for PathCofg {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    serde_json
      ::to_value(self)
      .unwrap()
      .as_object()
      .unwrap()
      .iter()
      .try_for_each(|(k, v)| { writeln!(f, "    {k}: {v}") })
  }
}

impl std::fmt::Display for Cofg {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    serde_json
      ::to_value(self)
      .unwrap()
      .as_object()
      .unwrap()
      .iter()
      .try_for_each(|(k, v)| {
        if k == "path" {
          writeln!(f, "{k}:")?;
          serde_json::from_value::<PathCofg>(v.clone()).unwrap().fmt(f)
        } else {
          writeln!(f, "{k}: {v}")
        }
      })
  }
}
*/

#[derive(Parser, Debug, Serialize)]
#[clap(about = "a tool for mod dev", version = VERSION, after_help = env!("CARGO_PKG_REPOSITORY"))]
/// 命令行參數結構體
struct Cli {
  /// 語言環境
  #[clap(long, short = 'i')]
  locale: Option<String>,
  /// 日誌級別
  #[clap(long, short)]
  loglv: Option<String>,
  /// 是否處理ts文件
  #[clap(long = "tsp", action = ArgAction::SetTrue)]
  ts_process: bool,
  /// 是否暫停
  #[clap(short, long, action = ArgAction::SetTrue)]
  pause: bool,
}
/*
impl std::fmt::Display for Cli {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    serde_json
      ::to_value(self)
      .unwrap()
      .as_object()
      .unwrap()
      .iter()
      .try_for_each(|(k, v)| { writeln!(f, "{k}: {v}") })
  }
}
*/

/// 檢查目錄是否為空
///
/// # 參數
/// * `path` - 要檢查的目錄路徑
///
/// # 返回值
/// * `bool` - true表示目錄為空，false表示非空
fn check_empty_dirs(path: &std::path::PathBuf) -> bool {
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

/// 主要處理 TypeScript 文件的函數
fn process_ts_files(cofg: &Cofg) {
  info!("### {} ###", t!("ts.start"));
  for entry in glob(&format!("{}/*/", cofg.path.tmp_path)).expect(&t!("filesystem.glob_failed")) {
    match entry {
      Err(_) => (),
      Ok(path) => {
        let mut has_ts_file = false;
        for entry in glob(&format!("{}/**/*.ts", path.display())).expect(
          &t!("filesystem.glob_failed")
        ) {
          match entry {
            Err(_) => (),
            Ok(ts_path) => {
              if ts_path.ends_with(".d.ts") {
                continue;
              }
              has_ts_file = true;
            }
          }
        }
        if has_ts_file {
          // 調用 tsc 編譯 TypeScript 文件
          let mut tsc_command = if cfg!(windows) {
            std::process::Command::new("tsc.cmd")
          } else {
            std::process::Command::new("tsc")
          };
          let output = tsc_command
            .arg("--project")
            .arg(path.to_str().unwrap())
            .output()
            .expect(&t!("ts.tsc_failed"));

          if !output.status.success() {
            error!(
              "    {}",
              t!(
                "ts.tsc_error",
                msg = String::from_utf8_lossy(&output.stdout),
                path = path.display()
              )
            );
          } else {
            info!("    {}", t!("ts.tsc_success", path = path.display()));
          }
        }
      }
    }
  }
  info!("=== {} ===", t!("ts.end"));
}

/// 主要處理 boot.json 文件的函數
/// 掃描、解析和更新所有mod文件夾中的boot.json文件
fn process_boot_json_files(cofg: &Cofg) {
  info!("### {} ###", t!("boot_json.start"));

  for entry in glob(&format!("{}/*/boot.json", cofg.path.tmp_path)).expect(
    &t!("filesystem.glob_failed")
  ) {
    match entry {
      Ok(path) => {
        let cwd = path.parent().unwrap();
        info!("    {}", t!("boot_json.processing", path = path.display()));

        match BootJson::new(path.to_str().unwrap()) {
          Ok(mut boot_json) => {
            if let Err(e) = boot_json.update_file_lists(cwd) {
              warn!("{}", t!("filesystem.update_failed", msg = format!("{:?}", e)));
              continue;
            }

            // 保存更新後的boot.json
            match serde_json::to_string_pretty(&boot_json) {
              Ok(json_string) => {
                if let Err(e) = std::fs::write(&path, json_string) {
                  warn!("{}", t!("filesystem.write_file_failed", path = path.display(), e = e));
                }
              }
              Err(e) => {
                warn!("{}", t!("json.serialize_error", msg = format!("{}", e)));
              }
            }
          }
          Err(e) => warn!("{}", t!("filesystem.read_file_failed", path = path.display(), e = e)),
        }
      }
      Err(e) => warn!("{}", t!("boot_json.read_error", e = format!("{:?}", e))),
    }
  }
  info!("=== {} ===", t!("boot_json.end"));
}

/// 壓縮所有的mod文件夾成zip格式
/// 將處理完的mod打包成最終發布格式
fn compress_mod_folders(cofg: &Cofg) {
  info!("### {} ###", t!("compress.start"));

  let results_dir = Path::new(&cofg.path.results_path);

  for entry in glob(&format!("{}/*/", cofg.path.tmp_path))
    .expect("Failed to read glob pattern")
    .flatten() {
    let src_dir = entry.as_path();
    let boot_json_path = src_dir.join("boot.json");

    match BootJson::new(boot_json_path.to_str().unwrap()) {
      Ok(boot_json) => {
        let zip_path = results_dir.join(
          cofg.file_name
            .replace("{name}", boot_json.name.as_str())
            .replace("{ver}", boot_json.version.as_deref().unwrap_or("1.0.0"))
        );

        match create_mod_zip(src_dir, &zip_path, boot_json) {
          Ok(_) => info!("    {}", t!("compress.done", path = src_dir.display())),
          Err(e) =>
            warn!("{}", t!("filesystem.compression_failed", path = src_dir.display(), e = e)),
        }
      }
      Err(e) => warn!("{}", t!("boot_json.read_error", e = e)),
    }
  }

  info!("=== {} ===", t!("compress.end"));
}

/// 壓縮指定目錄到zip文件
///
/// # 參數
/// * `src_dir` - 源目錄
/// * `zip_path` - 目標zip文件路徑
/// * `boot_json` - boot.json配置
fn create_mod_zip(
  src_dir: &Path,
  zip_path: &Path,
  boot_json: BootJson
) -> Result<(), Box<dyn std::error::Error>> {
  let file = File::create(zip_path)?;
  let zip = ZipWriter::new(file);
  let options: FileOptions<()> = FileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated)
    .unix_permissions(0o755)
    .compression_level(None);

  for entry in WalkDir::new(src_dir).sort_by_file_name() {
    match entry {
      Err(e) => warn!("{}", e),
      Ok(entry) => {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap();
        if path.is_file() && !boot_json.in_list(name.to_str().unwrap()) {
          remove_file(path)?;
          trace!("    f:{}", name.display());
        }
        if path.is_dir() && check_empty_dirs(&path.to_path_buf()) {
          remove_dir_all(path)?;
          trace!("    f:{}", name.display());
        }
      }
    }
  }

  zip.create_from_directory_with_options(&src_dir.to_path_buf(), |_| options)?;

  Ok(())
}

/// 文件系統操作相關的輔助函數
mod fs_utils {
  use super::{ fs, Path };

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
}

/// 將mods目錄下的所有內容複製到臨時目錄
/// 用於後續處理和打包
fn copy_to_tmp(cofg: &Cofg) {
  let mods_dir = Path::new(cofg.path.mods_path.as_str());
  let tmp_dir = Path::new(cofg.path.tmp_path.as_str());

  info!("### {} ###", t!("copy_to_tmp.start"));

  for entry in fs::read_dir(mods_dir).expect("Failed to read mods directory") {
    let entry = entry.expect("Failed to read entry");
    let path = entry.path();

    if path.is_dir() {
      if Path::new(&format!("{}/.ig", path.display())).exists() {
        info!(
          "    {}",
          t!("copy_to_tmp.skip", path = path.display().to_string().replace("/", "\\"))
        );
        continue;
      }
      if !Path::new(&format!("{}/boot.json", path.display())).exists() {
        info!(
          "    {}",
          t!("copy_to_tmp.skip", path = path.display().to_string().replace("/", "\\"))
        );
        continue;
      }
      let dest = tmp_dir.join(path.file_name().unwrap());
      if let Err(e) = fs_utils::copy_dir_all(&path, &dest) {
        warn!(
          "    {}",
          t!("filesystem.copy_dir_failed", path = path.display(), msg = format!("{}", e))
        );
      }
      info!("    {}", t!("copy.done", path = path.display().to_string().replace("/", "\\")));
    }
  }
  info!("=== {} ===", t!("copy_to_tmp.done"));
}

// 主函數
fn main() {
  // 設置 panic 處理
  human_panic::setup_panic!();

  // 初始化配置
  let cofg = Cofg::new();
  cofg.init();
  cofg.write_file();

  // 調試模式下打印配置信息
  if cfg!(debug_assertions) {
    debug!("{:#?}", cofg);

    // 測試不同日誌級別的輸出
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
    eprintln!("stderr");
    println!("stdout");
  }

  // 複製文件到臨時目錄
  copy_to_tmp(&cofg);

  // 如果需要處理 TypeScript 文件
  if cofg.ts_process {
    process_ts_files(&cofg);
  }
  // 處理 boot.json 文件
  process_boot_json_files(&cofg);

  // 壓縮打包 mod 文件
  compress_mod_folders(&cofg);

  // 如果需要暫停，等待用戶輸入
  if cofg.pause {
    info!("press any key to exit:");
    std::io::stdin().read_line(&mut String::new()).unwrap();
  }
}
