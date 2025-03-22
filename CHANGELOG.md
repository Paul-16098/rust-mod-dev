# 變更日誌

## 1.3.3 - 2025-3-22

- 在 `cofg.schema.json` 中新增 `file_name` 屬性。
- 在 `compress_mod_folders` 函數中使用 `file_name` 模板來生成壓縮文件名。
- 更新 `cofg.schema.json` 文件以包含 `file_name` 屬性。
- 更新 `src/boot_json.rs` 文件以公開 `version` 字段。
- 更新 `src/main.rs` 文件以處理新的 `file_name` 屬性。
