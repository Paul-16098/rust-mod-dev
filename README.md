<!-- form AI -->

# MOD 開發工具

## 發布

[Dev build](https://github.com/Paul-16098/rust-mod-dev/actions/workflows/cli.yml)
[Release build](https://github.com/Paul-16098/rust-mod-dev/release)

## 功能特點

- 自動化文件處理和打包

  - 自動掃描和識別 MOD 文件
  - 智能過濾無用文件
  - 高效率壓縮打包

- 開發者友好

  - 詳細的日誌記錄
  - 多語言支持 (支持 `zh_cn`, `zh_tw`, `en`)
  - 靈活的配置選項

- TypeScript 支持

  - 自動編譯 `.ts` 文件 (可選)

- **單元測試覆蓋**
  - 針對 `boot_json`、檔案過濾、路徑處理等核心功能皆有自動化測試
  - 測試涵蓋正常、異常、邊界情境，確保穩定性

## 使用方法

1. 將 MOD 文件夾放入 `mods` 目錄
2. 運行程序
3. 在 `results` 目錄中獲取打包後的文件

## 目錄結構

```dir
.
├── mods/           # 存放原始 MOD 文件夾
├── tmp/            # 臨時處理目錄
├── results/        # 輸出目錄
└── cofg.json       # 配置文件
```

## 配置文件 (cofg.json)

在 `cofg.json` 中可以設置以下選項：

- `locale`: 語言環境 (如 `zh_tw`, `en`)
- `loglv`: 日誌級別 (如 `warn`, `info`, `debug`, `trace`)
- `path`: 路徑相關配置
  - `tmp_path`: 臨時文件存放路徑
  - `results_path`: 輸出結果存放路徑
  - `mods_path`: MOD 源文件路徑
- `pause`: 是否在結束時暫停
- `ts_process`: 是否處理 TypeScript 文件
- `file_name`: 壓縮文件命名格式 (`{name}` 表示 MOD 名稱, `{ver}` 表示版本)

詳細結構請參考 [./cofg.schema.json](./cofg.schema.json)

## CLI 參數

程序支持以下命令行參數：

- `-i, --locale <locale>`: 設置語言環境 (如 `zh_tw`, `en`)
- `-l, --loglv <loglv>`: 設置日誌級別 (如 `warn`, `info`, `debug`, `trace`)
- `--tsp`: 啟用 TypeScript 文件處理
- `-p, --pause`: 啟用結束時暫停

示例：

```bash
mod-dev --locale zh_tw --loglv debug --tsp --pause
```

詳細結構請參考 [main.rs@Cli](./src/main.rs)

## 進階配置

### 文件過濾

- 在 MOD 目錄中創建 `.ig` 文件可以忽略該目錄
- `boot.json` 中的文件列表會自動更新

### 日誌級別

可選的日誌級別：

- `warn`: 只顯示警告和錯誤
- `info`: 顯示主要操作信息
- `debug`: 顯示詳細調試信息
- `trace`: 顯示所有跟踪信息

### TypeScript 支持

- 自動編譯非 `.d.ts` 的 `.ts` 文件
- 使用 `tsc` 命令進行編譯

## 測試

- 執行所有單元測試：

  ```bash
  cargo test
  ```

- 測試覆蓋範圍包括：
  - 路徑處理（`process_file_path`）
  - boot.json 文件讀寫、欄位補全、異常處理
  - 檔案掃描與重複過濾
  - 主要流程的正常與異常情境

## 開發環境

- Rust 2024 Edition

## 構建方法

```bash
# release
cargo build --release
# debug
cargo build
```

## 使用許可

此專案採用 MIT 授權條款。
