use crate::links;
use crate::utils;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{Position, Range, TextEdit, Url};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single file rename: old absolute path → new absolute path.
#[derive(Debug, Clone)]
pub struct FileRename {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
}

/// Diagnostic counters returned alongside edits so the caller can log them.
#[derive(Debug, Default)]
pub struct RenameStats {
    /// Source files whose bytes could not be read.
    pub read_failures: usize,
    /// Source files with no parseable parent directory.
    pub no_parent: usize,
    /// Imports skipped because pathdiff returned None.
    pub pathdiff_failures: usize,
    /// Imports whose new path equals the old path (no-op, correctly skipped).
    pub no_op_skips: usize,
    /// Edits suppressed because Case 1 already covered the range.
    pub dedup_skips: usize,
    /// Duplicate old_path entries detected in the rename list (last wins).
    pub duplicate_renames: usize,
}

/// Result of a rename_imports call: edits + diagnostic stats.
pub struct RenameResult {
    pub edits: HashMap<Url, Vec<TextEdit>>,
    pub stats: RenameStats,
}

/// Diagnostic counters returned alongside delete edits.
#[derive(Debug, Default)]
pub struct DeleteStats {
    /// Source files whose bytes could not be read.
    pub read_failures: usize,
    /// Source files with no parseable parent directory.
    pub no_parent: usize,
    /// Imports where we could not determine a full import-statement span.
    pub statement_range_failures: usize,
    /// Duplicate delete targets detected in the delete list.
    pub duplicate_deletes: usize,
    /// Duplicate edits skipped for the same statement range.
    pub dedup_skips: usize,
}

/// Result of delete_imports call: edits + diagnostic stats.
pub struct DeleteResult {
    pub edits: HashMap<Url, Vec<TextEdit>>,
    pub stats: DeleteStats,
}

// ---------------------------------------------------------------------------
// Folder expansion
// ---------------------------------------------------------------------------

/// Expand rename entries that target folders into per-file renames.
///
/// For each entry, if `old_path` is a directory (or has no `.sol` extension),
/// every source file under it is expanded to a concrete file rename.
/// Uses `Path::strip_prefix` for component-aware matching (won't match
/// `/src2` given `/src`).
pub fn expand_folder_renames(
    params: &[(PathBuf, PathBuf)],
    source_files: &[String],
) -> Vec<FileRename> {
    // Deduplicate by old path; last entry wins.
    let mut dedup: HashMap<PathBuf, PathBuf> = HashMap::new();
    for (old_path, new_path) in params {
        if old_path.is_dir() || !old_path.extension().map_or(false, |e| e == "sol") {
            for sf in source_files {
                let sf_path = Path::new(sf);
                if let Ok(suffix) = sf_path.strip_prefix(old_path) {
                    dedup.insert(sf_path.to_path_buf(), new_path.join(suffix));
                }
            }
        } else {
            dedup.insert(old_path.clone(), new_path.clone());
        }
    }
    dedup
        .into_iter()
        .map(|(old_path, new_path)| FileRename { old_path, new_path })
        .collect()
}

/// Expand delete entries that target folders into per-file paths.
///
/// For each entry, if it is a directory (or has no `.sol` extension),
/// every source file under it is expanded to a concrete file path.
pub fn expand_folder_deletes(params: &[PathBuf], source_files: &[String]) -> Vec<PathBuf> {
    let mut dedup: HashMap<PathBuf, ()> = HashMap::new();
    for old_path in params {
        if old_path.is_dir() || !old_path.extension().map_or(false, |e| e == "sol") {
            for sf in source_files {
                let sf_path = Path::new(sf);
                if sf_path.strip_prefix(old_path).is_ok() {
                    dedup.insert(sf_path.to_path_buf(), ());
                }
            }
        } else {
            dedup.insert(old_path.clone(), ());
        }
    }
    dedup.into_keys().collect()
}

/// Expand folder renames using candidate filesystem paths.
///
/// `candidate_paths` should be the union of discovered project files and files
/// currently present in `text_cache` so folder renames don't miss entries.
pub fn expand_folder_renames_from_paths(
    params: &[(Url, Url)],
    candidate_paths: &[PathBuf],
) -> Vec<(String, String)> {
    // Deduplicate by old URI; last entry wins.
    let mut dedup: HashMap<String, String> = HashMap::new();
    for (old_uri, new_uri) in params {
        let old_path = match old_uri.to_file_path() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let new_path = match new_uri.to_file_path() {
            Ok(p) => p,
            Err(_) => continue,
        };

        if old_path.extension().map_or(false, |e| e == "sol") && !old_path.is_dir() {
            dedup.insert(old_uri.to_string(), new_uri.to_string());
        } else {
            for existing_path in candidate_paths {
                if let Ok(suffix) = existing_path.strip_prefix(&old_path) {
                    let new_file_path = new_path.join(suffix);
                    let Ok(existing_uri) = Url::from_file_path(existing_path) else {
                        continue;
                    };
                    let Ok(new_file_uri) = Url::from_file_path(&new_file_path) else {
                        continue;
                    };
                    dedup.insert(existing_uri.to_string(), new_file_uri.to_string());
                }
            }
        }
    }
    dedup.into_iter().collect()
}

/// Expand delete entries using candidate filesystem paths.
///
/// `candidate_paths` should be the union of discovered project files and files
/// currently present in `text_cache` so folder deletes don't miss entries.
pub fn expand_folder_deletes_from_paths(
    params: &[Url],
    candidate_paths: &[PathBuf],
) -> Vec<PathBuf> {
    let mut dedup: HashMap<PathBuf, ()> = HashMap::new();
    for uri in params {
        let old_path = match uri.to_file_path() {
            Ok(p) => p,
            Err(_) => continue,
        };

        if old_path.extension().map_or(false, |e| e == "sol") && !old_path.is_dir() {
            dedup.insert(old_path, ());
        } else {
            for existing_path in candidate_paths {
                if existing_path.strip_prefix(&old_path).is_ok() {
                    dedup.insert(existing_path.clone(), ());
                }
            }
        }
    }
    dedup.into_keys().collect()
}

// ---------------------------------------------------------------------------
// Core rename logic
// ---------------------------------------------------------------------------

/// Compute import-path edits needed when one or more files are renamed/moved.
///
/// Handles batch renames correctly: when both an importer and its import target
/// are being moved in the same request (e.g. folder rename), the relative path
/// between them is computed using their **new** locations, so imports that are
/// still valid after the move are left unchanged.
///
/// Uses tree-sitter to find import strings in each source file, making this
/// robust against stale or unavailable solc AST data.
///
/// Handles two cases per renamed file:
/// 1. **Other files import the renamed file** — their import path string must
///    change to reflect the new location of the target.
/// 2. **The renamed file's own relative imports** — if the file moved to a
///    different directory, its relative imports to other files need updating.
pub fn rename_imports(
    source_files: &[String],
    renames: &[FileRename],
    project_root: &Path,
    get_source_bytes: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> RenameResult {
    let mut edits: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    let mut stats = RenameStats::default();

    if renames.is_empty() {
        return RenameResult { edits, stats };
    }

    // Build lookup: old_path → new_path.
    // Detect duplicates (same old_path appears more than once).
    let mut rename_map: HashMap<PathBuf, PathBuf> = HashMap::with_capacity(renames.len());
    for r in renames {
        if rename_map
            .insert(r.old_path.clone(), r.new_path.clone())
            .is_some()
        {
            stats.duplicate_renames += 1;
        }
    }

    // ── Case 1: files that import a renamed file ───────────────────────
    for source_fs_str in source_files {
        let source_path = resolve_source_path(source_fs_str, project_root);

        // If this source file is itself being renamed, use its NEW directory
        // for computing replacement import paths.
        let source_new_path = rename_map.get(&source_path);

        let effective_source_dir = match source_new_path {
            Some(new_p) => match new_p.parent() {
                Some(d) => d.to_path_buf(),
                None => {
                    stats.no_parent += 1;
                    continue;
                }
            },
            None => match source_path.parent() {
                Some(d) => d.to_path_buf(),
                None => {
                    stats.no_parent += 1;
                    continue;
                }
            },
        };

        let current_source_dir = match source_path.parent() {
            Some(d) => d,
            None => {
                stats.no_parent += 1;
                continue;
            }
        };

        let bytes = match source_path
            .to_str()
            .and_then(get_source_bytes)
            .or_else(|| get_source_bytes(source_fs_str))
        {
            Some(b) => b,
            None => {
                stats.read_failures += 1;
                continue;
            }
        };

        let imports = links::ts_find_imports(&bytes);

        for imp in &imports {
            let resolved = normalize_path(&current_source_dir.join(&imp.path));

            let resolved_target = if rename_map.contains_key(&resolved) {
                Some(resolved)
            } else if !imp.path.starts_with('.') {
                let via_root = normalize_path(&project_root.join(&imp.path));
                if rename_map.contains_key(&via_root) {
                    Some(via_root)
                } else {
                    None
                }
            } else {
                None
            };

            let old_target = match resolved_target {
                Some(t) => t,
                None => continue,
            };

            let new_target = &rename_map[&old_target];

            let new_import_path = if imp.path.starts_with('.') {
                match pathdiff::diff_paths(new_target, &effective_source_dir) {
                    Some(p) => ensure_dot_prefix(&p),
                    None => {
                        stats.pathdiff_failures += 1;
                        continue;
                    }
                }
            } else {
                match pathdiff::diff_paths(new_target, project_root) {
                    Some(p) => normalize_slashes(&p.to_string_lossy()),
                    None => {
                        stats.pathdiff_failures += 1;
                        continue;
                    }
                }
            };

            if new_import_path == imp.path {
                stats.no_op_skips += 1;
                continue;
            }

            let source_uri = match Url::from_file_path(&source_path) {
                Ok(u) => u,
                Err(_) => continue,
            };

            edits.entry(source_uri).or_default().push(TextEdit {
                range: range_with_quotes(imp.inner_range),
                new_text: format!("\"{}\"", new_import_path),
            });
        }
    }

    // ── Case 2: renamed files' own relative imports ─────────────────────
    for rename in renames {
        let old_dir = match rename.old_path.parent() {
            Some(d) => d,
            None => {
                stats.no_parent += 1;
                continue;
            }
        };
        let new_dir = match rename.new_path.parent() {
            Some(d) => d,
            None => {
                stats.no_parent += 1;
                continue;
            }
        };

        if old_dir == new_dir {
            continue;
        }

        let old_fs_str = match rename.old_path.to_str() {
            Some(s) => s,
            None => continue,
        };

        let bytes = match get_source_bytes(old_fs_str) {
            Some(b) => b,
            None => {
                stats.read_failures += 1;
                continue;
            }
        };

        let imports = links::ts_find_imports(&bytes);

        let old_uri = match Url::from_file_path(&rename.old_path) {
            Ok(u) => u,
            Err(_) => continue,
        };

        for imp in &imports {
            if !imp.path.starts_with('.') {
                continue;
            }

            let target_fs = normalize_path(&old_dir.join(&imp.path));
            let effective_target = rename_map.get(&target_fs).unwrap_or(&target_fs);

            let new_rel = match pathdiff::diff_paths(effective_target, new_dir) {
                Some(p) => p,
                None => {
                    stats.pathdiff_failures += 1;
                    continue;
                }
            };

            let new_import_str = ensure_dot_prefix(&new_rel);

            if new_import_str == imp.path {
                stats.no_op_skips += 1;
                continue;
            }

            let already_edited = edits.get(&old_uri).map_or(false, |file_edits| {
                let qr = range_with_quotes(imp.inner_range);
                file_edits.iter().any(|e| e.range == qr)
            });
            if already_edited {
                stats.dedup_skips += 1;
                continue;
            }

            edits.entry(old_uri.clone()).or_default().push(TextEdit {
                range: range_with_quotes(imp.inner_range),
                new_text: format!("\"{}\"", new_import_str),
            });
        }
    }

    RenameResult { edits, stats }
}

/// Compute import-statement removal edits needed when one or more files are
/// deleted.
///
/// For each Solidity source file, this scans all import directives and removes
/// the full import statement (`import ...;`) if it resolves to a deleted file.
///
/// This is intended for `workspace/willDeleteFiles` preview edits.
pub fn delete_imports(
    source_files: &[String],
    deletes: &[PathBuf],
    project_root: &Path,
    get_source_bytes: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> DeleteResult {
    let mut edits: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    let mut stats = DeleteStats::default();

    if deletes.is_empty() {
        return DeleteResult { edits, stats };
    }

    let mut delete_set: HashMap<PathBuf, ()> = HashMap::with_capacity(deletes.len());
    for p in deletes {
        if delete_set.insert(normalize_path(p), ()).is_some() {
            stats.duplicate_deletes += 1;
        }
    }

    for source_fs_str in source_files {
        let source_path = resolve_source_path(source_fs_str, project_root);
        let source_dir = match source_path.parent() {
            Some(d) => d,
            None => {
                stats.no_parent += 1;
                continue;
            }
        };

        let bytes = match source_path
            .to_str()
            .and_then(get_source_bytes)
            .or_else(|| get_source_bytes(source_fs_str))
        {
            Some(b) => b,
            None => {
                stats.read_failures += 1;
                continue;
            }
        };

        let source_str = match std::str::from_utf8(&bytes) {
            Ok(s) => s,
            Err(_) => {
                stats.read_failures += 1;
                continue;
            }
        };

        let imports = links::ts_find_imports(&bytes);
        let source_uri = match Url::from_file_path(&source_path) {
            Ok(u) => u,
            Err(_) => continue,
        };

        for imp in &imports {
            let resolved = normalize_path(&source_dir.join(&imp.path));

            let is_deleted = if delete_set.contains_key(&resolved) {
                true
            } else if !imp.path.starts_with('.') {
                let via_root = normalize_path(&project_root.join(&imp.path));
                delete_set.contains_key(&via_root)
            } else {
                false
            };

            if !is_deleted {
                continue;
            }

            let Some(statement_range) = import_statement_range(source_str, imp.inner_range) else {
                stats.statement_range_failures += 1;
                continue;
            };

            let duplicate = edits.get(&source_uri).map_or(false, |file_edits| {
                file_edits.iter().any(|e| e.range == statement_range)
            });
            if duplicate {
                stats.dedup_skips += 1;
                continue;
            }

            edits.entry(source_uri.clone()).or_default().push(TextEdit {
                range: statement_range,
                new_text: String::new(),
            });
        }
    }

    DeleteResult { edits, stats }
}

/// Backward-compatible wrapper: single rename, used by existing tests.
pub fn rename_imports_single(
    source_files: &[String],
    old_uri: &Url,
    new_uri: &Url,
    project_root: &Path,
    get_source_bytes: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> HashMap<Url, Vec<TextEdit>> {
    let old_path = match old_uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };
    let new_path = match new_uri.to_file_path() {
        Ok(p) => p,
        Err(_) => return HashMap::new(),
    };
    rename_imports(
        source_files,
        &[FileRename { old_path, new_path }],
        project_root,
        get_source_bytes,
    )
    .edits
}

// ---------------------------------------------------------------------------
// Cache patching
// ---------------------------------------------------------------------------

/// Apply computed edits to in-memory file content.
///
/// Returns the number of files patched (for logging).
pub fn apply_edits_to_cache(
    edits: &HashMap<Url, Vec<TextEdit>>,
    cache: &mut HashMap<crate::types::DocumentUri, (i32, String)>,
) -> usize {
    let mut patched = 0;
    for (uri, text_edits) in edits {
        let uri_str = uri.to_string();
        if let Some((version, content)) = cache.get(uri_str.as_str()).cloned() {
            let new_content = apply_text_edits(&content, text_edits);
            cache.insert(uri_str.into(), (version, new_content));
            patched += 1;
        }
    }
    patched
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Expand a range to include the surrounding quote characters.
fn range_with_quotes(inner: Range) -> Range {
    Range {
        start: Position {
            line: inner.start.line,
            character: inner.start.character.saturating_sub(1),
        },
        end: Position {
            line: inner.end.line,
            character: inner.end.character + 1,
        },
    }
}

/// Ensure a relative path starts with `./` or `../` for Solidity import convention.
/// Always uses forward slashes regardless of platform.
fn ensure_dot_prefix(rel: &Path) -> String {
    let s = normalize_slashes(&rel.to_string_lossy());
    if s.starts_with("..") || s.starts_with('.') {
        s
    } else {
        format!("./{s}")
    }
}

/// Resolve a source-file string to an absolute filesystem path.
///
/// Handles mixed forms produced by different indexers:
/// - absolute (`/repo/example/A.sol`)
/// - project-root relative (`A.sol`)
/// - workspace-root relative (`example/A.sol`)
fn resolve_source_path(source_fs_str: &str, project_root: &Path) -> PathBuf {
    let raw = Path::new(source_fs_str);
    if raw.is_absolute() {
        return normalize_path(raw);
    }

    let joined_project = normalize_path(&project_root.join(raw));
    if joined_project.exists() {
        return joined_project;
    }

    if let Ok(cwd) = std::env::current_dir() {
        let joined_cwd = normalize_path(&cwd.join(raw));
        if joined_cwd.exists() {
            return joined_cwd;
        }
    }

    joined_project
}

/// Replace backslashes with forward slashes for Solidity import paths.
/// Solidity uses forward slashes in import strings regardless of platform.
fn normalize_slashes(s: &str) -> String {
    s.replace('\\', "/")
}

/// Determine a range that covers the full import statement containing `inner`.
///
/// The returned range starts at the `import` keyword and ends at the
/// terminating `;`, plus one trailing newline when present.
fn import_statement_range(source: &str, inner: Range) -> Option<Range> {
    let start = utils::position_to_byte_offset(source, inner.start);
    let end = utils::position_to_byte_offset(source, inner.end);
    if start > end || end > source.len() {
        return None;
    }

    let bytes = source.as_bytes();
    let mut import_start = None;
    let mut i = start;
    while i > 0 {
        if i >= 6 && &bytes[i - 6..i] == b"import" {
            import_start = Some(i - 6);
            break;
        }
        if bytes[i - 1] == b';' {
            break;
        }
        i -= 1;
    }
    let import_start = import_start?;

    let mut semi = end;
    while semi < bytes.len() && bytes[semi] != b';' {
        semi += 1;
    }
    if semi >= bytes.len() || bytes[semi] != b';' {
        return None;
    }

    let mut import_end = semi + 1;
    if import_end + 1 < bytes.len() && bytes[import_end] == b'\r' && bytes[import_end + 1] == b'\n'
    {
        import_end += 2;
    } else if import_end < bytes.len() && bytes[import_end] == b'\n' {
        import_end += 1;
    }

    Some(Range {
        start: utils::byte_offset_to_position(source, import_start),
        end: utils::byte_offset_to_position(source, import_end),
    })
}

/// Apply a set of `TextEdit`s to a source string and return the new content.
///
/// Edits are sorted in reverse document order so that earlier byte offsets
/// remain valid as we splice in replacements from the end.
///
/// Positions are interpreted according to the negotiated encoding
/// (UTF-8 or UTF-16), matching the positions produced by `ts_find_imports`.
pub fn apply_text_edits(source: &str, edits: &[TextEdit]) -> String {
    // Convert to byte ranges on the original source.
    let mut resolved: Vec<(usize, usize, &str)> = edits
        .iter()
        .filter_map(|e| {
            let start = utils::position_to_byte_offset(source, e.range.start);
            let end = utils::position_to_byte_offset(source, e.range.end);
            if start > end {
                None
            } else {
                Some((start, end, e.new_text.as_str()))
            }
        })
        .collect();

    // Keep non-overlapping edits in forward order. For overlapping spans,
    // prefer the earliest-starting edit (wider edit when starts are equal).
    resolved.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    let mut filtered: Vec<(usize, usize, &str)> = Vec::with_capacity(resolved.len());
    for (start, end, new_text) in resolved {
        if let Some((_, last_end, _)) = filtered.last()
            && start < *last_end
        {
            continue;
        }
        filtered.push((start, end, new_text));
    }

    // Apply from back to front so earlier offsets remain valid.
    let mut result = source.to_string();
    for (start, end, new_text) in filtered.into_iter().rev() {
        result.replace_range(start..end, new_text);
    }
    result
}

// ---------------------------------------------------------------------------
// File scaffold generation
// ---------------------------------------------------------------------------

/// Generate scaffold content for a new `.sol` file.
///
/// Returns SPDX license identifier, pragma, and a stub contract/library/interface
/// named after the file. The `solc_version` from `foundry.toml` is used for
/// the pragma if available, otherwise defaults to `^0.8.0`.
///
/// `uri` is the file:// URI of the new file (used to derive the contract name).
pub fn generate_scaffold(uri: &Url, solc_version: Option<&str>) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    let stem = path.file_stem()?.to_str()?;

    // Only scaffold .sol files.
    let ext = path.extension()?;
    if ext != "sol" {
        return None;
    }

    let base_name = sanitize_identifier(stem);
    if base_name.is_empty() {
        return None;
    }

    // Derive pragma from solc_version.
    // "0.8.26" → "^0.8.26", already-prefixed values pass through.
    let pragma = match solc_version {
        Some(v) if !v.is_empty() => {
            let v = v.trim();
            if v.starts_with('^')
                || v.starts_with('>')
                || v.starts_with('<')
                || v.starts_with('=')
                || v.starts_with('~')
            {
                v.to_string()
            } else {
                format!("^{v}")
            }
        }
        _ => "^0.8.0".to_string(),
    };

    // Detect file kind from naming conventions.
    let is_test = stem.ends_with(".t");
    let is_script = stem.ends_with(".s");

    let kind = if is_test || is_script {
        // Foundry test/script files must always be contracts because they
        // inherit from Test/Script.
        "contract"
    } else if stem.starts_with('I')
        && stem.len() > 1
        && stem.chars().nth(1).map_or(false, |c| c.is_uppercase())
    {
        "interface"
    } else if stem.starts_with("Lib") || stem.starts_with("lib") {
        "library"
    } else {
        "contract"
    };

    let contract_name = if is_test {
        format!("{base_name}Test")
    } else if is_script {
        format!("{base_name}Script")
    } else {
        base_name
    };

    if is_test {
        Some(format!(
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity {pragma};\n\
             \n\
             import {{Test}} from \"forge-std/Test.sol\";\n\
             \n\
             {kind} {contract_name} is Test {{\n\
             \n\
             }}\n"
        ))
    } else if is_script {
        Some(format!(
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity {pragma};\n\
             \n\
             import {{Script}} from \"forge-std/Script.sol\";\n\
             \n\
             {kind} {contract_name} is Script {{\n\
             \n\
             }}\n"
        ))
    } else {
        Some(format!(
            "// SPDX-License-Identifier: MIT\n\
             pragma solidity {pragma};\n\
             \n\
             {kind} {contract_name} {{\n\
             \n\
             }}\n"
        ))
    }
}

/// Convert a filename stem to a valid Solidity identifier.
///
/// Strips `.t` and `.s` suffixes (Foundry test/script convention), removes
/// non-alphanumeric/underscore characters, and ensures the result doesn't
/// start with a digit.
fn sanitize_identifier(stem: &str) -> String {
    // Strip common Foundry suffixes: "Foo.t" → "Foo", "Bar.s" → "Bar"
    let stem = stem
        .strip_suffix(".t")
        .or_else(|| stem.strip_suffix(".s"))
        .unwrap_or(stem);

    let mut result = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        }
    }
    // Identifiers can't start with a digit.
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, '_');
    }
    // Avoid Solidity keywords as identifiers.
    if !result.is_empty() && !utils::is_valid_solidity_identifier(&result) {
        result.insert(0, '_');
    }
    result
}

/// Normalize a path by resolving `.` and `..` components without requiring
/// the file to exist on disk (unlike `std::fs::canonicalize`).
///
/// Guards against excessive `..` that would pop past the root by only
/// popping `Normal` components (never `RootDir` or `Prefix`).
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if matches!(components.last(), Some(std::path::Component::Normal(_))) {
                    components.pop();
                }
            }
            other => components.push(other),
        }
    }
    components.iter().collect()
}
