# MOD 開發工具

## 功能特點

- 自動化文件處理和打包

  - 自動掃描和識別 MOD 文件
  - 智能過濾無用文件
  - 高效率壓縮打包

- 開發者友好
  - 詳細的日誌記錄
  - 多語言界面支持
  - 靈活的配置選項

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

see [./cofg.schema.json](./cofg.schema.json)

## 進階配置

### 文件過濾

- 在 MOD 目錄中創建 `.ig` 文件可以忽略該目錄
- boot.json 中的文件列表會自動更新

### 日誌級別

可選的日誌級別：

- warn: 只顯示警告和錯誤
- info: 顯示主要操作信息
- debug: 顯示詳細調試信息
- trace: 顯示所有跟踪信息

## 開發環境

- Rust 2021 Edition

## 構建方法

```bash
# release
cargo build --release
# debug
cargo build
```

## 使用許可

此專案採用 MIT 授權條款。
