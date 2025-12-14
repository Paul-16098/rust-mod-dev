# Copilot Instructions for rust-mod-dev

**Purpose:** A Rust CLI that orchestrates MOD packaging: scans per-mod folders, auto-normalizes `boot.json`, optionally transpiles TypeScript, and outputs zipped MODs.

---

## Architecture & Data Flow

### Pipeline (Order-Critical)

The main orchestration in `src/main.rs` executes sequentially:

1. **`copy_to_tmp`** — Copies mods from `./mods/<modName>` to temp working directory; skips folders containing `.ig` marker files.
2. **`process_ts_files`** (conditional) — If `Cofg.ts_process` enabled, runs `tsc` on each temp folder with `.ts` files (excluding `.d.ts`).
3. **`process_boot_json_files`** — Reads/validates `boot.json`, auto-scans and populates file lists, writes normalized JSON back.
4. **`compress_mod_folders`** — Prunes unlisted files and empty directories, then zips mods to `./results/`.
5. **`pause`** (optional) — Pauses execution for user inspection.

### Config & Initialization (`Cofg` in `src/cofg.rs`)

- **Load source:** `./cofg.json` (schema defined in `cofg.schema.json`), then CLI overrides via clap.
- **First run:** If `cofg.json` doesn't exist, `new_user_select_cofg()` interactively prompts (locale, TS mode, pause, file naming, log level) via `dialoguer`, then writes config.
- **Critical:** `Cofg::init()` clears and recreates `tmp/` and `results/`; never point these to important directories.

### Metadata & Versioning (`build.rs`)

- Injects `VERSION` env var at compile time: `<pkg-ver>(<profile>)-<git-commit>(actions/runs/<id>|Local)`
- Git commit fetched from `.git`; in CI, `ACTIONS_ID` env var used; falls back to "Local" when not in GitHub Actions.
- `--version` flag displays this injected version string.

### Boot.json Structure (`src/boot_json.rs`)

- **Core fields:** `name` (required), `version` (optional, defaults to `1.0.0`).
- **File lists:** `imgFileList`, `scriptFileList`, `styleFileList`, `tweeFileList`, `additionFile` (e.g., README, License, `.js.map`).
- **Relations:** `addonPlugin` (nested: `modName`, `addonName`, `modVersion`, `params`), `dependenceInfo` (nested: `modName`, `version`).

---

## Conventions & Critical Gotchas

### Path Handling

- Mods **must** live at `./mods/<modName>/` with `boot.json` at root.
- Inside `boot.json`, **always use forward slashes** (`/`) for paths; `process_file_path()` normalizes cross-platform but favoring `/` avoids mismatches.
- `process_file_path()` strips base path and returns relative paths in `/` format.

### Auto-Scanning (`update_file_lists()`)

- Scans these patterns: `**/*.png`, `**/*.js`, `**/*.css`, `**/*.twee`, plus `README.*`, `License*`, `*.js.map`.
- Results populate the respective file lists; always validates via `in_list()` before zipping.

### Pre-Zip Pruning (`compress_mod_folders()`)

- **Removes** all files NOT listed in normalized `boot.json` (except `boot.json` itself).
- **Prunes** empty directories.
- **Unlisted files won't ship** — ensure file lists are complete.

### TypeScript Pipeline

- **Trigger:** `--tsp` CLI flag or `Cofg.ts_process = true`.
- **Command:** Windows uses `tsc.cmd`; other OSes use `tsc` directly (ensure in PATH).
- **Scope:** Runs per temp mod folder; skips `.d.ts` files.

### Zip Output

- **Naming:** Derived from `Cofg.file_name` template using `{name}` (from `boot.json`) and `{ver}` (version, default `1.0.0`).
- **Location:** `Cofg.path.results_path` (default `./results`).

### Skip Mechanism

- Add a `.ig` file inside `mods/<modName>/` to make `copy_to_tmp` skip that folder entirely.

---

## Internationalization & Logging

- **i18n:** All user-facing strings use `t!(key)` macro from `rust-i18n`; translations in `locales/app.yml`.
- **Logging:** Via `colog` + `log` crate; level set by `Cofg.loglv` or CLI (`--loglv` flag). Levels: warn, info, debug, trace.

---

## Source Map

| Module             | Responsibility                                                                                          |
| ------------------ | ------------------------------------------------------------------------------------------------------- |
| `src/main.rs`      | Pipeline orchestration, file I/O, zipping logic (using `zip` & `zip-extensions` crates)                 |
| `src/boot_json.rs` | `BootJson` struct, file list scanning, path normalization (`process_file_path`), validation (`in_list`) |
| `src/cofg.rs`      | Config load/save, CLI parsing (clap), interactive setup, i18n/log initialization                        |
| `src/fs_utils.rs`  | Filesystem utilities: `copy_dir_all()` (skips `.git`), `check_empty_dirs()`                             |
| `build.rs`         | Version injection, Git/Actions integration                                                              |

---

## Developer Workflows

### Building

```bash
cargo build        # Debug
cargo build --release
```

### Testing

```bash
cargo test
# or via VS Code task: "cargo: nextest"
```

Key test cases in `src/tests/mod.rs`: path normalization, list updates, file scanning. Tests use `tempfile` for isolated file operations.

### Local Development

1. Create test mods in `./mods/<modName>/` with `boot.json`.
2. Run binary; first execution creates `./cofg.json` interactively.
3. Outputs appear in `./results/`.

### CI/CD

- **`.github/workflows/cli.yml`:** Cross-builds for Linux (musl), Windows (MSVC), macOS, FreeBSD; publishes release artifacts.
- **`.github/workflows/Security-audit.yml`:** Weekly rustsec vulnerability scan.
- CI sets `ACTIONS_ID` env var for version tagging.

---

## Common Extension Points

### Adding a New File Type

1. Update scan pattern in `update_file_lists()` (e.g., `**/*.md`).
2. Add new field to `BootJson` struct (e.g., `docFileList`).
3. Update `in_list()` to check the new list.
4. Add localization strings to `locales/app.yml`.

### Customizing Zip Output

- Modify `Cofg.file_name` template (e.g., `"v{ver}-{name}.zip"`).
- Adjust compression logic in `compress_mod_folders()` if needed.

### Skipping Mods Selectively

- Add `.ig` marker file to mod folders to exclude from pipeline.

---

## Quality Standards

- **Paths in boot.json:** Always relative to mod root, use `/` separators (not `\`).
- **Pipeline order:** Critical; maintain sequence to ensure TS compilation before boot.json processing.
- **Localization:** Every user-facing string in `t!(...)` with keys in `locales/app.yml`.
- **File pruning:** Do not remove files users expect; verify `boot.json` lists are complete before shipping.
