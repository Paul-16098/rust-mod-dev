# MOD 開發工具

一個 MOD 文件處理和打包工具，提供 MOD 開發工作流程支持。

## 核心功能

- 自動化文件處理

  - MOD 文件掃描和分類
  - 配置文件自動更新
  - 文件打包

- 開發輔助功能
  - 多語言界面
  - 詳細日誌記錄

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
└── cofg.json        # 配置文件
```

## 配置文件 (cofg.json)

在 `cofg.json` 中可以設置以下選項：

```json
{
  "locale": "zh_cn", // 設置界面語言（zh_cn/zh_tw/en）
  "loglv": "info" // 日誌等級 (warn/info/debug/trace)
}
```

## 開發環境

- Rust 2021 Edition
- 依賴項：
  - glob
  - serde_json
  - zip
  - walkdir
  - rust-i18n

## 構建方法

```bash
# release
cargo build --release
# debug
cargo build
```

## 使用許可

此專案採用 MIT 授權條款。
