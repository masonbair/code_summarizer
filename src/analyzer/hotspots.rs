use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::structure::{FileInfo, ProjectAnalyzer};

/// Represents a "hot" file - one that is complex or frequently changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub file_path: PathBuf,
    pub relative_path: PathBuf,
    pub lines_of_code: usize,
    pub change_count: usize,
    pub dependency_count: usize,
    pub complexity_score: f64,
    pub hotness_score: f64,
}

impl ProjectAnalyzer {
    /// Identify hotspot files in the project
    pub fn identify_hotspots(&self) -> Result<Vec<Hotspot>> {
        let structure = self.analyze_structure()?;
        let mut hotspots = Vec::new();

        for module in &structure.modules {
            for file in &module.files {
                let hotspot = self.calculate_file_hotspot(file)?;
                hotspots.push(hotspot);
            }
        }

        // Sort by hotness score descending
        hotspots.sort_by(|a, b| {
            b.hotness_score
                .partial_cmp(&a.hotness_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(hotspots)
    }

    /// Calculate hotspot metrics for a single file
    fn calculate_file_hotspot(&self, file: &FileInfo) -> Result<Hotspot> {
        // Get change count from git if available
        let change_count = if let Some(git) = self.git_client() {
            git.get_file_change_count(&file.path).unwrap_or(0)
        } else {
            0
        };

        // Get dependency count from code-index if available
        let dependency_count = if let Some(index) = self.index_client() {
            index.get_dependency_count(&file.path).unwrap_or(0)
        } else {
            0
        };

        let complexity_score = calculate_complexity(file.lines, dependency_count);
        let hotness_score = calculate_hotness(file.lines, change_count, dependency_count);

        Ok(Hotspot {
            file_path: file.path.clone(),
            relative_path: file.relative_path.clone(),
            lines_of_code: file.lines,
            change_count,
            dependency_count,
            complexity_score,
            hotness_score,
        })
    }
}

/// Calculate complexity score based on lines and dependencies
pub fn calculate_complexity(lines: usize, dependency_count: usize) -> f64 {
    let line_score = lines as f64 / 50.0;
    let dep_score = dependency_count as f64 * 0.5;
    line_score + dep_score
}

/// Calculate hotness score combining multiple metrics
/// Formula: hotness = (change_count * 2.0) + (lines / 100.0) + (dependency_count * 1.5)
pub fn calculate_hotness(lines: usize, change_count: usize, dependency_count: usize) -> f64 {
    let change_weight = change_count as f64 * 2.0;
    let size_weight = lines as f64 / 100.0;
    let dep_weight = dependency_count as f64 * 1.5;

    change_weight + size_weight + dep_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_complexity_zero_inputs() {
        let score = calculate_complexity(0, 0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_complexity_lines_only() {
        let score = calculate_complexity(100, 0);
        assert_eq!(score, 2.0); // 100/50 = 2.0
    }

    #[test]
    fn test_calculate_complexity_deps_only() {
        let score = calculate_complexity(0, 10);
        assert_eq!(score, 5.0); // 10 * 0.5 = 5.0
    }

    #[test]
    fn test_calculate_complexity_combined() {
        let score = calculate_complexity(100, 10);
        assert_eq!(score, 7.0); // 2.0 + 5.0
    }

    #[test]
    fn test_calculate_hotness_zero_inputs() {
        let score = calculate_hotness(0, 0, 0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_hotness_changes_weighted_heavily() {
        let score = calculate_hotness(0, 10, 0);
        assert_eq!(score, 20.0); // 10 * 2.0 = 20.0
    }

    #[test]
    fn test_calculate_hotness_lines_weighted_lightly() {
        let score = calculate_hotness(100, 0, 0);
        assert_eq!(score, 1.0); // 100/100 = 1.0
    }

    #[test]
    fn test_calculate_hotness_deps_weighted_medium() {
        let score = calculate_hotness(0, 0, 10);
        assert_eq!(score, 15.0); // 10 * 1.5 = 15.0
    }

    #[test]
    fn test_calculate_hotness_combined() {
        let score = calculate_hotness(100, 5, 4);
        // (5 * 2.0) + (100/100) + (4 * 1.5) = 10 + 1 + 6 = 17.0
        assert_eq!(score, 17.0);
    }

    #[test]
    fn test_hotspot_ordering() {
        // Higher hotness should come first when sorted
        let mut hotspots = vec![
            Hotspot {
                file_path: PathBuf::from("low.rs"),
                relative_path: PathBuf::from("low.rs"),
                lines_of_code: 50,
                change_count: 1,
                dependency_count: 1,
                complexity_score: 1.5,
                hotness_score: 4.0,
            },
            Hotspot {
                file_path: PathBuf::from("high.rs"),
                relative_path: PathBuf::from("high.rs"),
                lines_of_code: 200,
                change_count: 20,
                dependency_count: 10,
                complexity_score: 9.0,
                hotness_score: 57.0,
            },
        ];

        hotspots.sort_by(|a, b| {
            b.hotness_score
                .partial_cmp(&a.hotness_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(hotspots[0].file_path, PathBuf::from("high.rs"));
        assert_eq!(hotspots[1].file_path, PathBuf::from("low.rs"));
    }
}
