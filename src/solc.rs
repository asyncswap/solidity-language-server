//! Direct `solc --standard-json` runner for fast AST generation.
//!
//! The output is normalized into the same shape that `forge build --json --ast`
//! produces, so all downstream consumers (goto, hover, completions, etc.) work
//! unchanged.

use crate::config::FoundryConfig;
use crate::links;
use crate::runner::RunnerError;
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tokio::process::Command;
use tower_lsp::lsp_types::Url;

/// Cached list of installed solc versions. Populated on first access,
/// invalidated after a successful `svm::install`.
static INSTALLED_VERSIONS: OnceLock<Mutex<Vec<SemVer>>> = OnceLock::new();

fn get_installed_versions() -> Vec<SemVer> {
    let mutex = INSTALLED_VERSIONS.get_or_init(|| Mutex::new(scan_installed_versions()));
    mutex.lock().unwrap().clone()
}

fn invalidate_installed_versions() {
    if let Some(mutex) = INSTALLED_VERSIONS.get() {
        *mutex.lock().unwrap() = scan_installed_versions();
    }
}

/// Convert a `semver::Version` (from svm-rs) to our lightweight `SemVer`.
fn semver_to_local(v: &semver::Version) -> SemVer {
    SemVer {
        major: v.major as u32,
        minor: v.minor as u32,
        patch: v.patch as u32,
    }
}

/// Resolve the path to the solc binary.
///
/// Resolution order:
/// 1. Parse `pragma solidity` from the source file.
///    - **Exact pragma** (`=0.7.6`): always use the file's version — foundry.toml
///      cannot override an exact pragma without breaking compilation.
///    - **Wildcard pragma** (`^0.8.0`, `>=0.8.0`, `>=0.6.2 <0.9.0`): if
///      `foundry.toml` specifies a solc version that satisfies the constraint,
///      use it. Otherwise pick the latest matching installed version.
/// 2. If no pragma, use the `foundry.toml` solc version if set.
/// 3. If no match is installed, auto-install via `svm install`.
/// 4. Fall back to whatever `solc` is on `$PATH`.
pub async fn resolve_solc_binary(
    config: &FoundryConfig,
    constraint: Option<&PragmaConstraint>,
    client: Option<&tower_lsp::Client>,
) -> PathBuf {
    // 1. Try pragma constraint (may be tightened from the full import graph)
    if let Some(constraint) = constraint {
        // For exact pragmas, always honour the file — foundry.toml can't override
        // without causing a compilation failure.
        // For wildcard pragmas, prefer the foundry.toml version if it satisfies
        // the constraint. This mirrors `forge build` behaviour where the project
        // config picks the version but the pragma must still be satisfied.
        if !matches!(constraint, PragmaConstraint::Exact(_))
            && let Some(ref config_ver) = config.solc_version
            && let Some(parsed) = SemVer::parse(config_ver)
            && version_satisfies(&parsed, constraint)
            && let Some(path) = find_solc_binary(config_ver)
        {
            if let Some(c) = client {
                c.log_message(
                    tower_lsp::lsp_types::MessageType::INFO,
                    format!("using solc {config_ver} (pragma {constraint})"),
                )
                .await;
            }
            return path;
        }

        let installed = get_installed_versions();
        if let Some(version) = find_matching_version(constraint, &installed)
            && let Some(path) = find_solc_binary(&version.to_string())
        {
            if let Some(c) = client {
                c.log_message(
                    tower_lsp::lsp_types::MessageType::INFO,
                    format!("using solc {version}"),
                )
                .await;
            }
            return path;
        }

        // No matching version installed — try auto-install via svm
        let install_version = version_to_install(constraint);
        if let Some(ref ver_str) = install_version {
            if let Some(c) = client {
                c.show_message(
                    tower_lsp::lsp_types::MessageType::INFO,
                    format!("Installing solc {ver_str}..."),
                )
                .await;
            }

            if svm_install(ver_str).await {
                // Refresh the cached version list after install
                invalidate_installed_versions();

                if let Some(c) = client {
                    c.show_message(
                        tower_lsp::lsp_types::MessageType::INFO,
                        format!("Installed solc {ver_str}"),
                    )
                    .await;
                }
                if let Some(path) = find_solc_binary(ver_str) {
                    return path;
                }
            } else if let Some(c) = client {
                c.show_message(
                    tower_lsp::lsp_types::MessageType::WARNING,
                    format!(
                        "Failed to install solc {ver_str}. \
                             Install it manually: svm install {ver_str}"
                    ),
                )
                .await;
            }
        }
    }

    // 2. No pragma — use foundry.toml version if available
    if let Some(ref version) = config.solc_version
        && let Some(path) = find_solc_binary(version)
    {
        if let Some(c) = client {
            c.log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!(
                    "solc: no pragma, using foundry.toml version {version} → {}",
                    path.display()
                ),
            )
            .await;
        }
        return path;
    }

    // 3. Fall back to system solc
    if let Some(c) = client {
        c.log_message(
            tower_lsp::lsp_types::MessageType::INFO,
            "solc: no pragma match, falling back to system solc",
        )
        .await;
    }
    PathBuf::from("solc")
}

/// Determine which version to install for a pragma constraint.
///
/// - Exact: install that version
/// - Caret `^0.8.20`: install `0.8.20` (minimum satisfying)
/// - Gte `>=0.8.0`: install `0.8.0` (minimum satisfying)
/// - Range `>=0.6.2 <0.9.0`: install `0.6.2` (minimum satisfying)
fn version_to_install(constraint: &PragmaConstraint) -> Option<String> {
    match constraint {
        PragmaConstraint::Exact(v) => Some(v.to_string()),
        PragmaConstraint::Caret(v) => Some(v.to_string()),
        PragmaConstraint::Gte(v) => Some(v.to_string()),
        PragmaConstraint::Range(lower, _) => Some(lower.to_string()),
    }
}

/// Install a solc version using svm-rs library.
///
/// Returns `true` if the install succeeded.
async fn svm_install(version: &str) -> bool {
    let ver = match semver::Version::parse(version) {
        Ok(v) => v,
        Err(_) => return false,
    };
    svm::install(&ver).await.is_ok()
}

/// Look up a solc binary by version string using `svm::version_binary()`.
fn find_solc_binary(version: &str) -> Option<PathBuf> {
    let path = svm::version_binary(version);
    if path.is_file() {
        return Some(path);
    }
    None
}

// ── Pragma parsing ────────────────────────────────────────────────────────

/// A parsed semver version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    fn parse(s: &str) -> Option<SemVer> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(SemVer {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A version constraint from `pragma solidity`.
#[derive(Debug, Clone, PartialEq)]
pub enum PragmaConstraint {
    /// `0.8.26` — exact match
    Exact(SemVer),
    /// `^0.8.0` — same major.minor, patch >= specified
    /// Actually in Solidity: `^0.8.0` means `>=0.8.0 <0.9.0`
    Caret(SemVer),
    /// `>=0.8.0` — at least this version
    Gte(SemVer),
    /// `>=0.6.2 <0.9.0` — range
    Range(SemVer, SemVer),
}

impl std::fmt::Display for PragmaConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PragmaConstraint::Exact(v) => write!(f, "={v}"),
            PragmaConstraint::Caret(v) => write!(f, "^{v}"),
            PragmaConstraint::Gte(v) => write!(f, ">={v}"),
            PragmaConstraint::Range(lo, hi) => write!(f, ">={lo} <{hi}"),
        }
    }
}

/// Resolve a Solidity import path to an absolute filesystem path.
///
/// Handles relative imports (`./`, `../`) and remapped imports.
fn resolve_import_to_abs(
    project_root: &Path,
    importer_abs: &Path,
    import_path: &str,
    remappings: &[String],
) -> Option<PathBuf> {
    if import_path.starts_with("./") || import_path.starts_with("../") {
        let base = importer_abs.parent()?;
        return Some(lexical_normalize(&base.join(import_path)));
    }

    for remap in remappings {
        let mut it = remap.splitn(2, '=');
        let prefix = it.next().unwrap_or_default();
        let target = it.next().unwrap_or_default();
        if prefix.is_empty() || target.is_empty() {
            continue;
        }
        if import_path.starts_with(prefix) {
            let suffix = import_path.strip_prefix(prefix).unwrap_or_default();
            return Some(lexical_normalize(
                &project_root.join(format!("{target}{suffix}")),
            ));
        }
    }

    Some(lexical_normalize(&project_root.join(import_path)))
}

/// Normalize a path by resolving `.` and `..` components lexically
/// (without hitting the filesystem).
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            _ => out.push(comp.as_os_str()),
        }
    }
    out
}

/// Collect pragma constraints from a file and all its transitive imports.
///
/// Walks the import graph using simple string scanning (no tree-sitter),
/// resolving import paths via remappings.  Returns all pragmas found so
/// that the caller can pick a solc version satisfying every file.
fn collect_import_pragmas(
    file_path: &Path,
    project_root: &Path,
    remappings: &[String],
) -> Vec<PragmaConstraint> {
    let mut pragmas = Vec::new();
    let mut visited = HashSet::new();
    collect_import_pragmas_recursive(
        file_path,
        project_root,
        remappings,
        &mut pragmas,
        &mut visited,
    );
    pragmas
}

fn collect_import_pragmas_recursive(
    file_path: &Path,
    project_root: &Path,
    remappings: &[String],
    pragmas: &mut Vec<PragmaConstraint>,
    visited: &mut HashSet<PathBuf>,
) {
    if !visited.insert(file_path.to_path_buf()) {
        return;
    }
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(_) => return,
    };
    if let Some(pragma) = parse_pragma(&source) {
        pragmas.push(pragma);
    }
    for imp in links::ts_find_imports(source.as_bytes()) {
        if let Some(abs) = resolve_import_to_abs(project_root, file_path, &imp.path, remappings) {
            collect_import_pragmas_recursive(&abs, project_root, remappings, pragmas, visited);
        }
    }
}

/// Tighten a set of pragma constraints into a single constraint that
/// satisfies all of them.
///
/// Rules:
/// - An exact pragma always wins (if any file requires `0.8.23`, we must
///   use exactly `0.8.23`).
/// - Multiple exact pragmas that disagree → returns the first one (solc
///   will error anyway, but we still try).
/// - For wildcard pragmas, compute the intersection range and return it.
fn tightest_constraint(pragmas: &[PragmaConstraint]) -> Option<PragmaConstraint> {
    if pragmas.is_empty() {
        return None;
    }

    // If any pragma is Exact, that version must be used.
    for p in pragmas {
        if matches!(p, PragmaConstraint::Exact(_)) {
            return Some(p.clone());
        }
    }

    // Normalize every constraint to a (lower, upper) range, then intersect.
    let mut lower = SemVer {
        major: 0,
        minor: 0,
        patch: 0,
    };
    let mut upper: Option<SemVer> = None;

    for p in pragmas {
        let (lo, hi) = constraint_to_range(p);
        if lo > lower {
            lower = lo;
        }
        if let Some(hi) = hi {
            upper = Some(match upper {
                Some(cur) if hi < cur => hi,
                Some(cur) => cur,
                None => hi,
            });
        }
    }

    match upper {
        Some(hi) if lower >= hi => None, // empty intersection
        Some(hi) => Some(PragmaConstraint::Range(lower, hi)),
        None => Some(PragmaConstraint::Gte(lower)),
    }
}

/// Convert a pragma constraint to an inclusive lower bound and optional
/// exclusive upper bound.
fn constraint_to_range(constraint: &PragmaConstraint) -> (SemVer, Option<SemVer>) {
    match constraint {
        PragmaConstraint::Exact(v) => (
            v.clone(),
            Some(SemVer {
                major: v.major,
                minor: v.minor,
                patch: v.patch + 1,
            }),
        ),
        PragmaConstraint::Caret(v) => (
            v.clone(),
            Some(SemVer {
                major: v.major,
                minor: v.minor + 1,
                patch: 0,
            }),
        ),
        PragmaConstraint::Gte(v) => (v.clone(), None),
        PragmaConstraint::Range(lo, hi) => (lo.clone(), Some(hi.clone())),
    }
}

/// Parse `pragma solidity <constraint>;` from Solidity source.
///
/// Handles:
/// - `pragma solidity 0.8.26;` → Exact
/// - `pragma solidity ^0.8.0;` → Caret
/// - `pragma solidity >=0.8.0;` → Gte
/// - `pragma solidity >=0.6.2 <0.9.0;` → Range
pub fn parse_pragma(source: &str) -> Option<PragmaConstraint> {
    // Find the pragma line — only scan the first ~20 lines for performance
    let pragma_line = source
        .lines()
        .take(20)
        .find(|line| line.trim_start().starts_with("pragma solidity"))?;

    // Extract the constraint string between "pragma solidity" and ";"
    let after_keyword = pragma_line
        .trim_start()
        .strip_prefix("pragma solidity")?
        .trim();
    let constraint_str = after_keyword
        .strip_suffix(';')
        .unwrap_or(after_keyword)
        .trim();

    if constraint_str.is_empty() {
        return None;
    }

    // Range: >=X.Y.Z <A.B.C
    if let Some(rest) = constraint_str.strip_prefix(">=") {
        let rest = rest.trim();
        if let Some(space_idx) = rest.find(|c: char| c.is_whitespace() || c == '<') {
            let lower_str = rest[..space_idx].trim();
            let upper_part = rest[space_idx..].trim();
            if let Some(upper_str) = upper_part.strip_prefix('<') {
                let upper_str = upper_str.trim();
                if let (Some(lower), Some(upper)) =
                    (SemVer::parse(lower_str), SemVer::parse(upper_str))
                {
                    return Some(PragmaConstraint::Range(lower, upper));
                }
            }
        }
        // Just >=X.Y.Z
        if let Some(ver) = SemVer::parse(rest) {
            return Some(PragmaConstraint::Gte(ver));
        }
    }

    // Caret: ^X.Y.Z
    if let Some(rest) = constraint_str.strip_prefix('^')
        && let Some(ver) = SemVer::parse(rest.trim())
    {
        return Some(PragmaConstraint::Caret(ver));
    }

    // Exact: X.Y.Z
    if let Some(ver) = SemVer::parse(constraint_str) {
        return Some(PragmaConstraint::Exact(ver));
    }

    None
}

/// List installed solc versions (cached — use `get_installed_versions()` internally).
pub fn list_installed_versions() -> Vec<SemVer> {
    get_installed_versions()
}

/// Scan the filesystem for installed solc versions using `svm::installed_versions()`.
///
/// Returns sorted, deduplicated versions (ascending).
fn scan_installed_versions() -> Vec<SemVer> {
    svm::installed_versions()
        .unwrap_or_default()
        .iter()
        .map(semver_to_local)
        .collect()
}

/// Find the best matching installed version for a pragma constraint.
///
/// For all constraint types, picks the **latest** installed version that
/// satisfies the constraint.
pub fn find_matching_version(
    constraint: &PragmaConstraint,
    installed: &[SemVer],
) -> Option<SemVer> {
    let candidates: Vec<&SemVer> = installed
        .iter()
        .filter(|v| version_satisfies(v, constraint))
        .collect();

    // Pick the latest (last, since installed is sorted ascending)
    candidates.last().cloned().cloned()
}

/// Check if a version satisfies a pragma constraint.
pub fn version_satisfies(version: &SemVer, constraint: &PragmaConstraint) -> bool {
    match constraint {
        PragmaConstraint::Exact(v) => version == v,
        PragmaConstraint::Caret(v) => {
            // Solidity caret: ^0.8.0 means >=0.8.0 <0.9.0
            // i.e. same major, next minor is the ceiling
            version.major == v.major && version >= v && version.minor < v.minor + 1
        }
        PragmaConstraint::Gte(v) => version >= v,
        PragmaConstraint::Range(lower, upper) => version >= lower && version < upper,
    }
}

/// Fetch remappings by running `forge remappings` in the project root.
///
/// Falls back to config remappings, then to an empty list.
pub async fn resolve_remappings(config: &FoundryConfig) -> Vec<String> {
    // Try `forge remappings` first — it merges all sources (foundry.toml,
    // remappings.txt, auto-detected libs).
    let output = Command::new("forge")
        .arg("remappings")
        .current_dir(&config.root)
        .env("FOUNDRY_DISABLE_NIGHTLY_WARNING", "1")
        .output()
        .await;

    if let Ok(output) = output
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let remappings: Vec<String> = stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
        if !remappings.is_empty() {
            return remappings;
        }
    }

    // Fall back to remappings from foundry.toml
    if !config.remappings.is_empty() {
        return config.remappings.clone();
    }

    // Fall back to remappings.txt at project root
    let remappings_txt = config.root.join("remappings.txt");
    if let Ok(content) = std::fs::read_to_string(&remappings_txt) {
        return content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
    }

    Vec::new()
}

/// Build the `--standard-json` input for solc.
///
/// Reads compiler settings from the `FoundryConfig` (parsed from `foundry.toml`)
/// and maps them to the solc standard JSON `settings` object:
///
/// - `via_ir` → `settings.viaIR`
/// - `evm_version` → `settings.evmVersion`
///
/// Note: `optimizer` and `evm.gasEstimates` are intentionally excluded.
/// The optimizer adds ~3s and doesn't affect AST/doc quality.
/// Gas estimates force solc through full EVM codegen — benchmarking on
/// a 510-file project showed 56s with vs 6s without (88% of cost).
pub fn build_standard_json_input(
    file_path: &str,
    remappings: &[String],
    config: &FoundryConfig,
) -> Value {
    let contract_outputs = vec!["devdoc", "userdoc", "evm.methodIdentifiers"];

    let mut settings = json!({
        "remappings": remappings,
        "outputSelection": {
            "*": {
                "*": contract_outputs,
                "": ["ast"]
            }
        }
    });

    if config.via_ir {
        settings["viaIR"] = json!(true);
    }

    // EVM version
    if let Some(ref evm_version) = config.evm_version {
        settings["evmVersion"] = json!(evm_version);
    }

    json!({
        "language": "Solidity",
        "sources": {
            file_path: {
                "urls": [file_path]
            }
        },
        "settings": settings
    })
}

/// Run `solc --standard-json` and return the parsed output.
pub async fn run_solc(
    solc_binary: &Path,
    input: &Value,
    project_root: &Path,
) -> Result<Value, RunnerError> {
    let _ = crate::project_cache::save_last_solc_input(project_root, input);
    let input_str = serde_json::to_string(input)?;

    let mut child = Command::new(solc_binary)
        .arg("--standard-json")
        .current_dir(project_root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write the standard-json input to solc's stdin.
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(input_str.as_bytes())
            .await
            .map_err(RunnerError::CommandError)?;
        // Drop stdin to close it, signaling EOF to solc.
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(RunnerError::CommandError)?;

    // solc writes JSON to stdout even on errors (errors are in the JSON)
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RunnerError::CommandError(std::io::Error::other(format!(
            "solc produced no output, stderr: {stderr}"
        ))));
    }

    let parsed: Value = serde_json::from_str(&stdout)?;
    Ok(parsed)
}

/// Normalize raw solc `--standard-json` output into the canonical shape.
///
/// Solc's native shape is already close to canonical:
/// - `sources[path] = { id, ast }` — kept as-is
/// - `contracts[path][name] = { abi, evm, ... }` — kept as-is
/// - `errors` — kept as-is (defaults to `[]` if absent)
///
/// When `project_root` is provided, relative source paths are resolved to
/// absolute paths so that downstream code (goto, hover, links) can map AST
/// paths back to `file://` URIs. This is necessary because `solc_ast()`
/// passes a relative path to solc (to fix import resolution), and solc then
/// returns relative paths in the AST `absolutePath` and source keys.
///
/// Constructs `source_id_to_path` from source IDs for cross-file resolution.
///
/// Takes ownership and uses `Value::take()` to move AST nodes in-place,
/// avoiding expensive clones of multi-MB AST data.
///
/// Also resolves `absolutePath` on nested `ImportDirective` nodes so that
/// goto-definition on import strings works regardless of CWD.
pub fn normalize_solc_output(mut solc_output: Value, project_root: Option<&Path>) -> Value {
    /// Walk an AST node tree and resolve `absolutePath` on `ImportDirective` nodes.
    fn resolve_import_absolute_paths(node: &mut Value, resolve: &dyn Fn(&str) -> String) {
        let is_import = node.get("nodeType").and_then(|v| v.as_str()) == Some("ImportDirective");

        if is_import {
            if let Some(abs_path) = node.get("absolutePath").and_then(|v| v.as_str()) {
                let resolved = resolve(abs_path);
                node.as_object_mut()
                    .unwrap()
                    .insert("absolutePath".to_string(), json!(resolved));
            }
        }

        // Recurse into "nodes" array (top-level AST children)
        if let Some(nodes) = node.get_mut("nodes").and_then(|v| v.as_array_mut()) {
            for child in nodes {
                resolve_import_absolute_paths(child, resolve);
            }
        }
    }
    let mut result = Map::new();

    // Move errors out (defaults to [] if absent)
    let errors = solc_output
        .get_mut("errors")
        .map(Value::take)
        .unwrap_or_else(|| json!([]));
    result.insert("errors".to_string(), errors);

    // Helper: resolve a path to absolute using the project root.
    // If the path is already absolute or no project root is given, return as-is.
    let resolve = |p: &str| -> String {
        if let Some(root) = project_root {
            let path = Path::new(p);
            if path.is_relative() {
                return root.join(path).to_string_lossy().into_owned();
            }
        }
        p.to_string()
    };

    // Sources: rekey with absolute paths and update AST absolutePath fields.
    // Also build source_id_to_path for cross-file resolution.
    let mut source_id_to_path = Map::new();
    let mut resolved_sources = Map::new();

    if let Some(sources) = solc_output
        .get_mut("sources")
        .and_then(|s| s.as_object_mut())
    {
        // Collect keys first to avoid borrow issues
        let keys: Vec<String> = sources.keys().cloned().collect();
        for key in keys {
            if let Some(mut source_data) = sources.remove(&key) {
                let abs_key = resolve(&key);

                // Update the AST absolutePath field to match, and resolve
                // absolutePath on nested ImportDirective nodes so that
                // goto-definition works regardless of CWD.
                if let Some(ast) = source_data.get_mut("ast") {
                    if let Some(abs_path) = ast.get("absolutePath").and_then(|v| v.as_str()) {
                        let resolved = resolve(abs_path);
                        ast.as_object_mut()
                            .unwrap()
                            .insert("absolutePath".to_string(), json!(resolved));
                    }
                    resolve_import_absolute_paths(ast, &resolve);
                }

                if let Some(id) = source_data.get("id") {
                    source_id_to_path.insert(id.to_string(), json!(&abs_key));
                }

                resolved_sources.insert(abs_key, source_data);
            }
        }
    }

    result.insert("sources".to_string(), Value::Object(resolved_sources));

    // Contracts: rekey with absolute paths
    let mut resolved_contracts = Map::new();
    if let Some(contracts) = solc_output
        .get_mut("contracts")
        .and_then(|c| c.as_object_mut())
    {
        let keys: Vec<String> = contracts.keys().cloned().collect();
        for key in keys {
            if let Some(contract_data) = contracts.remove(&key) {
                resolved_contracts.insert(resolve(&key), contract_data);
            }
        }
    }
    result.insert("contracts".to_string(), Value::Object(resolved_contracts));

    // Construct source_id_to_path for cross-file resolution
    result.insert(
        "source_id_to_path".to_string(),
        Value::Object(source_id_to_path),
    );

    Value::Object(result)
}

/// Normalize forge `build --json --ast` output into the canonical shape.
///
/// Forge wraps data in arrays with metadata:
/// - `sources[path] = [{ source_file: { id, ast }, build_id, profile, version }]`
/// - `contracts[path][name] = [{ contract: { abi, evm, ... }, build_id, profile, version }]`
/// - `build_infos = [{ source_id_to_path: { ... } }]`
///
/// This unwraps to the canonical flat shape:
/// - `sources[path] = { id, ast }`
/// - `contracts[path][name] = { abi, evm, ... }`
/// - `source_id_to_path = { ... }`
pub fn normalize_forge_output(mut forge_output: Value) -> Value {
    let mut result = Map::new();

    // Move errors out
    let errors = forge_output
        .get_mut("errors")
        .map(Value::take)
        .unwrap_or_else(|| json!([]));
    result.insert("errors".to_string(), errors);

    // Unwrap sources: [{ source_file: { id, ast } }] → { id, ast }
    let mut normalized_sources = Map::new();
    if let Some(sources) = forge_output
        .get_mut("sources")
        .and_then(|s| s.as_object_mut())
    {
        for (path, entries) in sources.iter_mut() {
            if let Some(arr) = entries.as_array_mut()
                && let Some(first) = arr.first_mut()
                && let Some(sf) = first.get_mut("source_file")
            {
                normalized_sources.insert(path.clone(), sf.take());
            }
        }
    }
    result.insert("sources".to_string(), Value::Object(normalized_sources));

    // Unwrap contracts: [{ contract: { ... } }] → { ... }
    let mut normalized_contracts = Map::new();
    if let Some(contracts) = forge_output
        .get_mut("contracts")
        .and_then(|c| c.as_object_mut())
    {
        for (path, names) in contracts.iter_mut() {
            let mut path_contracts = Map::new();
            if let Some(names_obj) = names.as_object_mut() {
                for (name, entries) in names_obj.iter_mut() {
                    if let Some(arr) = entries.as_array_mut()
                        && let Some(first) = arr.first_mut()
                        && let Some(contract) = first.get_mut("contract")
                    {
                        path_contracts.insert(name.clone(), contract.take());
                    }
                }
            }
            normalized_contracts.insert(path.clone(), Value::Object(path_contracts));
        }
    }
    result.insert("contracts".to_string(), Value::Object(normalized_contracts));

    // Extract source_id_to_path from build_infos
    let source_id_to_path = forge_output
        .get_mut("build_infos")
        .and_then(|bi| bi.as_array_mut())
        .and_then(|arr| arr.first_mut())
        .and_then(|info| info.get_mut("source_id_to_path"))
        .map(Value::take)
        .unwrap_or_else(|| json!({}));
    result.insert("source_id_to_path".to_string(), source_id_to_path);

    Value::Object(result)
}

/// Run solc for a file and return normalized output.
///
/// This is the main entry point used by the LSP. Reads the file source
/// to detect the pragma version and resolve the correct solc binary.
pub async fn solc_ast(
    file_path: &str,
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
) -> Result<Value, RunnerError> {
    let remappings = resolve_remappings(config).await;

    // Collect pragma constraints from the file and all its transitive imports
    // so we pick a solc version that satisfies the entire dependency graph.
    // This is a synchronous recursive FS crawl — run it on the blocking pool
    // so we don't stall the tokio async runtime on large projects.
    let file_abs = Path::new(file_path).to_path_buf();
    let config_root = config.root.clone();
    let remappings_clone = remappings.clone();
    let pragmas = tokio::task::spawn_blocking(move || {
        collect_import_pragmas(&file_abs, &config_root, &remappings_clone)
    })
    .await
    .unwrap_or_default();
    let constraint = tightest_constraint(&pragmas);
    let solc_binary = resolve_solc_binary(config, constraint.as_ref(), client).await;

    // Solc's import resolver fails when sources use absolute paths — it resolves
    // 0 transitive imports, causing "No matching declaration found" errors for
    // inherited members. Convert to a path relative to the project root so solc
    // can properly resolve `src/`, `lib/`, and remapped imports.
    let rel_path = Path::new(file_path)
        .strip_prefix(&config.root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file_path.to_string());

    let input = build_standard_json_input(&rel_path, &remappings, config);
    let raw_output = run_solc(&solc_binary, &input, &config.root).await?;

    Ok(normalize_solc_output(raw_output, Some(&config.root)))
}

/// Run solc for build diagnostics (same output, just used for error extraction).
pub async fn solc_build(
    file_path: &str,
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
) -> Result<Value, RunnerError> {
    solc_ast(file_path, config, client).await
}

// ── Project-wide indexing ──────────────────────────────────────────────────

/// Discover all Solidity source files under the project root.
///
/// Walks the entire project directory, including `test/`, `script/`, and
/// any other user-authored directories. Only skips:
/// - Directories listed in `config.libs` (default: `["lib"]`)
/// - Directories in `DISCOVER_SKIP_DIRS` (build artifacts)
/// - Hidden directories (starting with `.`)
///
/// Includes `.t.sol` (test) and `.s.sol` (script) files so that
/// find-references and rename work across the full project.
/// Discover the project's own source files by walking only the directories
/// configured in `foundry.toml`: `src`, `test`, and `script`.
///
/// This mirrors how Forge discovers compilable files — it never walks
/// directories outside these three (plus libs).  Stray directories like
/// `certora/` or `hardhat/` are ignored, preventing broken imports from
/// poisoning the solc batch.
pub fn discover_source_files(config: &FoundryConfig) -> Vec<PathBuf> {
    discover_source_files_inner(config, false)
}

/// Discover only the `src` directory files (no test, no script).
///
/// Used as the seed set for phase-1 of two-phase project indexing, where
/// we want to compile only the production source closure first for fast
/// time-to-first-reference.
pub fn discover_src_only_files(config: &FoundryConfig) -> Vec<PathBuf> {
    let root = &config.root;
    if !root.is_dir() {
        return Vec::new();
    }
    let mut files = Vec::new();
    let dir = root.join(&config.sources_dir);
    if dir.is_dir() {
        discover_recursive(&dir, &[], &mut files);
    }
    files.sort();
    files
}

/// Discover the compilation closure seeded only from `src` files.
///
/// Like [`discover_compilation_closure`] but seeds only from
/// [`discover_src_only_files`] instead of all project directories.
/// This produces the minimal set of files needed to compile the
/// production source code, excluding test and script files.
pub fn discover_src_only_closure(config: &FoundryConfig, remappings: &[String]) -> Vec<PathBuf> {
    let seeds = discover_src_only_files(config);
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: std::collections::VecDeque<PathBuf> = seeds.into_iter().collect();

    while let Some(file) = queue.pop_front() {
        if !visited.insert(file.clone()) {
            continue;
        }
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for imp in links::ts_find_imports(source.as_bytes()) {
            if let Some(abs) = resolve_import_to_abs(&config.root, &file, &imp.path, remappings) {
                if abs.exists() && !visited.contains(&abs) {
                    queue.push_back(abs);
                }
            }
        }
    }

    let mut result: Vec<PathBuf> = visited.into_iter().collect();
    result.sort();
    result
}

/// Discover source files including library directories.
///
/// When `fullProjectScan` is enabled, this includes files from the configured
/// `libs` directories (e.g. `dependencies/`, `node_modules/`).  Files with
/// incompatible pragma versions are handled by the error-driven retry loop
/// in [`solc_project_index_from_files`].
pub fn discover_source_files_with_libs(config: &FoundryConfig) -> Vec<PathBuf> {
    discover_source_files_inner(config, true)
}

fn discover_source_files_inner(config: &FoundryConfig, include_libs: bool) -> Vec<PathBuf> {
    let root = &config.root;
    if !root.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let no_skip: &[String] = &[];

    // Walk only the configured source directories (src, test, script).
    // This matches Forge's behaviour: only files under these three directories
    // are considered project sources.  Directories like `certora/`, `hardhat/`,
    // etc. are never seeded.
    for dir_name in [&config.sources_dir, &config.test_dir, &config.script_dir] {
        let dir = root.join(dir_name);
        if dir.is_dir() {
            discover_recursive(&dir, no_skip, &mut files);
        }
    }

    // When include_libs is requested, also walk lib directories.
    if include_libs {
        for lib_name in &config.libs {
            let lib_dir = root.join(lib_name);
            if lib_dir.is_dir() {
                discover_recursive(&lib_dir, no_skip, &mut files);
            }
        }
    }

    files.sort();
    files
}

/// Discover the true compilation closure by tracing imports from the
/// project's own source files (`src/`, `test/`, `script/`, and any other
/// non-lib top-level directories).
///
/// Starting from every `.sol` file returned by [`discover_source_files`]
/// (project files only, no lib dirs), this BFS-walks the import graph using
/// the provided remappings to resolve each `import` statement to an absolute
/// path.  It adds every reachable file — including lib files that are actually
/// imported — to the result set.
///
/// Files whose imports cannot be resolved (missing external deps that aren't
/// in this project) are silently skipped at that edge; the importer is still
/// included.
///
/// This produces a much smaller, self-consistent set than scanning all files
/// in lib directories, and avoids pulling in lib files that have broken
/// transitive deps (e.g. chainlink automation files that need `@eth-optimism`
/// which is not vendored here).
pub fn discover_compilation_closure(config: &FoundryConfig, remappings: &[String]) -> Vec<PathBuf> {
    // Seed: all project source files (no lib dirs).
    let seeds = discover_source_files(config);
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: std::collections::VecDeque<PathBuf> = seeds.into_iter().collect();

    while let Some(file) = queue.pop_front() {
        if !visited.insert(file.clone()) {
            continue;
        }
        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => continue,
        };
        for imp in links::ts_find_imports(source.as_bytes()) {
            if let Some(abs) = resolve_import_to_abs(&config.root, &file, &imp.path, remappings) {
                if abs.exists() && !visited.contains(&abs) {
                    queue.push_back(abs);
                }
            }
        }
    }

    let mut result: Vec<PathBuf> = visited.into_iter().collect();
    result.sort();
    result
}

/// Directories that are always skipped during source file discovery,
/// regardless of the `include_libs` setting.
const DISCOVER_SKIP_DIRS: &[&str] = &["out", "artifacts", "cache", "target", "broadcast"];

fn discover_recursive(dir: &Path, skip_libs: &[String], files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip hidden directories (e.g., .git, .github)
                if name.starts_with('.') {
                    continue;
                }
                // Skip build artifact directories
                if DISCOVER_SKIP_DIRS.contains(&name) {
                    continue;
                }
                // Skip user-configured library directories (unless include_libs)
                if skip_libs.iter().any(|lib| lib == name) {
                    continue;
                }
            }
            discover_recursive(&path, skip_libs, files);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.ends_with(".sol")
        {
            files.push(path);
        }
    }
}

/// Build a `--standard-json` input that compiles all given source files at once.
///
/// Each file is added as a source entry with a `urls` field (relative to project root).
/// This produces a single AST covering the entire project in one solc invocation.
///
/// See [`build_standard_json_input`] for rationale on excluded settings.
pub fn build_batch_standard_json_input(
    source_files: &[PathBuf],
    remappings: &[String],
    config: &FoundryConfig,
) -> Value {
    build_batch_standard_json_input_with_cache(source_files, remappings, config, None)
}

/// Build a batch standard-json input for solc.
///
/// When `content_cache` is provided, files whose URI string appears as a key
/// are included with `"content"` (in-memory source).  Files not in the cache
/// fall back to `"urls"` (solc reads from disk).
///
/// This allows the re-index after a rename to feed solc the updated import
/// paths from our text_cache without requiring the editor to have flushed
/// them to disk yet.
pub fn build_batch_standard_json_input_with_cache(
    source_files: &[PathBuf],
    remappings: &[String],
    config: &FoundryConfig,
    content_cache: Option<&HashMap<crate::types::DocumentUri, (i32, String)>>,
) -> Value {
    let contract_outputs = vec!["devdoc", "userdoc", "evm.methodIdentifiers"];

    let mut settings = json!({
        "remappings": remappings,
        "outputSelection": {
            "*": {
                "*": contract_outputs,
                "": ["ast"]
            }
        }
    });

    if config.via_ir {
        settings["viaIR"] = json!(true);
    }
    if let Some(ref evm_version) = config.evm_version {
        settings["evmVersion"] = json!(evm_version);
    }

    let mut sources = serde_json::Map::new();
    for file in source_files {
        let rel_path = file
            .strip_prefix(&config.root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| file.to_string_lossy().into_owned());

        // Try to use cached content so solc doesn't need to read from disk.
        let cached_content = content_cache.and_then(|cache| {
            let uri = Url::from_file_path(file).ok()?;
            cache.get(uri.as_str()).map(|(_, c)| c.as_str())
        });

        if let Some(content) = cached_content {
            sources.insert(rel_path, json!({ "content": content }));
        } else {
            sources.insert(rel_path.clone(), json!({ "urls": [rel_path] }));
        }
    }

    json!({
        "language": "Solidity",
        "sources": sources,
        "settings": settings
    })
}

/// Build an AST-only batch standard-json input for sub-cache builds.
///
/// Unlike the full batch input, this omits all codegen-affecting settings
/// (`viaIR`, `evmVersion`, optimizer) and only requests the AST — no
/// `devdoc`, `userdoc`, or `evm.methodIdentifiers`.  This is significantly
/// faster because solc skips type-checking contract outputs and codegen.
///
/// Sub-caches only need the AST for cross-file reference lookup (node IDs,
/// `referencedDeclaration`, source locations).
pub fn build_batch_standard_json_input_ast_only(
    source_files: &[PathBuf],
    remappings: &[String],
    root: &Path,
) -> Value {
    let settings = json!({
        "remappings": remappings,
        "outputSelection": {
            "*": {
                "": ["ast"]
            }
        }
    });

    let mut sources = serde_json::Map::new();
    for file in source_files {
        let rel_path = file
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| file.to_string_lossy().into_owned());
        sources.insert(rel_path.clone(), json!({ "urls": [rel_path] }));
    }

    json!({
        "language": "Solidity",
        "sources": sources,
        "settings": settings
    })
}

/// Build a parse-only standard-json input (``stopAfter: "parsing"``).
///
/// Unlike the full batch input this mode stops before import resolution and
/// type-checking.  That means:
///
/// * No version 5333 errors cascade from imported incompatible files — the
///   compatible files are NOT fetched from disk as imports.
/// * The resulting ASTs contain all declaration nodes and local
///   ``referencedDeclaration`` IDs but **not** cross-file resolved IDs.
/// * Only ``ast`` output is requested; contract outputs (abi, gas …) are
///   omitted because they require type-checking.
///
/// This is used for the compatible-file batch in the mixed-version project
/// index so we can get parse-time ASTs for all project/lib files that satisfy
/// the project pragma, without being blocked by imports into incompatible lib
/// files.
pub fn build_parse_only_json_input(
    source_files: &[PathBuf],
    remappings: &[String],
    config: &FoundryConfig,
) -> Value {
    let settings = json!({
        "stopAfter": "parsing",
        "remappings": remappings,
        "outputSelection": {
            "*": {
                "": ["ast"]
            }
        }
    });

    let mut sources = serde_json::Map::new();
    for file in source_files {
        let rel_path = file
            .strip_prefix(&config.root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| file.to_string_lossy().into_owned());
        sources.insert(rel_path.clone(), json!({ "urls": [rel_path] }));
    }

    json!({
        "language": "Solidity",
        "sources": sources,
        "settings": settings
    })
}

/// Run a project-wide solc compilation and return normalized output.
///
/// Discovers all source files, compiles them in a single `solc --standard-json`
/// invocation, and returns the normalized AST data.
///
/// When `text_cache` is provided, files whose URI string appears as a key
/// are fed to solc via `"content"` (in-memory) rather than `"urls"` (disk).
/// This ensures the re-index after a rename uses the updated import paths
/// from our cache, even if the editor hasn't flushed them to disk yet.
pub async fn solc_project_index(
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
    text_cache: Option<&HashMap<crate::types::DocumentUri, (i32, String)>>,
) -> Result<Value, RunnerError> {
    // Resolve remappings first — needed for import tracing.
    let remappings = resolve_remappings(config).await;

    // Trace imports from project source files to find the true compilation
    // closure.  This avoids pulling in lib files that are never imported by
    // the project (e.g. chainlink automation files that need @eth-optimism,
    // which isn't vendored here).
    let source_files = discover_compilation_closure(config, &remappings);
    if source_files.is_empty() {
        return Err(RunnerError::CommandError(std::io::Error::other(
            "no source files found for project index",
        )));
    }

    solc_project_index_from_files(config, client, text_cache, &source_files).await
}

/// AST-only project index for sub-cache builds.
///
/// Identical to [`solc_project_index`] but requests only AST output —
/// no `devdoc`, `userdoc`, or `evm.methodIdentifiers`.  Also omits
/// `viaIR`, `evmVersion`, and optimizer settings since they only affect
/// codegen (which is skipped when no contract outputs are requested).
///
/// This is significantly faster because solc skips all codegen work.
/// "Stack too deep" errors cannot occur in AST-only mode.
pub async fn solc_project_index_ast_only(
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
) -> Result<Value, RunnerError> {
    let remappings = resolve_remappings(config).await;
    let source_files = discover_compilation_closure(config, &remappings);
    if source_files.is_empty() {
        return Err(RunnerError::CommandError(std::io::Error::other(
            "no source files found for project index",
        )));
    }
    solc_project_index_from_files_ast_only(config, client, &source_files).await
}

/// AST-only compile over a list of source files.
///
/// Like [`solc_project_index_from_files`] but uses
/// [`build_batch_standard_json_input_ast_only`] — no codegen settings,
/// no contract outputs.
async fn solc_project_index_from_files_ast_only(
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
    source_files: &[PathBuf],
) -> Result<Value, RunnerError> {
    if source_files.is_empty() {
        return Err(RunnerError::CommandError(std::io::Error::other(
            "no source files found for AST-only project index",
        )));
    }

    let remappings = resolve_remappings(config).await;

    let project_version: Option<SemVer> =
        config.solc_version.as_ref().and_then(|v| SemVer::parse(v));
    let constraint: Option<PragmaConstraint> = if let Some(ref v) = project_version {
        Some(PragmaConstraint::Exact(v.clone()))
    } else {
        source_files.iter().find_map(|f| {
            std::fs::read_to_string(f)
                .ok()
                .and_then(|src| parse_pragma(&src))
        })
    };
    let solc_binary = resolve_solc_binary(config, constraint.as_ref(), client).await;

    // Pre-scan pragmas to separate compatible vs incompatible files.
    let (compatible_files, incompatible_files) = if let Some(ref ver) = project_version {
        let mut compat = Vec::with_capacity(source_files.len());
        let mut incompat = Vec::new();
        for file in source_files {
            let is_compatible = std::fs::read_to_string(file)
                .ok()
                .and_then(|src| parse_pragma(&src))
                .map(|pragma| version_satisfies(ver, &pragma))
                .unwrap_or(true);
            if is_compatible {
                compat.push(file.clone());
            } else {
                incompat.push(file.clone());
            }
        }
        (compat, incompat)
    } else {
        (source_files.to_vec(), Vec::new())
    };

    if !incompatible_files.is_empty() {
        if let Some(c) = client {
            c.log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!(
                    "project index: {} compatible, {} incompatible with solc {}",
                    compatible_files.len(),
                    incompatible_files.len(),
                    project_version
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
            )
            .await;
        }
    }

    let mut result = if compatible_files.is_empty() {
        json!({"sources": {}, "contracts": {}, "errors": [], "source_id_to_path": {}})
    } else {
        let input =
            build_batch_standard_json_input_ast_only(&compatible_files, &remappings, &config.root);
        let raw = run_solc(&solc_binary, &input, &config.root).await?;
        normalize_solc_output(raw, Some(&config.root))
    };

    if incompatible_files.is_empty() {
        return Ok(result);
    }

    // Compile incompatible files individually with their own solc versions.
    for file in &incompatible_files {
        let pragma = std::fs::read_to_string(file)
            .ok()
            .and_then(|src| parse_pragma(&src));
        let file_binary = resolve_solc_binary(config, pragma.as_ref(), client).await;
        let input =
            build_batch_standard_json_input_ast_only(&[file.clone()], &remappings, &config.root);
        if let Ok(raw) = run_solc(&file_binary, &input, &config.root).await {
            let normalized = normalize_solc_output(raw, Some(&config.root));
            merge_normalized_outputs(&mut result, normalized);
        }
    }

    if let Some(c) = client {
        let total = result
            .get("sources")
            .and_then(|s| s.as_object())
            .map_or(0, |obj| obj.len());
        c.log_message(
            tower_lsp::lsp_types::MessageType::INFO,
            format!(
                "project index: compiled {} files ({} needed different solc version)",
                total,
                incompatible_files.len(),
            ),
        )
        .await;
    }

    Ok(result)
}

/// Run a scoped project-index compile over a selected file list.
///
/// This is intended for aggressive incremental reindex strategies where only
/// a dependency-closure subset should be recompiled.
pub async fn solc_project_index_scoped(
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
    text_cache: Option<&HashMap<crate::types::DocumentUri, (i32, String)>>,
    source_files: &[PathBuf],
) -> Result<Value, RunnerError> {
    if source_files.is_empty() {
        return Err(RunnerError::CommandError(std::io::Error::other(
            "no source files provided for scoped project index",
        )));
    }

    solc_project_index_from_files(config, client, text_cache, source_files).await
}

/// Extract source file paths from solc error code 5333 ("Source file requires
/// different compiler version") errors.  Returns the relative paths exactly
/// as they appear in `sourceLocation.file`.
#[cfg(test)]
fn extract_version_error_files(solc_output: &Value) -> HashSet<String> {
    let mut files = HashSet::new();
    if let Some(errors) = solc_output.get("errors").and_then(|e| e.as_array()) {
        for err in errors {
            let is_5333 = err.get("errorCode").and_then(|c| c.as_str()) == Some("5333");
            if is_5333
                && let Some(file) = err
                    .get("sourceLocation")
                    .and_then(|sl| sl.get("file"))
                    .and_then(|f| f.as_str())
            {
                files.insert(file.to_string());
            }
        }
    }
    files
}

/// Extract source file paths from solc error code 6275 ("Source not found")
/// errors.  Returns the relative paths of source files whose imports failed.
#[cfg(test)]
#[allow(dead_code)]
fn extract_import_error_files(solc_output: &Value) -> HashSet<String> {
    let mut files = HashSet::new();
    if let Some(errors) = solc_output.get("errors").and_then(|e| e.as_array()) {
        for err in errors {
            let is_6275 = err.get("errorCode").and_then(|c| c.as_str()) == Some("6275");
            if is_6275
                && let Some(file) = err
                    .get("sourceLocation")
                    .and_then(|sl| sl.get("file"))
                    .and_then(|f| f.as_str())
            {
                files.insert(file.to_string());
            }
        }
    }
    files
}

/// Build a reverse-import closure: given a set of files to exclude, find all
/// files that transitively import any of them.  Those files must also be
/// excluded because solc will still resolve their imports from disk and fail.
///
/// Returns the full exclusion set (seed files + their transitive importers).
#[cfg(test)]
fn reverse_import_closure(
    source_files: &[PathBuf],
    exclude_abs: &HashSet<PathBuf>,
    project_root: &Path,
    remappings: &[String],
) -> HashSet<PathBuf> {
    // Build forward import graph: file -> set of files it imports.
    // Then invert to get reverse edges: imported_file -> set of importers.
    let mut reverse_edges: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();

    for file in source_files {
        let Ok(bytes) = std::fs::read(file) else {
            continue;
        };
        for imp in links::ts_find_imports(&bytes) {
            if let Some(imported_abs) =
                resolve_import_to_abs(project_root, file, &imp.path, remappings)
            {
                reverse_edges
                    .entry(imported_abs)
                    .or_default()
                    .insert(file.clone());
            }
        }
    }

    // BFS from excluded files through reverse edges.
    let mut closure: HashSet<PathBuf> = exclude_abs.clone();
    let mut queue: std::collections::VecDeque<PathBuf> = exclude_abs.iter().cloned().collect();

    while let Some(current) = queue.pop_front() {
        if let Some(importers) = reverse_edges.get(&current) {
            for importer in importers {
                if closure.insert(importer.clone()) {
                    queue.push_back(importer.clone());
                }
            }
        }
    }

    closure
}

/// Merge two normalized solc outputs at the `Value` level.
///
/// Combines `sources`, `contracts`, `source_id_to_path`, and `errors` from
/// `other` into `base`.  Source IDs in `other` are remapped to avoid
/// collisions with `base`.
fn merge_normalized_outputs(base: &mut Value, other: Value) {
    // Merge sources (keyed by absolute path — no collisions across partitions).
    if let (Some(base_sources), Some(other_sources)) = (
        base.get_mut("sources").and_then(|s| s.as_object_mut()),
        other.get("sources").and_then(|s| s.as_object()),
    ) {
        // Find the max source ID in base so we can remap other's IDs.
        let max_base_id = base_sources
            .values()
            .filter_map(|v| v.get("id").and_then(|id| id.as_u64()))
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);

        // Collect other's id -> path mappings for source_id_to_path.
        let mut remapped_id_to_path: Vec<(String, String)> = Vec::new();

        for (path, mut source_data) in other_sources.clone() {
            // Remap the source ID to avoid collisions.
            if let Some(id) = source_data.get("id").and_then(|id| id.as_u64()) {
                let new_id = id + max_base_id;
                source_data
                    .as_object_mut()
                    .unwrap()
                    .insert("id".to_string(), json!(new_id));
                remapped_id_to_path.push((new_id.to_string(), path.clone()));
            }
            base_sources.insert(path, source_data);
        }

        // Merge source_id_to_path.
        if let Some(base_id_map) = base
            .get_mut("source_id_to_path")
            .and_then(|m| m.as_object_mut())
        {
            for (id, path) in remapped_id_to_path {
                base_id_map.insert(id, json!(path));
            }
        }
    }

    // Merge contracts.
    if let (Some(base_contracts), Some(other_contracts)) = (
        base.get_mut("contracts").and_then(|c| c.as_object_mut()),
        other.get("contracts").and_then(|c| c.as_object()),
    ) {
        for (path, contract_data) in other_contracts {
            base_contracts.insert(path.clone(), contract_data.clone());
        }
    }

    // Don't merge errors — the retry errors from incompatible files are noise.
    // The base already has the clean errors from the successful compilation.
}

async fn solc_project_index_from_files(
    config: &FoundryConfig,
    client: Option<&tower_lsp::Client>,
    text_cache: Option<&HashMap<crate::types::DocumentUri, (i32, String)>>,
    source_files: &[PathBuf],
) -> Result<Value, RunnerError> {
    if source_files.is_empty() {
        return Err(RunnerError::CommandError(std::io::Error::other(
            "no source files found for project index",
        )));
    }

    let remappings = resolve_remappings(config).await;

    // Resolve the project's solc version from foundry.toml.
    let project_version: Option<SemVer> =
        config.solc_version.as_ref().and_then(|v| SemVer::parse(v));

    // When no version is pinned in foundry.toml, derive a constraint from the
    // source files' pragmas so that svm can auto-install a matching binary.
    let constraint: Option<PragmaConstraint> = if let Some(ref v) = project_version {
        Some(PragmaConstraint::Exact(v.clone()))
    } else {
        source_files.iter().find_map(|f| {
            std::fs::read_to_string(f)
                .ok()
                .and_then(|src| parse_pragma(&src))
        })
    };
    let solc_binary = resolve_solc_binary(config, constraint.as_ref(), client).await;

    // -- Pre-scan pragmas to separate compatible vs incompatible files. --
    //
    // Solc emits ZERO ASTs when any file in the batch has a version error
    // (5333).  We must exclude incompatible files before compiling so the
    // batch produces full AST output for all compatible files.
    let (compatible_files, incompatible_files) = if let Some(ref ver) = project_version {
        let mut compat = Vec::with_capacity(source_files.len());
        let mut incompat = Vec::new();
        for file in source_files {
            let is_compatible = std::fs::read_to_string(file)
                .ok()
                .and_then(|src| parse_pragma(&src))
                .map(|pragma| version_satisfies(ver, &pragma))
                // Files without a pragma are assumed compatible.
                .unwrap_or(true);
            if is_compatible {
                compat.push(file.clone());
            } else {
                incompat.push(file.clone());
            }
        }
        (compat, incompat)
    } else {
        // No project version configured — compile everything in one batch.
        (source_files.to_vec(), Vec::new())
    };

    if !incompatible_files.is_empty() {
        if let Some(c) = client {
            c.log_message(
                tower_lsp::lsp_types::MessageType::INFO,
                format!(
                    "project index: {} compatible, {} incompatible with solc {}",
                    compatible_files.len(),
                    incompatible_files.len(),
                    project_version
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
            )
            .await;
        }
    }

    // -- Full batch compile of compatible files. --
    //
    // The source file list comes from discover_compilation_closure which only
    // includes files reachable via imports from src/test/script — so all files
    // in the batch are version-compatible and their transitive imports resolve.
    // A full (non-parse-only) compile is required so that cross-file
    // referencedDeclaration IDs are populated for goto-references to work.
    let mut result = if compatible_files.is_empty() {
        json!({"sources": {}, "contracts": {}, "errors": [], "source_id_to_path": {}})
    } else {
        let input = build_batch_standard_json_input_with_cache(
            &compatible_files,
            &remappings,
            config,
            text_cache,
        );
        let raw = run_solc(&solc_binary, &input, &config.root).await?;
        normalize_solc_output(raw, Some(&config.root))
    };

    let batch_source_count = result
        .get("sources")
        .and_then(|s| s.as_object())
        .map_or(0, |obj| obj.len());

    if incompatible_files.is_empty() {
        return Ok(result);
    }

    if let Some(c) = client {
        // Log first few errors from the batch to understand why sources=0.
        let batch_errors: Vec<String> = result
            .get("errors")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|e| e.get("severity").and_then(|s| s.as_str()) == Some("error"))
                    .take(3)
                    .filter_map(|e| {
                        let msg = e.get("message").and_then(|m| m.as_str()).unwrap_or("?");
                        let file = e
                            .get("sourceLocation")
                            .and_then(|sl| sl.get("file"))
                            .and_then(|f| f.as_str())
                            .unwrap_or("?");
                        Some(format!("{file}: {msg}"))
                    })
                    .collect()
            })
            .unwrap_or_default();

        c.log_message(
            tower_lsp::lsp_types::MessageType::INFO,
            format!(
                "project index: batch produced {} sources, now compiling {} incompatible files individually{}",
                batch_source_count,
                incompatible_files.len(),
                if batch_errors.is_empty() {
                    String::new()
                } else {
                    format!(" [first errors: {}]", batch_errors.join("; "))
                },
            ),
        )
        .await;
    }

    // -- Individually compile incompatible files with their matching solc. --
    let mut compiled = 0usize;
    let mut skipped = 0usize;
    for file in &incompatible_files {
        let pragma = std::fs::read_to_string(file)
            .ok()
            .and_then(|src| parse_pragma(&src));

        let Some(file_constraint) = pragma else {
            skipped += 1;
            continue;
        };

        let file_binary = resolve_solc_binary(config, Some(&file_constraint), client).await;
        let input = build_batch_standard_json_input_with_cache(
            &[file.clone()],
            &remappings,
            config,
            text_cache,
        );
        match run_solc(&file_binary, &input, &config.root).await {
            Ok(raw) => {
                let normalized = normalize_solc_output(raw, Some(&config.root));
                merge_normalized_outputs(&mut result, normalized);
                compiled += 1;
            }
            Err(e) => {
                if let Some(c) = client {
                    c.log_message(
                        tower_lsp::lsp_types::MessageType::WARNING,
                        format!(
                            "project index: incompatible file {} failed: {e}",
                            file.display(),
                        ),
                    )
                    .await;
                }
                skipped += 1;
            }
        }
    }

    if let Some(c) = client {
        c.log_message(
            tower_lsp::lsp_types::MessageType::INFO,
            format!(
                "project index: incompatible files done — {compiled} compiled, {skipped} skipped",
            ),
        )
        .await;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_solc_sources() {
        let solc_output = json!({
            "sources": {
                "src/Foo.sol": {
                    "id": 0,
                    "ast": {
                        "nodeType": "SourceUnit",
                        "absolutePath": "src/Foo.sol",
                        "id": 100
                    }
                },
                "src/Bar.sol": {
                    "id": 1,
                    "ast": {
                        "nodeType": "SourceUnit",
                        "absolutePath": "src/Bar.sol",
                        "id": 200
                    }
                }
            },
            "contracts": {},
            "errors": []
        });

        let normalized = normalize_solc_output(solc_output, None);

        // Sources kept in solc-native shape: path -> { id, ast }
        let sources = normalized.get("sources").unwrap().as_object().unwrap();
        assert_eq!(sources.len(), 2);

        let foo = sources.get("src/Foo.sol").unwrap();
        assert_eq!(foo.get("id").unwrap(), 0);
        assert_eq!(
            foo.get("ast")
                .unwrap()
                .get("nodeType")
                .unwrap()
                .as_str()
                .unwrap(),
            "SourceUnit"
        );

        // Check source_id_to_path constructed
        let id_to_path = normalized
            .get("source_id_to_path")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(id_to_path.len(), 2);
    }

    #[test]
    fn test_normalize_solc_contracts() {
        let solc_output = json!({
            "sources": {},
            "contracts": {
                "src/Foo.sol": {
                    "Foo": {
                        "abi": [{"type": "function", "name": "bar"}],
                        "evm": {
                            "methodIdentifiers": {
                                "bar(uint256)": "abcd1234"
                            }
                        }
                    }
                }
            },
            "errors": []
        });

        let normalized = normalize_solc_output(solc_output, None);

        // Contracts kept in solc-native shape: path -> name -> { abi, evm, ... }
        let contracts = normalized.get("contracts").unwrap().as_object().unwrap();
        let foo_contracts = contracts.get("src/Foo.sol").unwrap().as_object().unwrap();
        let foo = foo_contracts.get("Foo").unwrap();

        let method_ids = foo
            .get("evm")
            .unwrap()
            .get("methodIdentifiers")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(
            method_ids.get("bar(uint256)").unwrap().as_str().unwrap(),
            "abcd1234"
        );
    }

    #[test]
    fn test_normalize_solc_errors_passthrough() {
        let solc_output = json!({
            "sources": {},
            "contracts": {},
            "errors": [{
                "sourceLocation": {"file": "src/Foo.sol", "start": 0, "end": 10},
                "type": "Warning",
                "component": "general",
                "severity": "warning",
                "errorCode": "2394",
                "message": "test warning",
                "formattedMessage": "Warning: test warning"
            }]
        });

        let normalized = normalize_solc_output(solc_output, None);

        let errors = normalized.get("errors").unwrap().as_array().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].get("errorCode").unwrap().as_str().unwrap(),
            "2394"
        );
    }

    #[test]
    fn test_normalize_empty_solc_output() {
        let solc_output = json!({
            "sources": {},
            "contracts": {}
        });

        let normalized = normalize_solc_output(solc_output, None);

        assert!(
            normalized
                .get("sources")
                .unwrap()
                .as_object()
                .unwrap()
                .is_empty()
        );
        assert!(
            normalized
                .get("contracts")
                .unwrap()
                .as_object()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            normalized.get("errors").unwrap().as_array().unwrap().len(),
            0
        );
        assert!(
            normalized
                .get("source_id_to_path")
                .unwrap()
                .as_object()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_build_standard_json_input() {
        let config = FoundryConfig::default();
        let input = build_standard_json_input(
            "/path/to/Foo.sol",
            &[
                "ds-test/=lib/forge-std/lib/ds-test/src/".to_string(),
                "forge-std/=lib/forge-std/src/".to_string(),
            ],
            &config,
        );

        let sources = input.get("sources").unwrap().as_object().unwrap();
        assert!(sources.contains_key("/path/to/Foo.sol"));

        let settings = input.get("settings").unwrap();
        let remappings = settings.get("remappings").unwrap().as_array().unwrap();
        assert_eq!(remappings.len(), 2);

        let output_sel = settings.get("outputSelection").unwrap();
        assert!(output_sel.get("*").is_some());

        // Default config: no optimizer, no viaIR, no evmVersion
        assert!(settings.get("optimizer").is_none());
        assert!(settings.get("viaIR").is_none());
        assert!(settings.get("evmVersion").is_none());

        // gasEstimates is never requested — forces full EVM codegen (88% of compile time)
        let outputs = settings["outputSelection"]["*"]["*"].as_array().unwrap();
        let output_names: Vec<&str> = outputs.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(!output_names.contains(&"evm.gasEstimates"));
        assert!(!output_names.contains(&"abi")); // ABI is intentionally omitted — no consumer
        assert!(output_names.contains(&"devdoc"));
        assert!(output_names.contains(&"userdoc"));
        assert!(output_names.contains(&"evm.methodIdentifiers"));
    }

    #[test]
    fn test_build_standard_json_input_with_config() {
        let config = FoundryConfig {
            optimizer: true,
            optimizer_runs: 9999999,
            via_ir: true,
            evm_version: Some("osaka".to_string()),
            ..Default::default()
        };
        let input = build_standard_json_input("/path/to/Foo.sol", &[], &config);

        let settings = input.get("settings").unwrap();

        // Optimizer is never passed — adds ~3s and doesn't affect AST/ABI/docs
        assert!(settings.get("optimizer").is_none());

        // viaIR IS passed when config has it (some contracts require it to compile)
        assert!(settings.get("viaIR").unwrap().as_bool().unwrap());

        // gasEstimates is never requested regardless of viaIR
        let outputs = settings["outputSelection"]["*"]["*"].as_array().unwrap();
        let output_names: Vec<&str> = outputs.iter().map(|v| v.as_str().unwrap()).collect();
        assert!(!output_names.contains(&"evm.gasEstimates"));

        // EVM version
        assert_eq!(
            settings.get("evmVersion").unwrap().as_str().unwrap(),
            "osaka"
        );
    }

    #[tokio::test]
    async fn test_resolve_solc_binary_default() {
        let config = FoundryConfig::default();
        let binary = resolve_solc_binary(&config, None, None).await;
        assert_eq!(binary, PathBuf::from("solc"));
    }

    #[test]
    fn test_parse_pragma_exact() {
        let source = "// SPDX\npragma solidity 0.8.26;\n";
        assert_eq!(
            parse_pragma(source),
            Some(PragmaConstraint::Exact(SemVer {
                major: 0,
                minor: 8,
                patch: 26
            }))
        );
    }

    #[test]
    fn test_parse_pragma_caret() {
        let source = "pragma solidity ^0.8.0;\n";
        assert_eq!(
            parse_pragma(source),
            Some(PragmaConstraint::Caret(SemVer {
                major: 0,
                minor: 8,
                patch: 0
            }))
        );
    }

    #[test]
    fn test_parse_pragma_gte() {
        let source = "pragma solidity >=0.8.0;\n";
        assert_eq!(
            parse_pragma(source),
            Some(PragmaConstraint::Gte(SemVer {
                major: 0,
                minor: 8,
                patch: 0
            }))
        );
    }

    #[test]
    fn test_parse_pragma_range() {
        let source = "pragma solidity >=0.6.2 <0.9.0;\n";
        assert_eq!(
            parse_pragma(source),
            Some(PragmaConstraint::Range(
                SemVer {
                    major: 0,
                    minor: 6,
                    patch: 2
                },
                SemVer {
                    major: 0,
                    minor: 9,
                    patch: 0
                },
            ))
        );
    }

    #[test]
    fn test_parse_pragma_none() {
        let source = "contract Foo {}\n";
        assert_eq!(parse_pragma(source), None);
    }

    #[test]
    fn test_version_satisfies_exact() {
        let v = SemVer {
            major: 0,
            minor: 8,
            patch: 26,
        };
        assert!(version_satisfies(&v, &PragmaConstraint::Exact(v.clone())));
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 8,
                patch: 25
            },
            &PragmaConstraint::Exact(v)
        ));
    }

    #[test]
    fn test_version_satisfies_caret() {
        let constraint = PragmaConstraint::Caret(SemVer {
            major: 0,
            minor: 8,
            patch: 0,
        });
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 8,
                patch: 0
            },
            &constraint
        ));
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 8,
                patch: 26
            },
            &constraint
        ));
        // 0.9.0 is outside ^0.8.0
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 9,
                patch: 0
            },
            &constraint
        ));
        // 0.7.0 is below
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 7,
                patch: 0
            },
            &constraint
        ));
    }

    #[test]
    fn test_version_satisfies_gte() {
        let constraint = PragmaConstraint::Gte(SemVer {
            major: 0,
            minor: 8,
            patch: 0,
        });
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 8,
                patch: 0
            },
            &constraint
        ));
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 9,
                patch: 0
            },
            &constraint
        ));
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 7,
                patch: 0
            },
            &constraint
        ));
    }

    #[test]
    fn test_version_satisfies_range() {
        let constraint = PragmaConstraint::Range(
            SemVer {
                major: 0,
                minor: 6,
                patch: 2,
            },
            SemVer {
                major: 0,
                minor: 9,
                patch: 0,
            },
        );
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 6,
                patch: 2
            },
            &constraint
        ));
        assert!(version_satisfies(
            &SemVer {
                major: 0,
                minor: 8,
                patch: 26
            },
            &constraint
        ));
        // 0.9.0 is the upper bound (exclusive)
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 9,
                patch: 0
            },
            &constraint
        ));
        assert!(!version_satisfies(
            &SemVer {
                major: 0,
                minor: 6,
                patch: 1
            },
            &constraint
        ));
    }

    #[test]
    fn test_find_matching_version() {
        let installed = vec![
            SemVer {
                major: 0,
                minor: 8,
                patch: 0,
            },
            SemVer {
                major: 0,
                minor: 8,
                patch: 20,
            },
            SemVer {
                major: 0,
                minor: 8,
                patch: 26,
            },
            SemVer {
                major: 0,
                minor: 8,
                patch: 33,
            },
        ];
        // ^0.8.20 should pick latest: 0.8.33
        let constraint = PragmaConstraint::Caret(SemVer {
            major: 0,
            minor: 8,
            patch: 20,
        });
        let matched = find_matching_version(&constraint, &installed);
        assert_eq!(
            matched,
            Some(SemVer {
                major: 0,
                minor: 8,
                patch: 33
            })
        );

        // exact 0.8.20
        let constraint = PragmaConstraint::Exact(SemVer {
            major: 0,
            minor: 8,
            patch: 20,
        });
        let matched = find_matching_version(&constraint, &installed);
        assert_eq!(
            matched,
            Some(SemVer {
                major: 0,
                minor: 8,
                patch: 20
            })
        );

        // exact 0.8.15 — not installed
        let constraint = PragmaConstraint::Exact(SemVer {
            major: 0,
            minor: 8,
            patch: 15,
        });
        let matched = find_matching_version(&constraint, &installed);
        assert_eq!(matched, None);
    }

    #[test]
    fn test_list_installed_versions() {
        // Just verify it doesn't panic — actual versions depend on system
        let versions = list_installed_versions();
        // Versions should be sorted
        for w in versions.windows(2) {
            assert!(w[0] <= w[1]);
        }
    }

    // -------------------------------------------------------------------
    // Tests for mixed-version retry helpers
    // -------------------------------------------------------------------

    #[test]
    fn test_extract_version_error_files_basic() {
        let output = json!({
            "errors": [
                {
                    "errorCode": "5333",
                    "severity": "error",
                    "message": "Source file requires different compiler version",
                    "sourceLocation": {
                        "file": "lib/openzeppelin/contracts/token/ERC20/ERC20.sol",
                        "start": 32,
                        "end": 58
                    }
                },
                {
                    "errorCode": "5333",
                    "severity": "error",
                    "message": "Source file requires different compiler version",
                    "sourceLocation": {
                        "file": "lib/old-lib/src/Legacy.sol",
                        "start": 32,
                        "end": 58
                    }
                },
                {
                    "errorCode": "9574",
                    "severity": "error",
                    "message": "Some other error",
                    "sourceLocation": {
                        "file": "src/Main.sol",
                        "start": 100,
                        "end": 200
                    }
                }
            ]
        });

        let files = extract_version_error_files(&output);
        assert_eq!(files.len(), 2);
        assert!(files.contains("lib/openzeppelin/contracts/token/ERC20/ERC20.sol"));
        assert!(files.contains("lib/old-lib/src/Legacy.sol"));
        // Non-5333 error files should NOT be included.
        assert!(!files.contains("src/Main.sol"));
    }

    #[test]
    fn test_extract_version_error_files_empty() {
        let output = json!({
            "errors": []
        });
        assert!(extract_version_error_files(&output).is_empty());

        // No errors key at all.
        let output = json!({});
        assert!(extract_version_error_files(&output).is_empty());
    }

    #[test]
    fn test_extract_version_error_files_no_source_location() {
        let output = json!({
            "errors": [
                {
                    "errorCode": "5333",
                    "severity": "error",
                    "message": "Source file requires different compiler version"
                    // No sourceLocation field.
                }
            ]
        });
        assert!(extract_version_error_files(&output).is_empty());
    }

    #[test]
    fn test_extract_version_error_files_dedup() {
        let output = json!({
            "errors": [
                {
                    "errorCode": "5333",
                    "severity": "error",
                    "sourceLocation": { "file": "lib/same.sol", "start": 0, "end": 10 }
                },
                {
                    "errorCode": "5333",
                    "severity": "error",
                    "sourceLocation": { "file": "lib/same.sol", "start": 50, "end": 70 }
                }
            ]
        });
        let files = extract_version_error_files(&output);
        assert_eq!(files.len(), 1);
        assert!(files.contains("lib/same.sol"));
    }

    #[test]
    fn test_reverse_import_closure_simple() {
        // Create a temp directory with three files:
        //   a.sol imports b.sol
        //   b.sol imports c.sol
        //   d.sol (standalone)
        //
        // If c.sol is excluded, the closure should include: c.sol, b.sol, a.sol
        // (b imports c, a imports b — both are transitive importers of c).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        std::fs::write(
            root.join("a.sol"),
            "// SPDX-License-Identifier: MIT\nimport \"./b.sol\";\ncontract A {}",
        )
        .unwrap();
        std::fs::write(
            root.join("b.sol"),
            "// SPDX-License-Identifier: MIT\nimport \"./c.sol\";\ncontract B {}",
        )
        .unwrap();
        std::fs::write(
            root.join("c.sol"),
            "// SPDX-License-Identifier: MIT\ncontract C {}",
        )
        .unwrap();
        std::fs::write(
            root.join("d.sol"),
            "// SPDX-License-Identifier: MIT\ncontract D {}",
        )
        .unwrap();

        let files: Vec<PathBuf> = vec![
            root.join("a.sol"),
            root.join("b.sol"),
            root.join("c.sol"),
            root.join("d.sol"),
        ];

        let exclude: HashSet<PathBuf> = [root.join("c.sol")].into_iter().collect();
        let closure = reverse_import_closure(&files, &exclude, root, &[]);

        assert!(
            closure.contains(&root.join("c.sol")),
            "seed file in closure"
        );
        assert!(closure.contains(&root.join("b.sol")), "direct importer");
        assert!(closure.contains(&root.join("a.sol")), "transitive importer");
        assert!(
            !closure.contains(&root.join("d.sol")),
            "unrelated file not in closure"
        );
        assert_eq!(closure.len(), 3);
    }

    #[test]
    fn test_reverse_import_closure_no_importers() {
        // Excluding a file that nothing imports — closure is just the seed.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        std::fs::write(root.join("a.sol"), "contract A {}").unwrap();
        std::fs::write(root.join("b.sol"), "contract B {}").unwrap();

        let files: Vec<PathBuf> = vec![root.join("a.sol"), root.join("b.sol")];
        let exclude: HashSet<PathBuf> = [root.join("a.sol")].into_iter().collect();

        let closure = reverse_import_closure(&files, &exclude, root, &[]);
        assert_eq!(closure.len(), 1);
        assert!(closure.contains(&root.join("a.sol")));
    }

    #[test]
    fn test_reverse_import_closure_diamond() {
        // Diamond pattern:
        //   a.sol imports b.sol and c.sol
        //   b.sol imports d.sol
        //   c.sol imports d.sol
        //
        // Excluding d.sol → closure = {d, b, c, a}
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        std::fs::write(
            root.join("a.sol"),
            "import \"./b.sol\";\nimport \"./c.sol\";\ncontract A {}",
        )
        .unwrap();
        std::fs::write(root.join("b.sol"), "import \"./d.sol\";\ncontract B {}").unwrap();
        std::fs::write(root.join("c.sol"), "import \"./d.sol\";\ncontract C {}").unwrap();
        std::fs::write(root.join("d.sol"), "contract D {}").unwrap();

        let files: Vec<PathBuf> = vec![
            root.join("a.sol"),
            root.join("b.sol"),
            root.join("c.sol"),
            root.join("d.sol"),
        ];
        let exclude: HashSet<PathBuf> = [root.join("d.sol")].into_iter().collect();

        let closure = reverse_import_closure(&files, &exclude, root, &[]);
        assert_eq!(closure.len(), 4);
    }

    #[test]
    fn test_merge_normalized_outputs_basic() {
        let mut base = json!({
            "sources": {
                "/abs/src/A.sol": { "id": 0, "ast": { "nodeType": "SourceUnit" } },
                "/abs/src/B.sol": { "id": 1, "ast": { "nodeType": "SourceUnit" } }
            },
            "contracts": {
                "/abs/src/A.sol": { "A": { "abi": [] } }
            },
            "errors": [],
            "source_id_to_path": {
                "0": "/abs/src/A.sol",
                "1": "/abs/src/B.sol"
            }
        });

        let other = json!({
            "sources": {
                "/abs/lib/C.sol": { "id": 0, "ast": { "nodeType": "SourceUnit" } }
            },
            "contracts": {
                "/abs/lib/C.sol": { "C": { "abi": [] } }
            },
            "errors": [],
            "source_id_to_path": {
                "0": "/abs/lib/C.sol"
            }
        });

        merge_normalized_outputs(&mut base, other);

        // Sources should now have 3 entries.
        let sources = base["sources"].as_object().unwrap();
        assert_eq!(sources.len(), 3);
        assert!(sources.contains_key("/abs/lib/C.sol"));

        // The merged source's ID should be remapped (0 + max_base_id=2 → 2).
        let c_id = sources["/abs/lib/C.sol"]["id"].as_u64().unwrap();
        assert_eq!(
            c_id, 2,
            "remapped id should be max_base_id (2) + original (0)"
        );

        // source_id_to_path should have 3 entries.
        let id_map = base["source_id_to_path"].as_object().unwrap();
        assert_eq!(id_map.len(), 3);
        assert_eq!(id_map["2"].as_str().unwrap(), "/abs/lib/C.sol");

        // Contracts should have 2 entries.
        let contracts = base["contracts"].as_object().unwrap();
        assert_eq!(contracts.len(), 2);
        assert!(contracts.contains_key("/abs/lib/C.sol"));
    }

    #[test]
    fn test_merge_normalized_outputs_empty_other() {
        let mut base = json!({
            "sources": {
                "/abs/src/A.sol": { "id": 0, "ast": {} }
            },
            "contracts": {},
            "errors": [],
            "source_id_to_path": { "0": "/abs/src/A.sol" }
        });

        let other = json!({
            "sources": {},
            "contracts": {},
            "errors": [],
            "source_id_to_path": {}
        });

        merge_normalized_outputs(&mut base, other);

        let sources = base["sources"].as_object().unwrap();
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_merge_normalized_outputs_empty_base() {
        let mut base = json!({
            "sources": {},
            "contracts": {},
            "errors": [],
            "source_id_to_path": {}
        });

        let other = json!({
            "sources": {
                "/abs/lib/X.sol": { "id": 0, "ast": {} }
            },
            "contracts": {
                "/abs/lib/X.sol": { "X": { "abi": [] } }
            },
            "errors": [],
            "source_id_to_path": { "0": "/abs/lib/X.sol" }
        });

        merge_normalized_outputs(&mut base, other);

        let sources = base["sources"].as_object().unwrap();
        assert_eq!(sources.len(), 1);
        // max_base_id is 0 (no entries), so remapped id = 0 + 0 = 0.
        let x_id = sources["/abs/lib/X.sol"]["id"].as_u64().unwrap();
        assert_eq!(x_id, 0);
    }
}
