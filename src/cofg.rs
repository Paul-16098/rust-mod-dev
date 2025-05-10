//! 配置相關結構體和實現

use clap::{ Parser, ArgAction };
use log::warn;
use nest_struct::nest_struct;
use config::Config;
use serde::{ Deserialize, Serialize };
use rust_i18n::t;
use super::fs;
use super::r#const::VERSION;

#[nest_struct]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct Cofg {
  /// 程序使用的語言環境(zh_cn/zh_tw/en)
  locale: String,
  /// 日誌級別(warn/info/debug/trace)
  loglv: String,
  /// 路徑相關配置
  pub path: PathCofg! {
    /// 臨時文件存放路徑
    pub tmp_path: String,
    /// 輸出結果存放路徑
    pub results_path: String,
    /// mod源文件路徑
    pub mods_path: String,
  },
  /// 最後暫停?
  pub pause: bool,
  /// 處理 ts 文件?
  pub ts_process: bool,
  /// file name
  pub file_name: String,
}

impl Cofg {
  /// 配置初始化函數
  /// * 讀取並解析cofg.json文件
  pub(crate) fn new() -> Cofg {
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
  pub(crate) fn write_file(&self) {
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
  pub(crate) fn init(&self) {
    self.clone().load_cli(Cli::parse());

    for path in [&self.path.tmp_path, &self.path.results_path].iter() {
      let path_obj = std::path::Path::new(path);
      if path_obj.exists() {
        fs::remove_dir_all(path_obj).unwrap();
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
