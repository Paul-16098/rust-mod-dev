<!-- Generated with the README blueprint: scans .github/copilot-instructions.md and codebase to produce a developer-focused guide. -->

# mod-dev（rust-mod-dev）

> 一個以 Rust 編寫的 CLI，用於掃描每個 MOD 子資料夾、標準化 `boot.json`、可選編譯 TypeScript，並將每個 MOD 打包為 zip 輸出。

[Dev build（CI）](https://github.com/Paul-16098/rust-mod-dev/actions/workflows/cli.yml) · [Releases](https://github.com/Paul-16098/rust-mod-dev/release)

---

## 技術棧（Technology Stack）

- 語言與版號：Rust（Edition 2024）
- 關鍵 crates：
  - `clap` 4.x（CLI 參數解析）
  - `serde`/`serde_json`（設定與 boot.json 的序列化/反序列化）
  - `glob`、`walkdir`（檔案掃描）
  - `zip`、`zip-extensions`（壓縮打包）
  - `colog`、`log`（結構化日誌）
  - `rust-i18n`（多語系，字串在 `locales/app.yml`）
  - `config`（讀取 `cofg.json`）
  - `nest_struct`（巢狀結構巨集）
  - `human-panic`（panic 體驗優化）
  - `dialoguer`（新用戶互動化初始化）
  - 測試：`tempfile`
- CI/CD：GitHub Actions cross-build（Linux/musl、Windows MSVC、macOS、FreeBSD），週期性安全審計（rustsec）。

## 專案架構（Architecture）

- 核心流程（順序不可變）：
  1. `copy_to_tmp` → 2) `process_ts_files`（可選） → 3) `process_boot_json_files` → 4) `compress_mod_folders` → 5) `pause`（可選）。
- 設定物件 `Cofg`（`src/cofg.rs`）：
  - 優先讀取 `./cofg.json`（結構見 `cofg.schema.json`），若不存在則以 `dialoguer` 問答產生預設並寫回檔案。
  - CLI 參數覆蓋設定（`clap`）。`init()` 將清空重建 `tmp`/`results` 並確保 `mods` 存在，請勿把這些路徑指向重要資料夾。
- I18N 與日誌：
  - 所有對使用者可見的字串透過 `t!(...)`（字典於 `locales/app.yml`）。
  - 日誌層級由 `Cofg.loglv`/CLI 決定（`colog` + `log`）。
- 版本字串：`build.rs` 注入 `VERSION = <pkg-ver>(<profile>)-<git-commit>(actions/runs/<id>|Local)`，`--version` 會顯示。
- 元件地圖：
  - `src/main.rs`：流程管線與壓縮/掃描實作
  - `src/boot_json.rs`：`BootJson` 結構、`new`/`update_file_lists`/`in_list` 等
  - `src/cofg.rs`：設定載入/驗證/寫入、CLI、i18n+log 初始化、互動式初始設定
  - `src/fs_utils.rs`：`copy_dir_all()`（跳過 `.git`）、`check_empty_dirs()`

> 延伸閱讀：`.github/copilot-instructions.md`（面向 AI/開發者的縮寫版作業規範）

## 快速開始（Getting Started）

### 先決條件

- 已安裝 Rust 工具鏈
- 如需 TypeScript 流程：安裝 Node.js 與 TypeScript，並確保 PATH 可執行 `tsc`（Windows 以 `tsc.cmd` 呼叫）

### 安裝與建置

```powershell
# 下載相依並建置（Debug）
cargo build

# 釋出版建置（建議實際使用）
cargo build --release
```

可執行檔位置：`target/release/mod-dev`（Windows 為 `mod-dev.exe`）。

### 基本使用

1. 將每個 MOD 放到 `./mods/<modName>/`，且該資料夾根目錄需有 `boot.json`。
2. 第一次執行會互動產生 `cofg.json`。之後可直接執行，zip 會輸出到 `./results/`。

```powershell
# 常用參數示例
./target/release/mod-dev --locale zh_tw --loglv debug --tsp --pause
```

### 設定（cofg.json）

- 結構定義：`cofg.schema.json`
- 重要欄位：
  - `locale`（zh_cn/zh_tw/en）、`loglv`（warn/info/debug/trace）
  - `path.tmp_path`、`path.results_path`、`path.mods_path`
  - `pause`（結束後暫停）
  - `ts_process`（啟用 TS 編譯）
  - `file_name`（壓縮檔命名樣板，支援 `{name}`、`{ver}`）

## 專案結構（Project Structure）

```text
.
├─ src/
│  ├─ main.rs          # 流程編排、壓縮與掃描
│  ├─ boot_json.rs     # BootJson 結構與掃描/比對
│  ├─ cofg.rs          # 設定、CLI、i18n/log 初始化
│  ├─ fs_utils.rs      # 檔案/目錄工具
│  └─ tests/mod.rs     # 單元測試
├─ mods/               # 放置原始 MOD
├─ tmp/                # 流程運行時臨時資料
├─ results/            # 輸出 zip
├─ locales/app.yml     # i18n 字典
├─ cofg.schema.json    # 設定 Schema
├─ build.rs            # 版本字串注入
└─ .github/
   ├─ workflows/cli.yml          # Cross-build 與發佈
   ├─ workflows/Security-audit.yml
   └─ copilot-instructions.md    # 開發/AI 快速規範
```

## 主要功能（Key Features）

- 自動掃描 MOD 檔案、標準化 `boot.json` 清單
- 過濾未列於 `boot.json` 的檔案（除 `boot.json` 本身），並清空空目錄
- 可選擇每個暫存 MOD 目錄執行 `tsc`（排除 `.d.ts`）
- 多語系日誌與友善錯誤訊息（`human-panic`）
- 跨平台建置與版本戳記（含 Git commit 與 GitHub `ACTIONS_ID`）

## 開發流程（Development Workflow）

- 一般流程：
  1. 放置 MOD → 2) 執行 CLI → 3) 收取 `results/` zip。
- CI：`.github/workflows/cli.yml` 於多平台建置並發布產物，環境變數 `ACTIONS_ID` 會寫入版本字串；安全審計工作流每週執行一次。
- 注意：`Cofg.init()` 會清空 `tmp` 與 `results`；請勿把它們設定為珍貴路徑。

## 編碼規範與慣例（Coding Standards）

- `boot.json` 內的路徑一律使用 `/` 分隔（Windows 也同樣如此）。
- `process_boot_json_files()` 會自動掃描並補齊：`**/*.png`, `**/*.js`, `**/*.css`, `**/*.twee`，以及 `README.*`、`License*`、`*.js.map`（進 `additionFile`）。
- 打包前必定清理未列出的檔案（除 `boot.json`），確保產物最小化且可重現。
- 新增使用者可見字串請使用 `t!(...)` 並在 `locales/app.yml` 補上對應 key。
- 變更檔案型別收錄規則：請同步更新 `src/boot_json.rs` 的 `update_file_lists()` 與 `in_list()`。

> 更多可操作的準則與範例：請參考 `.github/copilot-instructions.md`。

## Docker 使用說明（Docker Usage）

### 建置映像（Build the image）

```powershell
docker build -t rust-mod-dev:latest .
```

### 執行 CLI（Run the CLI in container）

建議將本機 mods、results、tmp 目錄掛載到容器內 `/work`，以便持久化與互通：

```powershell
# Windows PowerShell
docker run --rm -v ${PWD}:/work rust-mod-dev:latest --help

# Linux/macOS bash
docker run --rm -v $(pwd):/work rust-mod-dev:latest --help
```

#### 常見掛載範例

```powershell
# 掛載本機 mods、results、tmp 到容器
docker run --rm -v ${PWD}/mods:/work/mods -v ${PWD}/results:/work/results -v ${PWD}/tmp:/work/tmp rust-mod-dev:latest

# 覆蓋 cofg.json 設定
docker run --rm -v ${PWD}/cofg.json:/work/cofg.json rust-mod-dev:latest
```

> 提示：
>
> - 容器內已提供一份預設 `cofg.json`，可用 `-v` 掛載覆蓋。
> - 若需要 TypeScript 編譯（`ts_process: true` 或 `--tsp`），請在宿主機先行完成 tsc 編譯後再執行容器，或自訂 Dockerfile 加入 Node.js/tsc；本預設映像不包含 tsc。
> - 映像包含簡單 HEALTHCHECK（執行 `mod-dev --version`），可用 `docker ps` 查看健康狀態。

---

- 單元測試（見 `src/tests/mod.rs`）涵蓋：
  - 路徑處理（`process_file_path`）
  - `BootJson` 清單更新與 `in_list`
  - 檔案掃描與去重
  - 主流程管線的基本煙霧測試

```powershell
# 執行測試
cargo test
```

（VS Code 使用者可使用工作任務「cargo: nextest」。）

## 貢獻（Contributing）

- 提交改動前請閱讀 `.github/copilot-instructions.md` 的慣例（流程順序、i18n、壓縮前清理、路徑分隔符要求）。
- 若新增檔案型別或輸出邏輯，請同步補齊：
  - 掃描/清單與 `in_list()` 規則
  - 相對應 i18n key
  - 基本單元測試案例
