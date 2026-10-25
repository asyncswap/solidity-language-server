//! Findings from analysis tools, on the lines they're about: `<project root>/.formal/findings.sarif` (SARIF 2.1.0, as
//! concolity's `--sarif` and Formal Mind's agents write it). Results at level error, warning or note become
//! diagnostics; every result on a line (level `none` included: solved branch sides, untaken sides) shows in its hover.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

/// Where the findings file lives, relative to the project root.
pub const FILE: &str = ".formal/findings.sarif";

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// Absolute path of the file the result is about.
    pub path: PathBuf,
    /// 1-based, as SARIF counts.
    pub line: u32,
    pub level: String,
    pub rule: String,
    pub tool: String,
    pub message: String,
}

type Cache = Mutex<HashMap<PathBuf, (SystemTime, Arc<Vec<Finding>>)>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The project's findings, read again only when the file changed.
pub fn load(root: &Path) -> Arc<Vec<Finding>> {
    let file = root.join(FILE);
    let Ok(modified) = std::fs::metadata(&file).and_then(|m| m.modified()) else {
        return Arc::new(vec![]);
    };
    if let Some((at, f)) = cache().lock().unwrap().get(&file)
        && *at == modified
    {
        return f.clone();
    }
    let parsed = Arc::new(
        std::fs::read_to_string(&file)
            .ok()
            .map(|t| parse(root, &t))
            .unwrap_or_default(),
    );
    cache()
        .lock()
        .unwrap()
        .insert(file, (modified, parsed.clone()));
    parsed
}

/// Forgets the cached file, so the next `load` reads it (on a watched-file change).
pub fn invalidate(root: &Path) {
    cache().lock().unwrap().remove(&root.join(FILE));
}

/// A result's file: relative to the project root (`uriBaseId`), an absolute path, or a `file://` URI.
fn resolve(root: &Path, uri: &str) -> PathBuf {
    if let Ok(u) = tower_lsp::lsp_types::Url::parse(uri)
        && let Ok(p) = u.to_file_path()
    {
        return p;
    }
    let p = Path::new(uri);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

pub fn parse(root: &Path, text: &str) -> Vec<Finding> {
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(text) else {
        return vec![];
    };
    let mut out = vec![];
    for run in doc["runs"].as_array().into_iter().flatten() {
        let tool = run["tool"]["driver"]["name"]
            .as_str()
            .unwrap_or("analysis")
            .to_string();
        // A rule's default level, for results that don't say.
        let defaults: HashMap<&str, &str> = run["tool"]["driver"]["rules"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| {
                Some((
                    r["id"].as_str()?,
                    r["defaultConfiguration"]["level"].as_str()?,
                ))
            })
            .collect();
        for r in run["results"].as_array().into_iter().flatten() {
            let rule = r["ruleId"].as_str().unwrap_or("").to_string();
            let level = r["level"]
                .as_str()
                .or_else(|| defaults.get(rule.as_str()).copied())
                .unwrap_or("warning")
                .to_string();
            let message = r["message"]["text"].as_str().unwrap_or("").to_string();
            for loc in r["locations"].as_array().into_iter().flatten() {
                let phys = &loc["physicalLocation"];
                let (Some(uri), Some(line)) = (
                    phys["artifactLocation"]["uri"].as_str(),
                    phys["region"]["startLine"].as_u64(),
                ) else {
                    continue;
                };
                out.push(Finding {
                    path: resolve(root, uri),
                    line: line.max(1) as u32,
                    level: level.clone(),
                    rule: rule.clone(),
                    tool: tool.clone(),
                    message: message.clone(),
                });
            }
        }
    }
    out
}

fn same(a: &Path, b: &Path) -> bool {
    a == b || matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y)
}

/// The diagnostics for one file: each result at error, warning or note, over its line's text (`source` is the live
/// buffer, for the line's extent).
pub fn diagnostics(findings: &[Finding], file: &Path, source: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    findings
        .iter()
        .filter(|f| same(&f.path, file))
        .filter_map(|f| {
            let severity = match f.level.as_str() {
                "error" => DiagnosticSeverity::ERROR,
                "warning" => DiagnosticSeverity::WARNING,
                "note" => DiagnosticSeverity::INFORMATION,
                _ => return None,
            };
            let row = f.line - 1;
            let text = lines.get(row as usize).copied().unwrap_or("");
            let start = text.len() - text.trim_start().len();
            let utf16 = |s: &str| s.encode_utf16().count() as u32;
            Some(Diagnostic {
                range: Range {
                    start: Position::new(row, utf16(&text[..start])),
                    end: Position::new(row, utf16(text)),
                },
                severity: Some(severity),
                code: (!f.rule.is_empty()).then(|| NumberOrString::String(f.rule.clone())),
                source: Some(f.tool.clone()),
                message: f.message.clone(),
                ..Default::default()
            })
        })
        .collect()
}

/// Markdown for the hover on a line (0-based `row`): every result there, the hover-only ones too.
pub fn hover(findings: &[Finding], file: &Path, row: u32) -> Option<String> {
    let here: Vec<&Finding> = findings
        .iter()
        .filter(|f| f.line == row + 1 && same(&f.path, file))
        .collect();
    if here.is_empty() {
        return None;
    }
    let items: Vec<String> = here
        .iter()
        .map(|f| {
            let what = f
                .rule
                .rsplit('/')
                .next()
                .unwrap_or(&f.rule)
                .replace('-', " ");
            format!("**{}** · {what}  \n{}", f.tool, f.message)
        })
        .collect();
    Some(items.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SARIF: &str = r#"{ "version": "2.1.0", "runs": [ { "tool": { "driver": { "name": "concolity", "rules": [
        { "id": "concolity/solved-side", "defaultConfiguration": { "level": "none" } } ] } },
        "results": [
          { "ruleId": "concolity/test-fails-on-solved-input", "level": "error", "message": { "text": "fails with Panic(0x11)" },
            "locations": [ { "physicalLocation": { "artifactLocation": { "uri": "src/Magic.sol", "uriBaseId": "PROJECT" }, "region": { "startLine": 3 } } } ] },
          { "ruleId": "concolity/solved-side", "message": { "text": "takes the jump side with [7]" },
            "locations": [ { "physicalLocation": { "artifactLocation": { "uri": "src/Magic.sol" }, "region": { "startLine": 3 } } } ] }
        ] } ] }"#;

    #[test]
    fn errors_become_diagnostics_and_every_result_hovers() {
        let root = Path::new("/project");
        let f = parse(root, SARIF);
        assert_eq!(f.len(), 2);
        assert_eq!(f[1].level, "none", "a rule's default level");
        let file = root.join("src/Magic.sol");
        let d = diagnostics(&f, &file, "line one\nline two\n    z = z + 7;\n");
        assert_eq!(d.len(), 1, "level none stays out of diagnostics");
        assert_eq!(d[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(
            (d[0].range.start, d[0].range.end),
            (Position::new(2, 4), Position::new(2, 14))
        );
        assert_eq!(
            d[0].code,
            Some(NumberOrString::String(
                "concolity/test-fails-on-solved-input".into()
            ))
        );
        let h = hover(&f, &file, 2).unwrap();
        assert!(
            h.contains("Panic(0x11)") && h.contains("solved side") && h.contains("[7]"),
            "{h}"
        );
        assert!(hover(&f, &file, 1).is_none());
        assert!(diagnostics(&f, &root.join("src/Other.sol"), "").is_empty());
    }
}
