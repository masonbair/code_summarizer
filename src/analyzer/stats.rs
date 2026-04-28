use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::structure::ProjectAnalyzer;

/// Project-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_size_bytes: u64,
    pub language_breakdown: HashMap<String, usize>,
    pub module_count: usize,
    pub avg_file_size_lines: f64,
    pub largest_file: Option<FileStats>,
    pub smallest_file: Option<FileStats>,
}

/// Statistics for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    pub path: String,
    pub lines: usize,
    pub size_bytes: u64,
    pub language: String,
}

impl ProjectAnalyzer {
    /// Calculate comprehensive project statistics
    pub fn calculate_stats(&self) -> Result<ProjectStats> {
        let structure = self.analyze_structure()?;

        let mut total_files = 0;
        let mut total_lines = 0;
        let mut total_size_bytes = 0u64;
        let mut largest_file: Option<FileStats> = None;
        let mut smallest_file: Option<FileStats> = None;

        for module in &structure.modules {
            for file in &module.files {
                total_files += 1;
                total_lines += file.lines;
                total_size_bytes += file.size_bytes;

                let file_stats = FileStats {
                    path: file.relative_path.to_string_lossy().to_string(),
                    lines: file.lines,
                    size_bytes: file.size_bytes,
                    language: file.language.clone(),
                };

                // Track largest file
                if largest_file.is_none() || file.lines > largest_file.as_ref().unwrap().lines {
                    largest_file = Some(file_stats.clone());
                }

                // Track smallest file (excluding empty files)
                if file.lines > 0
                    && (smallest_file.is_none() || file.lines < smallest_file.as_ref().unwrap().lines)
                    {
                        smallest_file = Some(file_stats);
                    }
            }
        }

        let avg_file_size_lines = if total_files > 0 {
            total_lines as f64 / total_files as f64
        } else {
            0.0
        };

        Ok(ProjectStats {
            total_files,
            total_lines,
            total_size_bytes,
            language_breakdown: structure.language_breakdown,
            module_count: structure.modules.len(),
            avg_file_size_lines,
            largest_file,
            smallest_file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn sub(a: i32, b: i32) -> i32 {\n    a - b\n}\n",
        )
        .unwrap();

        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/test.rs"), "#[test]\nfn it_works() {}\n").unwrap();

        temp
    }

    #[test]
    fn test_calculate_stats_total_files() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        assert_eq!(stats.total_files, 3); // main.rs, lib.rs, test.rs
    }

    #[test]
    fn test_calculate_stats_total_lines() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        // main.rs: 3 lines, lib.rs: 7 lines (including blank line), test.rs: 2 lines
        assert_eq!(stats.total_lines, 12);
    }

    #[test]
    fn test_calculate_stats_language_breakdown() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        assert_eq!(stats.language_breakdown.get("Rust"), Some(&3));
    }

    #[test]
    fn test_calculate_stats_module_count() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        // src and tests are two modules
        assert_eq!(stats.module_count, 2);
    }

    #[test]
    fn test_calculate_stats_avg_file_size() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        // 12 total lines / 3 files = 4.0 avg
        let expected_avg = 12.0 / 3.0;
        assert!((stats.avg_file_size_lines - expected_avg).abs() < 0.01);
    }

    #[test]
    fn test_calculate_stats_largest_file() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        let largest = stats.largest_file.unwrap();
        assert_eq!(largest.lines, 7); // lib.rs has 7 lines (including blank line)
        assert!(largest.path.contains("lib.rs"));
    }

    #[test]
    fn test_calculate_stats_smallest_file() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        let smallest = stats.smallest_file.unwrap();
        assert_eq!(smallest.lines, 2); // test.rs has 2 lines
        assert!(smallest.path.contains("test.rs"));
    }

    #[test]
    fn test_empty_project_stats() {
        let temp = TempDir::new().unwrap();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let stats = analyzer.calculate_stats().unwrap();

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.avg_file_size_lines, 0.0);
        assert!(stats.largest_file.is_none());
        assert!(stats.smallest_file.is_none());
    }
}
