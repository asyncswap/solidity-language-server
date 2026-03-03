use crate::{
    build::build_output_to_diagnostics, config::LintSettings, lint::lint_output_to_diagnostics,
    solc::normalize_forge_output,
};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use thiserror::Error;
use tokio::process::Command;
use tower_lsp::{
    async_trait,
    lsp_types::{Diagnostic, Url},
};

pub struct ForgeRunner;

#[async_trait]
pub trait Runner: Send + Sync {
    async fn build(&self, file: &str) -> Result<serde_json::Value, RunnerError>;
    async fn lint(
        &self,
        file: &str,
        lint_settings: &LintSettings,
    ) -> Result<serde_json::Value, RunnerError>;
    async fn ast(&self, file: &str) -> Result<serde_json::Value, RunnerError>;
    async fn format(&self, file: &str) -> Result<String, RunnerError>;
    async fn get_build_diagnostics(&self, file: &Url) -> Result<Vec<Diagnostic>, RunnerError>;
    async fn get_lint_diagnostics(
        &self,
        file: &Url,
        lint_settings: &LintSettings,
    ) -> Result<Vec<Diagnostic>, RunnerError>;
}

#[async_trait]
impl Runner for ForgeRunner {
    async fn lint(
        &self,
        file_path: &str,
        lint_settings: &LintSettings,
    ) -> Result<serde_json::Value, RunnerError> {
        let mut cmd = Command::new("forge");
        cmd.arg("lint")
            .arg(file_path)
            .arg("--json")
            .env("FOUNDRY_DISABLE_NIGHTLY_WARNING", "1");

        // Pass --severity flags from settings
        for sev in &lint_settings.severity {
            cmd.args(["--severity", sev]);
        }

        // Pass --only-lint flags from settings
        for lint_id in &lint_settings.only {
            cmd.args(["--only-lint", lint_id]);
        }

        let output = cmd.output().await?;

        let stderr_str = String::from_utf8_lossy(&output.stderr);

        // Parse JSON output line by line
        let mut diagnostics = Vec::new();
        for line in stderr_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(value) => diagnostics.push(value),
                Err(_e) => {
                    continue;
                }
            }
        }

        Ok(serde_json::Value::Array(diagnostics))
    }

    async fn build(&self, file_path: &str) -> Result<serde_json::Value, RunnerError> {
        let output = Command::new("forge")
            .arg("build")
            .arg(file_path)
            .arg("--json")
            .arg("--no-cache")
            .arg("--ast")
            .arg("--ignore-eip-3860")
            .args(["--ignored-error-codes", "5574"])
            .env("FOUNDRY_DISABLE_NIGHTLY_WARNING", "1")
            .env("FOUNDRY_LINT_LINT_ON_BUILD", "false")
            .output()
            .await?;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parsed = parse_json_from_mixed_stdout(&stdout_str)?;

        Ok(parsed)
    }

    async fn ast(&self, file_path: &str) -> Result<serde_json::Value, RunnerError> {
        let output = Command::new("forge")
            .arg("build")
            .arg(file_path)
            .arg("--json")
            .arg("--no-cache")
            .arg("--ast")
            .arg("--ignore-eip-3860")
            .args(["--ignored-error-codes", "5574"])
            .env("FOUNDRY_DISABLE_NIGHTLY_WARNING", "1")
            .env("FOUNDRY_LINT_LINT_ON_BUILD", "false")
            .output()
            .await?;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let parsed = parse_json_from_mixed_stdout(&stdout_str)?;

        Ok(normalize_forge_output(parsed))
    }

    async fn format(&self, file_path: &str) -> Result<String, RunnerError> {
        let output = Command::new("forge")
            .arg("fmt")
            .arg(file_path)
            .arg("--check")
            .arg("--raw")
            .env("FOUNDRY_DISABLE_NIGHTLY_WARNING", "1")
            .output()
            .await?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        match output.status.code() {
            Some(0) => {
                // Already formatted, read the current file content
                tokio::fs::read_to_string(file_path)
                    .await
                    .map_err(|_| RunnerError::ReadError)
            }
            Some(1) => {
                // Needs formatting, stdout has the formatted content
                if stdout.is_empty() {
                    Err(RunnerError::CommandError(io::Error::other(format!(
                        "forge fmt unexpected empty output on {}: exit code {}, stderr: {}",
                        file_path, output.status, stderr
                    ))))
                } else {
                    Ok(stdout)
                }
            }
            _ => Err(RunnerError::CommandError(io::Error::other(format!(
                "forge fmt failed on {}: exit code {}, stderr: {}",
                file_path, output.status, stderr
            )))),
        }
    }

    async fn get_lint_diagnostics(
        &self,
        file: &Url,
        lint_settings: &LintSettings,
    ) -> Result<Vec<Diagnostic>, RunnerError> {
        let path: PathBuf = file.to_file_path().map_err(|_| RunnerError::InvalidUrl)?;
        let path_str = path.to_str().ok_or(RunnerError::InvalidUrl)?;
        let lint_output = self.lint(path_str, lint_settings).await?;
        let diagnostics = lint_output_to_diagnostics(&lint_output, path_str);
        Ok(diagnostics)
    }

    async fn get_build_diagnostics(&self, file: &Url) -> Result<Vec<Diagnostic>, RunnerError> {
        let path = file.to_file_path().map_err(|_| RunnerError::InvalidUrl)?;
        let path_str = path.to_str().ok_or(RunnerError::InvalidUrl)?;
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|_| RunnerError::ReadError)?;
        let build_output = self.build(path_str).await?;
        let diagnostics = build_output_to_diagnostics(&build_output, &path, &content, &[]);
        Ok(diagnostics)
    }
}

/// Parse JSON from forge stdout, tolerating non-JSON log lines before payload.
fn parse_json_from_mixed_stdout(stdout: &str) -> Result<serde_json::Value, RunnerError> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return Err(RunnerError::EmptyOutput);
    }

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Ok(v);
    }

    // Some forge/log integrations can print warnings to stdout before JSON.
    // Try parsing from each `{` onward and take the first valid JSON object.
    for (idx, ch) in trimmed.char_indices() {
        if ch != '{' {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&trimmed[idx..]) {
            return Ok(v);
        }
    }

    Err(RunnerError::JsonError(serde_json::Error::io(
        io::Error::other("failed to parse forge JSON from mixed stdout"),
    )))
}

#[cfg(test)]
mod tests {
    use super::parse_json_from_mixed_stdout;

    #[test]
    fn parse_json_from_mixed_stdout_accepts_plain_json() {
        let out = r#"{ "errors": [], "sources": {}, "contracts": {}, "build_infos": [] }"#;
        let parsed = parse_json_from_mixed_stdout(out).expect("valid JSON");
        assert!(parsed.get("errors").is_some());
    }

    #[test]
    fn parse_json_from_mixed_stdout_skips_leading_logs() {
        let out = r#"WARN cache write failed
{ "errors": [], "sources": {}, "contracts": {}, "build_infos": [] }"#;
        let parsed = parse_json_from_mixed_stdout(out).expect("mixed output should parse");
        assert!(parsed.get("sources").is_some());
    }
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Invalid file URL")]
    InvalidUrl,
    #[error("Failed to run command: {0}")]
    CommandError(#[from] io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Empty output from compiler")]
    EmptyOutput,
    #[error("ReadError")]
    ReadError,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourceLocation {
    file: String,
    start: i32, // Changed to i32 to handle -1 values
    end: i32,   // Changed to i32 to handle -1 values
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ForgeDiagnosticMessage {
    #[serde(rename = "sourceLocation")]
    source_location: SourceLocation,
    #[serde(rename = "type")]
    error_type: String,
    component: String,
    severity: String,
    #[serde(rename = "errorCode")]
    error_code: String,
    message: String,
    #[serde(rename = "formattedMessage")]
    formatted_message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompileOutput {
    errors: Option<Vec<ForgeDiagnosticMessage>>,
    sources: serde_json::Value,
    contracts: serde_json::Value,
    build_infos: Vec<serde_json::Value>,
}
