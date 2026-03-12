use crate::utils::byte_offset_to_position;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};

/// Default error codes that are always suppressed (contract-size and
/// code-size warnings that are noisy for LSP users).
const DEFAULT_IGNORED_CODES: &[&str] = &["5574", "3860"];

/// Check whether a solc error should be suppressed based on its error code.
///
/// Suppresses the hardcoded defaults plus any codes provided in `extra_codes`
/// (from `foundry.toml` `ignored_error_codes`).
pub fn ignored_error_code_warning(value: &serde_json::Value, extra_codes: &[u64]) -> bool {
    let error_code = value
        .get("errorCode")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    if DEFAULT_IGNORED_CODES.contains(&error_code) {
        return true;
    }

    // Check user-configured ignored codes from foundry.toml
    if let Ok(code_num) = error_code.parse::<u64>()
        && extra_codes.contains(&code_num)
    {
        return true;
    }

    false
}

pub fn build_output_to_diagnostics(
    solc_output: &serde_json::Value,
    path: impl AsRef<Path>,
    content: &str,
    ignored_error_codes: &[u64],
) -> Vec<Diagnostic> {
    let Some(errors) = solc_output.get("errors").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let path = path.as_ref();
    errors
        .iter()
        .filter_map(|err| parse_diagnostic(err, path, content, ignored_error_codes))
        .collect()
}

/// Check whether the source path from solc's error output refers to the same
/// file the editor has open.
///
/// Solc reports error paths relative to its working directory (wherever the
/// LSP process runs from), e.g. `example/Shop.sol` or just `Shop.sol`.  The
/// editor provides the full absolute path.  We simply check whether the
/// absolute path ends with the relative path solc reported.
fn source_location_matches(source_path: &str, path: &Path) -> bool {
    let source_path = Path::new(source_path);
    if source_path.is_absolute() {
        source_path == path
    } else {
        path.ends_with(source_path)
    }
}

fn parse_diagnostic(
    err: &Value,
    path: &Path,
    content: &str,
    ignored_error_codes: &[u64],
) -> Option<Diagnostic> {
    if ignored_error_code_warning(err, ignored_error_codes) {
        return None;
    }
    let source_file = err
        .get("sourceLocation")
        .and_then(|loc| loc.get("file"))
        .and_then(|f| f.as_str())?;

    if !source_location_matches(source_file, path) {
        return None;
    }

    let start_offset = err
        .get("sourceLocation")
        .and_then(|loc| loc.get("start"))
        .and_then(|s| s.as_u64())
        .unwrap_or(0) as usize;

    let end_offset = err
        .get("sourceLocation")
        .and_then(|loc| loc.get("end"))
        .and_then(|s| s.as_u64())
        .map(|v| v as usize)
        .unwrap_or(start_offset);

    let start = byte_offset_to_position(content, start_offset);
    let end = byte_offset_to_position(content, end_offset);

    let range = Range { start, end };

    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown error");

    let severity = match err.get("severity").and_then(|s| s.as_str()) {
        Some("error") => Some(DiagnosticSeverity::ERROR),
        Some("warning") => Some(DiagnosticSeverity::WARNING),
        Some("note") => Some(DiagnosticSeverity::INFORMATION),
        Some("help") => Some(DiagnosticSeverity::HINT),
        _ => Some(DiagnosticSeverity::INFORMATION),
    };

    let code = err
        .get("errorCode")
        .and_then(|c| c.as_str())
        .map(|s| NumberOrString::String(s.to_string()));

    Some(Diagnostic {
        range,
        severity,
        code,
        code_description: None,
        source: Some("solc".to_string()),
        message: message.to_string(),
        related_information: None,
        tags: None,
        data: None,
    })
}

/// Extract error-level diagnostics for files OTHER than the one being compiled.
///
/// When compiling `A.sol`, solc may report errors in imported files (e.g.
/// `B.sol` has `import {Test} from "./A.sol"` but `Test` was removed).
/// `build_output_to_diagnostics` filters those out.  This function collects
/// them so the LSP can publish diagnostics to the affected files.
///
/// Returns a map of `absolute_path → Vec<Diagnostic>`.  Only error-severity
/// diagnostics are included.  The `project_root` resolves relative paths.
pub fn cross_file_error_diagnostics(
    solc_output: &Value,
    current_file: &Path,
    project_root: &Path,
    ignored_error_codes: &[u64],
) -> HashMap<PathBuf, Vec<Diagnostic>> {
    let Some(errors) = solc_output.get("errors").and_then(|v| v.as_array()) else {
        return HashMap::new();
    };

    let mut result: HashMap<PathBuf, Vec<Diagnostic>> = HashMap::new();

    for err in errors {
        if ignored_error_code_warning(err, ignored_error_codes) {
            continue;
        }
        if err.get("severity").and_then(|s| s.as_str()) != Some("error") {
            continue;
        }
        let Some(source_file) = err
            .get("sourceLocation")
            .and_then(|loc| loc.get("file"))
            .and_then(|f| f.as_str())
        else {
            continue;
        };
        if source_location_matches(source_file, current_file) {
            continue;
        }

        let source_path = Path::new(source_file);
        let abs_path = if source_path.is_absolute() {
            source_path.to_path_buf()
        } else {
            project_root.join(source_path)
        };
        let Ok(content) = std::fs::read_to_string(&abs_path) else {
            continue;
        };

        let start_offset = err
            .get("sourceLocation")
            .and_then(|loc| loc.get("start"))
            .and_then(|s| s.as_u64())
            .unwrap_or(0) as usize;
        let end_offset = err
            .get("sourceLocation")
            .and_then(|loc| loc.get("end"))
            .and_then(|s| s.as_u64())
            .map(|v| v as usize)
            .unwrap_or(start_offset);

        let start = byte_offset_to_position(&content, start_offset);
        let end = byte_offset_to_position(&content, end_offset);

        let code = err
            .get("errorCode")
            .and_then(|c| c.as_str())
            .map(|s| NumberOrString::String(s.to_string()));

        let message = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        result.entry(abs_path).or_default().push(Diagnostic {
            range: Range { start, end },
            severity: Some(DiagnosticSeverity::ERROR),
            code,
            code_description: None,
            source: Some("solc".to_string()),
            message: message.to_string(),
            related_information: None,
            tags: None,
            data: None,
        });
    }

    result
}
