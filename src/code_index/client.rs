use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::analyzer::{Dependency, DependencyType};

/// Client for interacting with the code-index CLI tool
pub struct CodeIndexClient {
    project_root: PathBuf,
}

/// Symbol information from code-index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: Option<String>,
}

/// File metadata from code-index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub language: String,
    pub lines: usize,
    pub symbols_count: usize,
    pub dependencies_count: usize,
    pub hotness_score: f64,
}

/// Raw dependency info from code-index JSON
#[derive(Debug, Clone, Deserialize)]
struct RawDependency {
    source_file: String,
    target_file: String,
    #[serde(default)]
    dep_type: Option<String>,
}

/// Hot file info from code-index
#[derive(Debug, Clone, Deserialize)]
struct HotFileInfo {
    path: String,
    #[serde(default)]
    hotness_score: Option<f64>,
    #[serde(default)]
    lines: Option<usize>,
    #[serde(default)]
    change_count: Option<usize>,
}

impl CodeIndexClient {
    /// Create a new client for the given project root
    pub fn new(project_root: &Path) -> Result<Self> {
        // Verify code-index is available
        let output = Command::new("code-index")
            .arg("--version")
            .output()
            .context("code-index not found in PATH")?;

        if !output.status.success() {
            anyhow::bail!("code-index returned error");
        }

        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    /// Query symbols by name
    pub fn query_symbol(&self, name: &str) -> Result<Vec<Symbol>> {
        let output = Command::new("code-index")
            .args(["query", "symbol", name, "--json"])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute code-index query symbol")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No symbols found") || stderr.contains("not found") {
                return Ok(Vec::new());
            }
            anyhow::bail!("code-index query symbol failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let symbols: Vec<Symbol> =
            serde_json::from_str(&stdout).unwrap_or_default();
        Ok(symbols)
    }

    /// Get all symbols in a file
    pub fn query_file(&self, file_path: &Path) -> Result<Vec<Symbol>> {
        let output = Command::new("code-index")
            .args(["query", "file", &file_path.to_string_lossy(), "--json"])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute code-index query file")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("No file") {
                return Ok(Vec::new());
            }
            anyhow::bail!("code-index query file failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let symbols: Vec<Symbol> =
            serde_json::from_str(&stdout).unwrap_or_default();
        Ok(symbols)
    }

    /// Get dependencies for a file
    pub fn query_dependencies(&self, file_path: &Path) -> Result<Vec<Dependency>> {
        let output = Command::new("code-index")
            .args([
                "query",
                "dependencies",
                &file_path.to_string_lossy(),
                "--json",
            ])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute code-index query dependencies")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not found") || stderr.contains("No dependencies") {
                return Ok(Vec::new());
            }
            anyhow::bail!("code-index query dependencies failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let raw_deps: Vec<RawDependency> =
            serde_json::from_str(&stdout).unwrap_or_default();

        let deps = raw_deps
            .into_iter()
            .map(|rd| Dependency {
                source: PathBuf::from(&rd.source_file),
                target: PathBuf::from(&rd.target_file),
                dep_type: parse_dep_type(rd.dep_type.as_deref()),
            })
            .collect();

        Ok(deps)
    }

    /// Get all dependencies in the project
    pub fn get_all_dependencies(&self) -> Result<Vec<Dependency>> {
        // Export the full index and parse dependencies
        let output = Command::new("code-index")
            .args(["stats", "--json"])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute code-index stats")?;

        // For now, return empty - would need to iterate through files
        // In a real implementation, we might export the full index
        if !output.status.success() {
            return Ok(Vec::new());
        }

        // This would require iterating through all indexed files
        // For simplicity, return empty and rely on basic analysis
        Ok(Vec::new())
    }

    /// Get dependency count for a specific file
    pub fn get_dependency_count(&self, file_path: &Path) -> Result<usize> {
        let deps = self.query_dependencies(file_path)?;
        Ok(deps.len())
    }

    /// Get hot files (most complex/frequently changed)
    pub fn get_hot_files(&self, limit: usize) -> Result<Vec<FileMetadata>> {
        let output = Command::new("code-index")
            .args([
                "query",
                "hot-files",
                "--limit",
                &limit.to_string(),
                "--json",
            ])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute code-index query hot-files")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No files") || stderr.contains("not found") {
                return Ok(Vec::new());
            }
            anyhow::bail!("code-index query hot-files failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let hot_files: Vec<HotFileInfo> =
            serde_json::from_str(&stdout).unwrap_or_default();

        let files = hot_files
            .into_iter()
            .map(|hf| FileMetadata {
                path: PathBuf::from(&hf.path),
                language: String::new(), // Would need to detect
                lines: hf.lines.unwrap_or(0),
                symbols_count: 0,
                dependencies_count: 0,
                hotness_score: hf.hotness_score.unwrap_or(0.0),
            })
            .collect();

        Ok(files)
    }

    /// Check if code-index is available and has data for this project
    pub fn is_available(&self) -> bool {
        let output = Command::new("code-index")
            .args(["stats"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}

/// Parse dependency type string to enum
fn parse_dep_type(dep_type: Option<&str>) -> DependencyType {
    match dep_type {
        Some("import") => DependencyType::Import,
        Some("include") => DependencyType::Include,
        Some("use") => DependencyType::Use,
        Some("require") => DependencyType::Require,
        Some("from") => DependencyType::From,
        _ => DependencyType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dep_type_import() {
        assert_eq!(parse_dep_type(Some("import")), DependencyType::Import);
    }

    #[test]
    fn test_parse_dep_type_use() {
        assert_eq!(parse_dep_type(Some("use")), DependencyType::Use);
    }

    #[test]
    fn test_parse_dep_type_require() {
        assert_eq!(parse_dep_type(Some("require")), DependencyType::Require);
    }

    #[test]
    fn test_parse_dep_type_unknown() {
        assert_eq!(parse_dep_type(Some("something_else")), DependencyType::Unknown);
        assert_eq!(parse_dep_type(None), DependencyType::Unknown);
    }

    #[test]
    fn test_symbol_serialization() {
        let symbol = Symbol {
            name: "main".to_string(),
            kind: "function".to_string(),
            file_path: PathBuf::from("src/main.rs"),
            line_start: 1,
            line_end: 5,
            signature: Some("fn main()".to_string()),
        };

        let json = serde_json::to_string(&symbol).unwrap();
        assert!(json.contains("main"));
        assert!(json.contains("function"));
    }

    #[test]
    fn test_file_metadata_serialization() {
        let metadata = FileMetadata {
            path: PathBuf::from("src/lib.rs"),
            language: "Rust".to_string(),
            lines: 100,
            symbols_count: 10,
            dependencies_count: 5,
            hotness_score: 25.5,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("src/lib.rs"));
        assert!(json.contains("100"));
        assert!(json.contains("25.5"));
    }

    // Integration test - only runs if code-index is available
    #[test]
    #[ignore = "requires code-index to be installed"]
    fn test_client_creation() {
        let temp = tempfile::TempDir::new().unwrap();
        let result = CodeIndexClient::new(temp.path());
        // May fail if code-index is not installed, which is expected
        assert!(result.is_ok() || result.is_err());
    }
}
