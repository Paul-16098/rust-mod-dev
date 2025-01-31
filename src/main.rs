#![allow(non_snake_case)]

use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use std::{
    fs::File,
    io::{Seek, Write},
    process::Command,
};
use walkdir::WalkDir;
use zip::write::{FileOptions, ZipWriter};

#[derive(Serialize, Deserialize, Debug)]
struct BootJson {
    name: Option<String>,
    additionFile: Option<Vec<String>>,
    imgFileList: Option<Vec<String>>,
    scriptFileList: Option<Vec<String>>,
    styleFileList: Option<Vec<String>>,
}

impl BootJson {
    fn new(path: &str) -> BootJson {
        let file_content = std::fs::read(path).expect("Failed to read file");
        let json: BootJson = serde_json::from_slice(&file_content).expect("Failed to parse JSON");
        return json;
    }
}

fn add_to_zip<W>(path: &str, zip: &mut ZipWriter<W>)
where
    W: Write + Seek,
{
    let options: FileOptions<()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let mut added_files = HashSet::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let new_path = entry.path();
        let mut name = new_path
            .strip_prefix(std::path::Path::new(path))
            .unwrap()
            .to_path_buf();

        #[cfg(debug_assertions)]
        println!("add to zip: {}", new_path.display());

        if new_path.is_file() {
            let mut f = File::open(new_path).unwrap();
            while added_files.contains(&name) {
                name = name.with_extension(format!(
                    "{}_dup",
                    name.extension().unwrap_or_default().to_str().unwrap()
                ));
            }
            zip.start_file(name.to_str().unwrap(), options).unwrap();
            std::io::copy(&mut f, zip).unwrap();
            added_files.insert(name);
        } else if name.as_os_str().len() != 0 {
            zip.add_directory(name.to_str().unwrap(), options).unwrap();
        }
    }
}

fn main() {
    // 在debug模式下打印當前工作目錄
    #[cfg(debug_assertions)]
    println!("cwd: {:?}", std::env::current_dir().unwrap());

    println!("===== ts相關處理開始");

    // 編譯所有的TypeScript文件
    for entry in glob("./mods/*/").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                // 檢查是否存在TypeScript文件
                if glob(&format!("{}/**/*.ts", path.display()))
                    .expect("Failed to read glob pattern")
                    .count()
                    > 0
                {
                    // 編譯TypeScript文件
                    let output = Command::new("tsc.cmd")
                        .arg(format!("-p {}", path.to_str().unwrap()))
                        .stdout(std::process::Stdio::null())
                        .status();
                    match output {
                        Ok(_) => println!("    {} 編譯完畢", path.to_str().unwrap()),
                        Err(e) => println!("{:?}", e),
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    println!("##### ts相關處理結束");

    println!("===== boot.json相關處理開始");

    // 處理所有的boot.json文件
    for entry in glob("./mods/*/boot.json").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let cwd = path.parent().unwrap();
                println!("    處理boot.json文件: {:?}", path.display());

                // 加載boot.json文件
                let boot_json_file = File::open(path.clone()).expect("Failed to open file");
                let mut boot_json: BootJson = match serde_json::from_reader(boot_json_file) {
                    Err(e) => {
                        println!("Failed to parse JSON in file {:?}: {:?}", path, e);
                        continue;
                    }
                    Ok(f) => f,
                };

                let mut boot_json_file = File::create(path.clone()).expect("Failed to open file");

                // 確保boot.json中的字段不為None
                boot_json
                    .name
                    .get_or_insert_with(|| cwd.file_name().unwrap().to_str().unwrap().to_string());
                boot_json.additionFile.get_or_insert_with(Vec::new);
                boot_json.imgFileList.get_or_insert_with(Vec::new);
                boot_json.scriptFileList.get_or_insert_with(Vec::new);
                boot_json.styleFileList.get_or_insert_with(Vec::new);

                // 添加README和License文件到additionFile
                for file in ["README.md", "README.txt", "License.txt", "License"].iter() {
                    if std::path::Path::new(&format!("{}/{file}", cwd.display())).exists() {
                        boot_json
                            .additionFile
                            .as_mut()
                            .unwrap()
                            .push(file.to_string());
                    }
                }

                // 添加所有的img文件到boot.json
                for entry in glob(&format!("{}/**/*.img", cwd.display()))
                    .expect("Failed to read glob pattern")
                {
                    match entry {
                        Ok(path) => {
                            boot_json
                                .imgFileList
                                .as_mut()
                                .unwrap()
                                .push(path.to_str().unwrap().to_string());
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }

                // 添加所有的js文件到boot.json
                for entry in glob(&format!("{}/**/*.js", cwd.display()))
                    .expect("Failed to read glob pattern")
                {
                    match entry {
                        Ok(path) => {
                            boot_json
                                .scriptFileList
                                .as_mut()
                                .unwrap()
                                .push(path.to_str().unwrap().to_string());
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }

                // 添加所有的css文件到boot.json
                for entry in glob(&format!("{}/**/*.css", cwd.display()))
                    .expect("Failed to read glob pattern")
                {
                    match entry {
                        Ok(path) => {
                            boot_json
                                .styleFileList
                                .as_mut()
                                .unwrap()
                                .push(path.to_str().unwrap().to_string());
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }

                // 將更新後的boot.json寫回文件
                let boot_json_string =
                    serde_json::to_string(&boot_json).expect("Failed to serialize JSON");
                boot_json_file
                    .write_all(boot_json_string.as_bytes())
                    .expect("Failed to write file");
            }
            Err(e) => println!("{:?}", e),
        }
    }
    println!("##### 處理boot.json文件結束");

    println!("===== 壓縮所有的mod文件夾開始");

    let results_dir = "./results/";
    if std::path::Path::new(results_dir).exists() {
        std::fs::remove_dir_all(results_dir).expect("Failed to remove directory");
    }
    std::fs::create_dir(results_dir).expect("Failed to create results directory");

    for entry in glob("./mods/*/").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let cwd = path.as_path();
                let boot_json = BootJson::new(format!("{}/boot.json", cwd.display()).as_str());
                let zip_file = match File::create(&format!(
                    "{results_dir}{}.mod.zip",
                    boot_json.name.unwrap_or_else(|| "unknown".to_string())
                )) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to create file: {:?}", e);
                        #[cfg(debug_assertions)]
                        println!("{}", cwd.display());
                        continue;
                    }
                };
                let mut zip = ZipWriter::new(zip_file);

                // 壓縮所有文件
                add_to_zip(cwd.to_str().unwrap(), &mut zip);

                match zip.finish() {
                    Ok(_) => println!("    壓縮{}完畢", cwd.display()),
                    Err(e) => println!("{:?}", e),
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
    println!("##### 壓縮所有的mod文件夾結束");
}
