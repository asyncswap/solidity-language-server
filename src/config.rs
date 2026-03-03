use serde::Deserialize;
use std::path::{Path, PathBuf};

// ── LSP Settings (from editor / initializationOptions) ─────────────────

/// Top-level settings object sent by the editor.
///
/// Editors wrap settings under the server name key:
/// ```lua
/// settings = {
///   ["solidity-language-server"] = {
///     inlayHints = { parameters = true },
///     lint = { enabled = true, exclude = { "pascal-case-struct" } },
///   },
/// }
/// ```
///
/// All fields use `Option` with `#[serde(default)]` so that missing keys
/// keep their defaults — the editor only needs to send overrides.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct Settings {
    #[serde(default)]
    pub inlay_hints: InlayHintsSettings,
    #[serde(default)]
    pub lint: LintSettings,
    #[serde(default)]
    pub file_operations: FileOperationsSettings,
    #[serde(default)]
    pub project_index: ProjectIndexSettings,
}

/// Inlay-hint settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlayHintsSettings {
    /// Show parameter-name hints on function/event/struct calls.
    #[serde(default = "default_true")]
    pub parameters: bool,
    /// Show gas estimate hints on functions/contracts annotated with
    /// `@custom:lsp-enable gas-estimates`.
    #[serde(default = "default_true")]
    pub gas_estimates: bool,
}

impl Default for InlayHintsSettings {
    fn default() -> Self {
        Self {
            parameters: true,
            gas_estimates: true,
        }
    }
}

/// Lint settings (overrides foundry.toml when provided).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LintSettings {
    /// Master toggle for forge-lint diagnostics.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Filter lints by severity (e.g. `["high", "med", "gas"]`).
    /// Maps to `forge lint --severity high --severity med --severity gas`.
    /// Empty means all severities.
    #[serde(default)]
    pub severity: Vec<String>,
    /// Run only specific lint rules by ID (e.g. `["incorrect-shift", "unchecked-call"]`).
    /// Maps to `forge lint --only-lint incorrect-shift --only-lint unchecked-call`.
    /// Empty means all rules.
    #[serde(default)]
    pub only: Vec<String>,
    /// Lint rule names to exclude from diagnostics (post-hoc filtering).
    /// These are filtered after `forge lint` runs.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for LintSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Vec::new(),
            only: Vec::new(),
            exclude: Vec::new(),
        }
    }
}

/// File operation feature settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOperationsSettings {
    /// Auto-generate Solidity scaffold when creating a new `.sol` file.
    #[serde(default = "default_true")]
    pub template_on_create: bool,
    /// Auto-update Solidity imports during `workspace/willRenameFiles`.
    #[serde(default = "default_true")]
    pub update_imports_on_rename: bool,
    /// Auto-update Solidity imports during `workspace/willDeleteFiles`.
    #[serde(default = "default_true")]
    pub update_imports_on_delete: bool,
}

impl Default for FileOperationsSettings {
    fn default() -> Self {
        Self {
            template_on_create: true,
            update_imports_on_rename: true,
            update_imports_on_delete: true,
        }
    }
}

/// Project indexing feature settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndexSettings {
    /// If true, run a full-project index scan at startup / first successful build.
    /// If false, skip eager full-project scanning for faster startup.
    #[serde(default)]
    pub full_project_scan: bool,
    /// Persistent reference cache mode:
    /// - `v2` (default): per-file shard cache
    /// - `auto`: alias to v2 for compatibility
    #[serde(default)]
    pub cache_mode: ProjectIndexCacheMode,
    /// If true, use an aggressive scoped reindex strategy on dirty sync:
    /// recompile only reverse-import affected files from recent changes.
    /// Falls back to full-project reindex on failure.
    #[serde(default)]
    pub incremental_edit_reindex: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectIndexCacheMode {
    Auto,
    #[default]
    V2,
}

impl Default for ProjectIndexSettings {
    fn default() -> Self {
        Self {
            full_project_scan: true,
            cache_mode: ProjectIndexCacheMode::V2,
            incremental_edit_reindex: false,
        }
    }
}

fn default_true() -> bool {
    true
}

fn active_profile_name() -> String {
    std::env::var("FOUNDRY_PROFILE").unwrap_or_else(|_| "default".to_string())
}

fn find_profile_table<'a>(table: &'a toml::Table, profile_name: &str) -> Option<&'a toml::Table> {
    table
        .get("profile")
        .and_then(|p| p.as_table())
        .and_then(|profiles| profiles.get(profile_name))
        .and_then(|p| p.as_table())
}

fn get_profile_value<'a>(
    active_profile: Option<&'a toml::Table>,
    default_profile: Option<&'a toml::Table>,
    key: &str,
) -> Option<&'a toml::Value> {
    active_profile
        .and_then(|p| p.get(key))
        .or_else(|| default_profile.and_then(|p| p.get(key)))
}

fn get_lint_table<'a>(
    active_profile: Option<&'a toml::Table>,
    default_profile: Option<&'a toml::Table>,
) -> Option<&'a toml::Table> {
    active_profile
        .and_then(|p| p.get("lint"))
        .and_then(|l| l.as_table())
        .or_else(|| {
            default_profile
                .and_then(|p| p.get("lint"))
                .and_then(|l| l.as_table())
        })
}

/// Try to parse `Settings` from a `serde_json::Value`.
///
/// Handles both direct settings objects and the wrapped form where the
/// editor nests under `"solidity-language-server"`:
/// ```json
/// { "solidity-language-server": { "inlayHints": { ... } } }
/// ```
pub fn parse_settings(value: &serde_json::Value) -> Settings {
    // Try the wrapped form first
    if let Some(inner) = value.get("solidity-language-server")
        && let Ok(s) = serde_json::from_value::<Settings>(inner.clone())
    {
        return s;
    }
    // Try direct form
    serde_json::from_value::<Settings>(value.clone()).unwrap_or_default()
}

/// Project-level configuration extracted from `foundry.toml`.
///
/// This includes both lint settings and compiler settings needed by the
/// solc runner (solc version, remappings, optimizer, via-IR, EVM version).
#[derive(Debug, Clone)]
pub struct FoundryConfig {
    /// The project root where `foundry.toml` was found.
    pub root: PathBuf,
    /// Solc version from `[profile.default] solc = "0.8.26"`.
    /// `None` means use the system default.
    pub solc_version: Option<String>,
    /// Remappings from `[profile.default] remappings = [...]`.
    /// Empty if not specified (will fall back to `forge remappings`).
    pub remappings: Vec<String>,
    /// Whether to compile via the Yul IR pipeline (`via_ir = true`).
    /// Maps to `"viaIR": true` in the solc standard JSON settings.
    pub via_ir: bool,
    /// Whether the optimizer is enabled (`optimizer = true`).
    pub optimizer: bool,
    /// Number of optimizer runs (`optimizer_runs = 200`).
    /// Only meaningful when `optimizer` is `true`.
    pub optimizer_runs: u64,
    /// Target EVM version (`evm_version = "cancun"`).
    /// Maps to `"evmVersion"` in the solc standard JSON settings.
    /// `None` means use solc's default.
    pub evm_version: Option<String>,
    /// Error codes to suppress from diagnostics (`ignored_error_codes = [2394, 5574]`).
    pub ignored_error_codes: Vec<u64>,
    /// Source directory relative to `root` (default: `src` for Foundry, `contracts` for Hardhat).
    pub sources_dir: String,
    /// Library directories to exclude from project-wide indexing.
    /// Parsed from `libs = ["lib"]` in foundry.toml (default: `["lib"]`).
    pub libs: Vec<String>,
}

impl Default for FoundryConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            solc_version: None,
            remappings: Vec::new(),
            via_ir: false,
            optimizer: false,
            optimizer_runs: 200,
            evm_version: None,
            ignored_error_codes: Vec::new(),
            sources_dir: "src".to_string(),
            libs: vec!["lib".to_string()],
        }
    }
}

/// Load project configuration from the nearest `foundry.toml`.
///
/// When no `foundry.toml` is found, returns a default config with `root` set
/// to the nearest git root or the file's parent directory.  This ensures that
/// bare Solidity projects (Hardhat, node_modules, loose files) still get a
/// usable project root for solc invocation.
pub fn load_foundry_config(file_path: &Path) -> FoundryConfig {
    let toml_path = match find_foundry_toml(file_path) {
        Some(p) => p,
        None => {
            let start = if file_path.is_file() {
                file_path.parent().unwrap_or(file_path)
            } else {
                file_path
            };
            let root = find_git_root(start).unwrap_or_else(|| start.to_path_buf());
            return FoundryConfig {
                root,
                ..Default::default()
            };
        }
    };
    load_foundry_config_from_toml(&toml_path)
}

/// Load project configuration from a known `foundry.toml` path.
pub fn load_foundry_config_from_toml(toml_path: &Path) -> FoundryConfig {
    load_foundry_config_from_toml_with_profile_name(toml_path, &active_profile_name())
}

fn load_foundry_config_from_toml_with_profile_name(
    toml_path: &Path,
    profile_name: &str,
) -> FoundryConfig {
    let root = toml_path.parent().unwrap_or(Path::new("")).to_path_buf();

    let content = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(_) => {
            return FoundryConfig {
                root,
                ..Default::default()
            };
        }
    };

    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => {
            return FoundryConfig {
                root,
                ..Default::default()
            };
        }
    };

    let active_profile = find_profile_table(&table, profile_name);
    let default_profile = find_profile_table(&table, "default");
    if active_profile.is_none() && default_profile.is_none() {
        return FoundryConfig {
            root,
            ..Default::default()
        };
    }

    // Parse solc version: `solc = "0.8.26"` or `solc_version = "0.8.26"`
    let solc_version = get_profile_value(active_profile, default_profile, "solc")
        .or_else(|| get_profile_value(active_profile, default_profile, "solc_version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse remappings: `remappings = ["ds-test/=lib/...", ...]`
    let remappings = get_profile_value(active_profile, default_profile, "remappings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    // Parse via_ir: `via_ir = true`
    let via_ir = get_profile_value(active_profile, default_profile, "via_ir")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Parse optimizer: `optimizer = true`
    let optimizer = get_profile_value(active_profile, default_profile, "optimizer")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Parse optimizer_runs: `optimizer_runs = 200`
    let optimizer_runs = get_profile_value(active_profile, default_profile, "optimizer_runs")
        .and_then(|v| v.as_integer())
        .map(|v| v as u64)
        .unwrap_or(200);

    // Parse evm_version: `evm_version = "cancun"` or `evm_version = "osaka"`
    let evm_version = get_profile_value(active_profile, default_profile, "evm_version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse src: `src = "contracts"` (default: "src")
    let sources_dir = get_profile_value(active_profile, default_profile, "src")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "src".to_string());

    // Parse libs: `libs = ["lib", "node_modules"]` (default: ["lib"])
    let libs = get_profile_value(active_profile, default_profile, "libs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["lib".to_string()]);

    // Parse ignored_error_codes: `ignored_error_codes = [2394, 6321, 3860, 5574]`
    let ignored_error_codes =
        get_profile_value(active_profile, default_profile, "ignored_error_codes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_integer())
                    .map(|v| v as u64)
                    .collect()
            })
            .unwrap_or_default();

    FoundryConfig {
        root,
        solc_version,
        remappings,
        via_ir,
        optimizer,
        optimizer_runs,
        evm_version,
        ignored_error_codes,
        sources_dir,
        libs,
    }
}

/// Lint-related configuration extracted from `foundry.toml`.
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// The project root where `foundry.toml` was found.
    pub root: PathBuf,
    /// Whether linting is enabled on build (default: true).
    pub lint_on_build: bool,
    /// Compiled glob patterns from the `ignore` list.
    pub ignore_patterns: Vec<glob::Pattern>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            lint_on_build: true,
            ignore_patterns: Vec::new(),
        }
    }
}

impl LintConfig {
    /// Returns `true` if the given file should be linted.
    ///
    /// A file is skipped when:
    /// - `lint_on_build` is `false`, or
    /// - the file's path (relative to the project root) matches any `ignore` pattern.
    pub fn should_lint(&self, file_path: &Path) -> bool {
        if !self.lint_on_build {
            return false;
        }

        if self.ignore_patterns.is_empty() {
            return true;
        }

        // Build a relative path from the project root so that patterns like
        // "test/**/*" work correctly.
        let relative = file_path.strip_prefix(&self.root).unwrap_or(file_path);

        let rel_str = relative.to_string_lossy();

        for pattern in &self.ignore_patterns {
            if pattern.matches(&rel_str) {
                return false;
            }
        }

        true
    }
}

/// Returns the root of the git repository containing `start`, if any.
///
/// This mirrors foundry's own `find_git_root` behavior: walk up ancestors
/// until a directory containing `.git` is found.
fn find_git_root(start: &Path) -> Option<PathBuf> {
    let start = if start.is_file() {
        start.parent()?
    } else {
        start
    };
    start
        .ancestors()
        .find(|p| p.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Walk up from `start` to find the nearest `foundry.toml`, stopping at the
/// git repository root (consistent with foundry's `find_project_root`).
///
/// See: <https://github.com/foundry-rs/foundry/blob/5389caefb5bfb035c547dffb4fd0f441a37e5371/crates/config/src/utils.rs#L62>
pub fn find_foundry_toml(start: &Path) -> Option<PathBuf> {
    let start_dir = if start.is_file() {
        start.parent()?
    } else {
        start
    };

    let boundary = find_git_root(start_dir);

    start_dir
        .ancestors()
        // Don't look outside of the git repo, matching foundry's behavior.
        .take_while(|p| {
            if let Some(boundary) = &boundary {
                p.starts_with(boundary)
            } else {
                true
            }
        })
        .find(|p| p.join("foundry.toml").is_file())
        .map(|p| p.join("foundry.toml"))
}

/// Load the lint configuration from the nearest `foundry.toml` relative to
/// `file_path`. Returns `LintConfig::default()` when no config is found or
/// the relevant sections are absent.
pub fn load_lint_config(file_path: &Path) -> LintConfig {
    let toml_path = match find_foundry_toml(file_path) {
        Some(p) => p,
        None => return LintConfig::default(),
    };
    load_lint_config_from_toml(&toml_path)
}

/// Load lint config from a known `foundry.toml` path (used when reloading
/// after a file-watch notification).
pub fn load_lint_config_from_toml(toml_path: &Path) -> LintConfig {
    load_lint_config_from_toml_with_profile_name(toml_path, &active_profile_name())
}

fn load_lint_config_from_toml_with_profile_name(
    toml_path: &Path,
    profile_name: &str,
) -> LintConfig {
    let root = toml_path.parent().unwrap_or(Path::new("")).to_path_buf();

    let content = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(_) => {
            return LintConfig {
                root,
                ..Default::default()
            };
        }
    };

    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => {
            return LintConfig {
                root,
                ..Default::default()
            };
        }
    };

    let active_profile = find_profile_table(&table, profile_name);
    let default_profile = find_profile_table(&table, "default");
    let lint_table = get_lint_table(active_profile, default_profile);

    let lint_table = match lint_table {
        Some(t) => t,
        None => {
            return LintConfig {
                root,
                ..Default::default()
            };
        }
    };

    let lint_on_build = lint_table
        .get("lint_on_build")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let ignore_patterns = lint_table
        .get("ignore")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| glob::Pattern::new(s).ok())
                .collect()
        })
        .unwrap_or_default();

    LintConfig {
        root,
        lint_on_build,
        ignore_patterns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config_lints_everything() {
        let config = LintConfig::default();
        assert!(config.should_lint(Path::new("test/MyTest.sol")));
        assert!(config.should_lint(Path::new("src/Token.sol")));
    }

    #[test]
    fn test_lint_on_build_false_skips_all() {
        let config = LintConfig {
            lint_on_build: false,
            ..Default::default()
        };
        assert!(!config.should_lint(Path::new("src/Token.sol")));
    }

    #[test]
    fn test_ignore_pattern_matches() {
        let config = LintConfig {
            root: PathBuf::from("/project"),
            lint_on_build: true,
            ignore_patterns: vec![glob::Pattern::new("test/**/*").unwrap()],
        };
        assert!(!config.should_lint(Path::new("/project/test/MyTest.sol")));
        assert!(config.should_lint(Path::new("/project/src/Token.sol")));
    }

    #[test]
    fn test_multiple_ignore_patterns() {
        let config = LintConfig {
            root: PathBuf::from("/project"),
            lint_on_build: true,
            ignore_patterns: vec![
                glob::Pattern::new("test/**/*").unwrap(),
                glob::Pattern::new("script/**/*").unwrap(),
            ],
        };
        assert!(!config.should_lint(Path::new("/project/test/MyTest.sol")));
        assert!(!config.should_lint(Path::new("/project/script/Deploy.sol")));
        assert!(config.should_lint(Path::new("/project/src/Token.sol")));
    }

    #[test]
    fn test_load_lint_config_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default.lint]
ignore = ["test/**/*"]
lint_on_build = true
"#,
        )
        .unwrap();

        let config = load_lint_config_from_toml(&toml_path);
        assert!(config.lint_on_build);
        assert_eq!(config.ignore_patterns.len(), 1);
        assert!(!config.should_lint(&dir.path().join("test/MyTest.sol")));
        assert!(config.should_lint(&dir.path().join("src/Token.sol")));
    }

    #[test]
    fn test_load_lint_config_lint_on_build_false() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default.lint]
lint_on_build = false
"#,
        )
        .unwrap();

        let config = load_lint_config_from_toml(&toml_path);
        assert!(!config.lint_on_build);
        assert!(!config.should_lint(&dir.path().join("src/Token.sol")));
    }

    #[test]
    fn test_load_lint_config_no_lint_section() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
src = "src"
"#,
        )
        .unwrap();

        let config = load_lint_config_from_toml(&toml_path);
        assert!(config.lint_on_build);
        assert!(config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_load_lint_config_falls_back_to_default_lint_section() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default.lint]
ignore = ["test/**/*"]
lint_on_build = false

[profile.local]
src = "src"
"#,
        )
        .unwrap();

        let config = load_lint_config_from_toml_with_profile_name(&toml_path, "local");
        assert!(!config.lint_on_build);
        assert_eq!(config.ignore_patterns.len(), 1);
        assert!(!config.should_lint(&dir.path().join("test/MyTest.sol")));
    }

    #[test]
    fn test_find_foundry_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(&toml_path, "[profile.default]").unwrap();

        // Create a nested directory
        let nested = dir.path().join("src");
        fs::create_dir_all(&nested).unwrap();

        let found = find_foundry_toml(&nested);
        assert_eq!(found, Some(toml_path));
    }

    #[test]
    fn test_load_lint_config_walks_ancestors() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default.lint]
ignore = ["test/**/*"]
"#,
        )
        .unwrap();

        let nested_file = dir.path().join("src/Token.sol");
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(&nested_file, "// solidity").unwrap();

        let config = load_lint_config(&nested_file);
        assert_eq!(config.root, dir.path());
        assert_eq!(config.ignore_patterns.len(), 1);
    }

    #[test]
    fn test_find_git_root() {
        let dir = tempfile::tempdir().unwrap();
        // Create a fake .git directory
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        let nested = dir.path().join("sub/deep");
        fs::create_dir_all(&nested).unwrap();

        let root = find_git_root(&nested);
        assert_eq!(root, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn test_find_foundry_toml_stops_at_git_boundary() {
        // Layout:
        //   tmp/
        //     foundry.toml          <-- outside git repo, should NOT be found
        //     repo/
        //       .git/
        //       sub/
        //         [search starts here]
        let dir = tempfile::tempdir().unwrap();

        // foundry.toml outside the git repo
        fs::write(dir.path().join("foundry.toml"), "[profile.default]").unwrap();

        // git repo with no foundry.toml
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("sub")).unwrap();

        let found = find_foundry_toml(&repo.join("sub"));
        // Should NOT find the foundry.toml above the .git boundary
        assert_eq!(found, None);
    }

    #[test]
    fn test_find_foundry_toml_within_git_boundary() {
        // Layout:
        //   tmp/
        //     repo/
        //       .git/
        //       foundry.toml        <-- inside git repo, should be found
        //       src/
        //         [search starts here]
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        let toml_path = repo.join("foundry.toml");
        fs::write(&toml_path, "[profile.default]").unwrap();

        let found = find_foundry_toml(&repo.join("src"));
        assert_eq!(found, Some(toml_path));
    }

    #[test]
    fn test_find_foundry_toml_no_git_repo_still_walks_up() {
        // When there's no .git directory at all, the search should still
        // walk up (unbounded), matching foundry's behavior.
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(&toml_path, "[profile.default]").unwrap();

        let nested = dir.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();

        let found = find_foundry_toml(&nested);
        assert_eq!(found, Some(toml_path));
    }

    // ── Compiler settings parsing ─────────────────────────────────────

    #[test]
    fn test_load_foundry_config_compiler_settings() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
src = "src"
solc = '0.8.33'
optimizer = true
optimizer_runs = 9999999
via_ir = true
evm_version = 'osaka'
ignored_error_codes = [2394, 6321, 3860, 5574, 2424, 8429, 4591]
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml(&toml_path);
        assert_eq!(config.solc_version, Some("0.8.33".to_string()));
        assert!(config.optimizer);
        assert_eq!(config.optimizer_runs, 9999999);
        assert!(config.via_ir);
        assert_eq!(config.evm_version, Some("osaka".to_string()));
        assert_eq!(
            config.ignored_error_codes,
            vec![2394, 6321, 3860, 5574, 2424, 8429, 4591]
        );
    }

    #[test]
    fn test_load_foundry_config_defaults_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
src = "src"
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml(&toml_path);
        assert_eq!(config.solc_version, None);
        assert!(!config.optimizer);
        assert_eq!(config.optimizer_runs, 200);
        assert!(!config.via_ir);
        assert_eq!(config.evm_version, None);
        assert!(config.ignored_error_codes.is_empty());
        assert_eq!(config.libs, vec!["lib".to_string()]);
    }

    #[test]
    fn test_load_foundry_config_partial_settings() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
via_ir = true
evm_version = "cancun"
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml(&toml_path);
        assert!(config.via_ir);
        assert!(!config.optimizer); // default false
        assert_eq!(config.optimizer_runs, 200); // default
        assert_eq!(config.evm_version, Some("cancun".to_string()));
        assert!(config.ignored_error_codes.is_empty());
    }

    #[test]
    fn test_load_foundry_config_libs() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
libs = ["lib", "node_modules", "dependencies"]
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml(&toml_path);
        assert_eq!(
            config.libs,
            vec![
                "lib".to_string(),
                "node_modules".to_string(),
                "dependencies".to_string()
            ]
        );
    }

    #[test]
    fn test_load_foundry_config_libs_defaults_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
src = "src"
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml(&toml_path);
        assert_eq!(config.libs, vec!["lib".to_string()]);
    }

    #[test]
    fn test_load_foundry_config_falls_back_to_default_profile_values() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("foundry.toml");
        fs::write(
            &toml_path,
            r#"
[profile.default]
solc = "0.8.33"
optimizer_runs = 1234
libs = ["lib", "node_modules"]

[profile.local]
src = "contracts"
"#,
        )
        .unwrap();

        let config = load_foundry_config_from_toml_with_profile_name(&toml_path, "local");
        assert_eq!(config.solc_version, Some("0.8.33".to_string()));
        assert_eq!(config.optimizer_runs, 1234);
        assert_eq!(
            config.libs,
            vec!["lib".to_string(), "node_modules".to_string()]
        );
        assert_eq!(config.sources_dir, "contracts".to_string());
    }

    // ── Settings parsing ──────────────────────────────────────────────

    #[test]
    fn test_parse_settings_defaults() {
        let value = serde_json::json!({});
        let s = parse_settings(&value);
        assert!(s.inlay_hints.parameters);
        assert!(s.inlay_hints.gas_estimates);
        assert!(s.lint.enabled);
        assert!(s.file_operations.template_on_create);
        assert!(s.file_operations.update_imports_on_rename);
        assert!(s.file_operations.update_imports_on_delete);
        assert!(s.project_index.full_project_scan);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(!s.project_index.incremental_edit_reindex);
        assert!(s.lint.severity.is_empty());
        assert!(s.lint.only.is_empty());
        assert!(s.lint.exclude.is_empty());
    }

    #[test]
    fn test_parse_settings_wrapped() {
        let value = serde_json::json!({
            "solidity-language-server": {
                "inlayHints": { "parameters": false, "gasEstimates": false },
                "lint": {
                    "enabled": true,
                    "severity": ["high", "med"],
                    "only": ["incorrect-shift"],
                    "exclude": ["pascal-case-struct", "mixed-case-variable"]
                },
                "fileOperations": {
                    "templateOnCreate": false,
                    "updateImportsOnRename": false,
                    "updateImportsOnDelete": false
                },
                "projectIndex": {
                    "fullProjectScan": true,
                    "cacheMode": "v2",
                    "incrementalEditReindex": true
                },
            }
        });
        let s = parse_settings(&value);
        assert!(!s.inlay_hints.parameters);
        assert!(!s.inlay_hints.gas_estimates);
        assert!(s.lint.enabled);
        assert!(!s.file_operations.template_on_create);
        assert!(!s.file_operations.update_imports_on_rename);
        assert!(!s.file_operations.update_imports_on_delete);
        assert!(s.project_index.full_project_scan);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(s.project_index.incremental_edit_reindex);
        assert_eq!(s.lint.severity, vec!["high", "med"]);
        assert_eq!(s.lint.only, vec!["incorrect-shift"]);
        assert_eq!(
            s.lint.exclude,
            vec!["pascal-case-struct", "mixed-case-variable"]
        );
    }

    #[test]
    fn test_parse_settings_direct() {
        let value = serde_json::json!({
            "inlayHints": { "parameters": false },
            "lint": { "enabled": false },
            "fileOperations": {
                "templateOnCreate": false,
                "updateImportsOnRename": false,
                "updateImportsOnDelete": false
            },
            "projectIndex": {
                "fullProjectScan": true,
                "cacheMode": "v2",
                "incrementalEditReindex": true
            }
        });
        let s = parse_settings(&value);
        assert!(!s.inlay_hints.parameters);
        assert!(!s.lint.enabled);
        assert!(!s.file_operations.template_on_create);
        assert!(!s.file_operations.update_imports_on_rename);
        assert!(!s.file_operations.update_imports_on_delete);
        assert!(s.project_index.full_project_scan);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(s.project_index.incremental_edit_reindex);
    }

    #[test]
    fn test_parse_settings_partial() {
        let value = serde_json::json!({
            "solidity-language-server": {
                "lint": { "exclude": ["unused-import"] }
            }
        });
        let s = parse_settings(&value);
        // inlayHints not specified → defaults to true
        assert!(s.inlay_hints.parameters);
        assert!(s.inlay_hints.gas_estimates);
        // lint.enabled not specified → defaults to true
        assert!(s.lint.enabled);
        assert!(s.file_operations.template_on_create);
        assert!(s.file_operations.update_imports_on_rename);
        assert!(s.file_operations.update_imports_on_delete);
        assert!(s.project_index.full_project_scan);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(!s.project_index.incremental_edit_reindex);
        assert!(s.lint.severity.is_empty());
        assert!(s.lint.only.is_empty());
        assert_eq!(s.lint.exclude, vec!["unused-import"]);
    }

    #[test]
    fn test_parse_settings_empty_wrapped() {
        let value = serde_json::json!({
            "solidity-language-server": {}
        });
        let s = parse_settings(&value);
        assert!(s.inlay_hints.parameters);
        assert!(s.inlay_hints.gas_estimates);
        assert!(s.lint.enabled);
        assert!(s.file_operations.template_on_create);
        assert!(s.file_operations.update_imports_on_rename);
        assert!(s.file_operations.update_imports_on_delete);
        assert!(s.project_index.full_project_scan);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(!s.project_index.incremental_edit_reindex);
        assert!(s.lint.severity.is_empty());
        assert!(s.lint.only.is_empty());
        assert!(s.lint.exclude.is_empty());
    }

    #[test]
    fn test_parse_settings_project_index_cache_mode_defaults_on_invalid() {
        let value = serde_json::json!({
            "solidity-language-server": {
                "projectIndex": {
                    "cacheMode": "bad-mode"
                }
            }
        });
        let s = parse_settings(&value);
        assert_eq!(s.project_index.cache_mode, ProjectIndexCacheMode::V2);
        assert!(!s.project_index.incremental_edit_reindex);
    }

    #[test]
    fn test_parse_settings_severity_only() {
        let value = serde_json::json!({
            "solidity-language-server": {
                "lint": {
                    "severity": ["high", "gas"],
                    "only": ["incorrect-shift", "asm-keccak256"]
                }
            }
        });
        let s = parse_settings(&value);
        assert_eq!(s.lint.severity, vec!["high", "gas"]);
        assert_eq!(s.lint.only, vec!["incorrect-shift", "asm-keccak256"]);
        assert!(s.lint.exclude.is_empty());
    }
}
