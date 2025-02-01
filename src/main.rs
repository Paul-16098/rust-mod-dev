use glob::glob;
use log::debug;
#[allow(unused_imports)]
use log::{info, trace, warn};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{self, File};
use std::io::{Seek, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};

rust_i18n::i18n!(fallback = ["en"]);

/// boot.json結構體
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct BootJson {
    name: Option<String>,
    additionFile: Option<Vec<String>>,
    imgFileList: Option<Vec<String>>,
    scriptFileList: Option<Vec<String>>,
    styleFileList: Option<Vec<String>>,
}

impl BootJson {
    /// 從文件路徑創建BootJson實例
    fn new(path: &str) -> Result<BootJson, Box<dyn std::error::Error>> {
        let file_content = std::fs::read(path)?;
        let mut json: BootJson = serde_json::from_slice(&file_content)?;

        // 初始化所有Option字段
        json.name = Some(json.name.unwrap_or_else(|| "unknown".to_string()));
        json.additionFile = Some(json.additionFile.unwrap_or_default());
        json.imgFileList = Some(json.imgFileList.unwrap_or_default());
        json.scriptFileList = Some(json.scriptFileList.unwrap_or_default());
        json.styleFileList = Some(json.styleFileList.unwrap_or_default());

        Ok(json)
    }

    /// 更新文件列表
    fn update_file_lists(
        &mut self,
        cwd: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let show_cwd = cwd.display();

        // 確保所有列表已初始化
        let addition_files = self.additionFile.get_or_insert_with(Vec::new);
        let img_files = self.imgFileList.get_or_insert_with(Vec::new);
        let script_files = self.scriptFileList.get_or_insert_with(Vec::new);
        let style_files = self.styleFileList.get_or_insert_with(Vec::new);

        // 處理附加文件
        for file in ["README.md", "README.txt", "License.txt", "License"].iter() {
            let file_path = format!("{}/{}", show_cwd, file);
            if std::path::Path::new(&file_path).exists()
                && !addition_files.contains(&file.to_string())
            {
                addition_files.push(file.to_string());
            }
        }

        // 處理各類型文件
        scan_and_add_files(&format!("{}/**/*.png", show_cwd), img_files, cwd)?;
        scan_and_add_files(&format!("{}/**/*.js", show_cwd), script_files, cwd)?;
        scan_and_add_files(&format!("{}/**/*.css", show_cwd), style_files, cwd)?;

        Ok(())
    }
}

/// 將文件夾添加到zip壓縮包中
fn add_to_zip<W>(path: &str, zip: &mut ZipWriter<W>)
where
    W: Write + Seek,
{
    let options: FileOptions<()> = FileOptions::default();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let new_path = entry.path();
        let name = new_path
            .strip_prefix(std::path::Path::new(path))
            .unwrap()
            .to_path_buf();

        #[cfg(debug_assertions)]
        trace!("add to zip: {}", new_path.display());

        if new_path.is_file() {
            let mut f = File::open(new_path).unwrap();
            zip.start_file(name.to_str().unwrap(), options).unwrap();
            std::io::copy(&mut f, zip).unwrap();
        } else if name.as_os_str().len() != 0 {
            zip.add_directory(name.to_str().unwrap(), options).unwrap();
        }
    }
}

/// 用於處理文件路徑的工具函數
fn process_file_path(path: &std::path::Path, cwd: &std::path::Path) -> Option<String> {
    path.strip_prefix(cwd).ok()?.to_str().map(|s| s.to_string())
}

/// 掃描並添加特定類型的文件
fn scan_and_add_files(
    pattern: &str,
    file_list: &mut Vec<String>,
    cwd: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in glob(pattern)? {
        if let Ok(path) = entry {
            if let Some(rel_path) = process_file_path(&path, cwd) {
                if !file_list.contains(&rel_path) {
                    file_list.push(rel_path);
                }
            }
        }
    }
    Ok(())
}

/// 處理boot.json文件
pub fn process_boot_json_files() {
    info!("{}", t!("boot_json.start"));

    for entry in glob("./tmp/*/boot.json").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let cwd = path.parent().unwrap();
                info!("{}", t!("boot_json.processing", path = path.display()));

                match BootJson::new(path.to_str().unwrap()) {
                    Ok(mut boot_json) => {
                        if let Err(e) = boot_json.update_file_lists(cwd) {
                            warn!("更新文件列表失敗: {:?}", e);
                            continue;
                        }

                        // 保存更新後的boot.json
                        match serde_json::to_string_pretty(&boot_json) {
                            Ok(json_string) => {
                                if let Err(e) = std::fs::write(&path, json_string) {
                                    warn!("寫入boot.json失敗: {:?}", e);
                                }
                            }
                            Err(e) => warn!("序列化JSON失敗: {:?}", e),
                        }
                    }
                    Err(e) => warn!("讀取boot.json失敗: {:?}", e),
                }
            }
            Err(e) => warn!("{:?}", e),
        }
    }
    info!("{}", t!("boot_json.end"));
}

/// 壓縮所有的mod文件夾
pub fn compress_mod_folders() {
    info!("{}", t!("compress.start"));

    let results_dir = "./results/";
    if std::path::Path::new(results_dir).exists() {
        std::fs::remove_dir_all(results_dir).expect("Failed to remove directory");
    }
    std::fs::create_dir(results_dir).expect("Failed to create results directory");

    for entry in glob("./tmp/*/").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let cwd = path.as_path();
                let boot_json = BootJson::new(format!("{}/boot.json", cwd.display()).as_str())
                    .expect("Failed to create BootJson");
                let zip_file = match File::create(&format!(
                    "{results_dir}{}.mod.zip",
                    boot_json.name.unwrap_or_else(|| "unknown".to_string())
                )) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Failed to create file: {:?}", e);
                        #[cfg(debug_assertions)]
                        trace!("{}", cwd.display());
                        continue;
                    }
                };
                let mut zip = ZipWriter::new(zip_file);

                // 壓縮所有文件
                add_to_zip(cwd.to_str().unwrap(), &mut zip);

                match zip.finish() {
                    Ok(_) => info!("{}", t!("compress.done", path = cwd.display())),
                    Err(e) => warn!("{:?}", e),
                }
            }
            Err(e) => warn!("{:?}", e),
        }
    }
    info!("{}", t!("compress.end"));
}

/// 遞迴複製目錄
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// copy ./mods/* to ./tmp/*
pub fn copy_to_tmp() {
    info!("{}", t!("copy_to_tmp.start"));
    let mods_dir = Path::new("./mods");
    let tmp_dir = Path::new("./tmp");

    if tmp_dir.exists() {
        fs::remove_dir_all(tmp_dir).expect("Failed to remove tmp directory");
    }
    fs::create_dir(tmp_dir).expect("Failed to create tmp directory");
    if !mods_dir.exists() {
        fs::create_dir(mods_dir).expect("Failed to create mods directory");
    }

    for entry in fs::read_dir(mods_dir).expect("Failed to read mods directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            let dest = tmp_dir.join(path.file_name().unwrap());
            if let Err(e) = copy_dir_all(&path, &dest) {
                warn!("Failed to copy directory {}: {}", path.display(), e);
            }
            info!(
                "{}",
                t!(
                    "copy.done",
                    path = path.display().to_string().replace("/", "\\")
                )
            );
        }
    }
    info!("{}", t!("copy_to_tmp.done"));
}

fn main() {
    let mut locale = "en";
    let config_content = match fs::read_to_string("./cofg.txt") {
        Ok(f) => f,
        Err(_) => "locale=en".to_string(),
    };
    for line in config_content.lines() {
        let mut l = line.split("=");
        let key = l.next().unwrap_or("");
        let value = l.next().unwrap_or("");
        if key == "locale" {
            if ["zh", "en"].contains(&value) {
                locale = value;
            }
        }
    }

    rust_i18n::set_locale(locale);
    colog::init();
    debug!("cwd: {:?}", std::env::current_dir().unwrap());
    debug!("locale: {:?}", rust_i18n::locale().as_ref() as &str);

    copy_to_tmp();
    process_boot_json_files();
    compress_mod_folders();
}
