# MOD 開發工具

一個用於處理和打包 MOD 文件的 Rust 工具。

## 功能特點

- 自動掃描並處理 MOD 文件夾
- 更新 boot.json 文件配置
- 自動打包成發布格式
- 支持多語言界面 (中文/英文)

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
└── cofg.txt        # 配置文件
```

## 配置文件

在 `cofg.txt` 中可以設置以下選項：

```plaintext
locale=zh    # 設置界面語言（zh/en）
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
cargo build --release
```

## 使用許可

此專案採用 MIT 授權條款。
