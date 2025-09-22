# Copilot instructions for rust-mod-dev

Purpose: a CLI that scans per-mod folders, normalizes boot.json, optionally compiles TS, and zips each mod into results. Keep changes aligned with these patterns.

## Big picture
- Orchestration in `src/main.rs` (order matters): `copy_to_tmp` → `process_ts_files` (optional) → `process_boot_json_files` → `compress_mod_folders` → optional pause.
- Config `Cofg` (`src/cofg.rs`) loads from `./cofg.json` (see `cofg.schema.json`), then applies CLI overrides (clap). First run (no file) prompts via dialoguer and writes defaults.
- i18n with `rust-i18n`, keys in `locales/app.yml` via `t!(...)`. Logging via `colog` + `log`, level from config/CLI.
- Build metadata: `build.rs` injects env `VERSION = <pkg-ver>(<profile>)-<git-commit>(actions/runs/<id>|Local)`, used by clap `--version`.

## Conventions & gotchas
- Mods live under `./mods/<modName>/` and MUST have a `boot.json` at the root; a `.ig` file in a mod folder makes `copy_to_tmp` skip it.
- Inside `boot.json`, use forward slashes (`/`) for paths. `BootJson::in_list()` normalizes but favor `/` to avoid mismatches.
- `process_boot_json_files()` auto-fills lists by scanning the mod folder: `**/*.png`, `**/*.js`, `**/*.css`, `**/*.twee`, plus `README.*`, `License*`, `*.js.map` into `additionFile`.
- Before zipping, `compress_mod_folders()` removes files not listed in `boot.json` (except `boot.json`) and prunes empty directories. Unlisted files won’t ship.
- Zip name pattern from `Cofg.file_name` using `{name}` and `{ver}` (from `boot.json`, default `1.0.0`), output to `Cofg.path.results_path` (default `./results`).
- TS pipeline: if `Cofg.ts_process` or `--tsp`, run `tsc` per temp mod folder containing `.ts` (excluding `.d.ts`). Uses `tsc.cmd` on Windows; ensure `tsc` is on PATH.
- `Cofg.init()` clears and recreates `tmp`/`results`, and ensures `mods` exists. Do not point these paths to important dirs.

## Source map
- `src/boot_json.rs`: data model + `new()`, `update_file_lists()`, `in_list()`, `scan_and_add_files()`, `process_file_path()`.
- `src/cofg.rs`: CLI, config load/validate/write, i18n/log init, interactive defaults, `Default` values.
- `src/fs_utils.rs`: `copy_dir_all()` (skips `.git`), `check_empty_dirs()`.
- `src/main.rs`: pipeline, zipping (`zip`, `zip-extensions`), walking (`walkdir`), globbing (`glob`).

## Developer workflows
- Build: `cargo build` (or `--release`).
- Tests: `cargo test` (VS Code task available: “cargo: nextest”). Key cases in `src/tests/mod.rs` cover path handling, list updates, and main pipeline.
- Typical run: put mods in `./mods/`, run the binary, collect zips from `./results/`.
- CI: `.github/workflows/cli.yml` cross-builds (Linux/musl, Windows MSVC, macOS, FreeBSD) and publishes artifacts; exports `ACTIONS_ID`. Weekly rustsec audit in `Security-audit.yml`.

## Examples to follow
- Include new file type: extend scan patterns in `update_file_lists()` and list checks in `in_list()` (both in `src/boot_json.rs`).
- Skip a mod folder: add `.ig` inside `mods/<name>/` (honored by `copy_to_tmp`).
- Force ship a file: add its relative path (with `/`) to `additionFile` in `boot.json`.

## Quality bar
- Keep `boot.json` paths relative to mod root and use `/` separators.
- Preserve pipeline order and the prune-before-zip behavior.
- Localize user-facing strings with `t!(...)` and define keys in `locales/app.yml`.
