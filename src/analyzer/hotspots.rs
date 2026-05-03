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
    /// Enhanced metrics (optional, may not be available for all files)
    pub enhanced_metrics: Option<EnhancedMetrics>,
}

/// Enhanced complexity metrics using static analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMetrics {
    /// Cyclomatic complexity (count of branches: if, match, for, while, etc.)
    pub cyclomatic_complexity: usize,
    /// Maximum nesting depth
    pub max_nesting_depth: usize,
    /// Number of public API items
    pub public_api_surface: usize,
    /// Number of functions/methods
    pub function_count: usize,
    /// Average function length
    pub avg_function_length: f64,
    /// Estimated test coverage (if test files detected)
    pub test_coverage_estimate: Option<f64>,
    /// Priority level based on combined metrics
    pub priority: HotspotPriority,
    /// Recommendations for improving the file
    pub recommendations: Vec<String>,
}

/// Priority level for hotspot attention
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HotspotPriority {
    Low,
    Medium,
    High,
    Critical,
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

        // Calculate enhanced metrics if semantic info available
        let enhanced_metrics = self.calculate_enhanced_metrics(file, change_count, dependency_count);

        // Calculate complexity using enhanced metrics if available
        let complexity_score = if let Some(ref metrics) = enhanced_metrics {
            calculate_enhanced_complexity(
                file.lines,
                dependency_count,
                metrics.cyclomatic_complexity,
                metrics.max_nesting_depth,
            )
        } else {
            calculate_complexity(file.lines, dependency_count)
        };

        let hotness_score = if let Some(ref metrics) = enhanced_metrics {
            calculate_enhanced_hotness(
                file.lines,
                change_count,
                dependency_count,
                metrics.cyclomatic_complexity,
                metrics.public_api_surface,
            )
        } else {
            calculate_hotness(file.lines, change_count, dependency_count)
        };

        Ok(Hotspot {
            file_path: file.path.clone(),
            relative_path: file.relative_path.clone(),
            lines_of_code: file.lines,
            change_count,
            dependency_count,
            complexity_score,
            hotness_score,
            enhanced_metrics,
        })
    }

    /// Calculate enhanced metrics from file semantics
    fn calculate_enhanced_metrics(
        &self,
        file: &FileInfo,
        change_count: usize,
        dependency_count: usize,
    ) -> Option<EnhancedMetrics> {
        let semantics = file.semantics.as_ref()?;

        // Count functions and calculate average length
        let function_count = semantics.public_apis.len();
        let avg_function_length = if function_count > 0 {
            file.lines as f64 / function_count as f64
        } else {
            file.lines as f64
        };

        // Public API surface
        let public_api_surface = semantics.public_apis.len()
            + semantics.traits.len()
            + semantics.types.len();

        // Estimate cyclomatic complexity from file content
        let (cyclomatic_complexity, max_nesting_depth) = estimate_complexity_from_file(&file.path);

        // Generate recommendations
        let mut recommendations = Vec::new();

        if file.lines > 400 {
            recommendations.push("Consider splitting into smaller modules".to_string());
        }

        if cyclomatic_complexity > 15 {
            recommendations.push("High branching complexity - consider refactoring conditional logic".to_string());
        }

        if max_nesting_depth > 4 {
            recommendations.push("Deep nesting detected - consider early returns or guard clauses".to_string());
        }

        if dependency_count > 10 {
            recommendations.push("High coupling - consider dependency injection or interface abstractions".to_string());
        }

        if change_count > 20 {
            recommendations.push("Frequently changed file - add tests to prevent regressions".to_string());
        }

        // Determine priority
        let priority = determine_priority(
            file.lines,
            cyclomatic_complexity,
            max_nesting_depth,
            change_count,
            dependency_count,
        );

        Some(EnhancedMetrics {
            cyclomatic_complexity,
            max_nesting_depth,
            public_api_surface,
            function_count,
            avg_function_length,
            test_coverage_estimate: None, // Would need test file analysis
            priority,
            recommendations,
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

/// Enhanced complexity calculation including cyclomatic complexity and nesting
pub fn calculate_enhanced_complexity(
    lines: usize,
    dependency_count: usize,
    cyclomatic_complexity: usize,
    max_nesting_depth: usize,
) -> f64 {
    let line_score = lines as f64 / 50.0;
    let dep_score = dependency_count as f64 * 0.5;
    let cyclo_score = cyclomatic_complexity as f64 * 1.0;
    let nesting_score = (max_nesting_depth as f64).powi(2) * 0.5;

    line_score + dep_score + cyclo_score + nesting_score
}

/// Enhanced hotness calculation with more metrics
pub fn calculate_enhanced_hotness(
    lines: usize,
    change_count: usize,
    dependency_count: usize,
    cyclomatic_complexity: usize,
    public_api_surface: usize,
) -> f64 {
    let change_weight = change_count as f64 * 2.0;
    let size_weight = lines as f64 / 100.0;
    let dep_weight = dependency_count as f64 * 1.5;
    let cyclo_weight = cyclomatic_complexity as f64 * 0.8;
    let api_weight = public_api_surface as f64 * 0.3;

    change_weight + size_weight + dep_weight + cyclo_weight + api_weight
}

/// Estimate cyclomatic complexity and max nesting from file content
fn estimate_complexity_from_file(path: &std::path::Path) -> (usize, usize) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Count branch keywords based on language
    let branch_patterns: &[&str] = match ext {
        "rs" => &["if ", "else ", "match ", "for ", "while ", "loop ", "?"],
        "py" => &["if ", "elif ", "else:", "for ", "while ", "except ", "try:"],
        "ts" | "tsx" | "js" | "jsx" => &["if ", "else ", "switch ", "for ", "while ", "catch ", "? "],
        "go" => &["if ", "else ", "switch ", "for ", "select "],
        _ => &["if ", "else ", "for ", "while "],
    };

    let mut cyclomatic = 1; // Base complexity
    for pattern in branch_patterns {
        cyclomatic += content.matches(pattern).count();
    }

    // Estimate max nesting by counting indentation levels
    let mut max_nesting = 0;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        let indent = line.len() - trimmed.len();
        let indent_units = if line.contains('\t') {
            indent // Tab-based
        } else {
            indent / 4 // Assume 4-space indent
        };

        // Track nesting based on indentation and block openers
        let current_nesting = if trimmed.ends_with('{') || trimmed.ends_with(':') || trimmed.ends_with("do") {
            indent_units + 1
        } else {
            indent_units
        };

        if current_nesting > max_nesting {
            max_nesting = current_nesting;
        }
    }

    (cyclomatic, max_nesting)
}

/// Determine priority level based on combined metrics
fn determine_priority(
    lines: usize,
    cyclomatic_complexity: usize,
    max_nesting_depth: usize,
    change_count: usize,
    dependency_count: usize,
) -> HotspotPriority {
    let mut score = 0;

    // Size factors
    if lines > 500 {
        score += 3;
    } else if lines > 300 {
        score += 2;
    } else if lines > 150 {
        score += 1;
    }

    // Complexity factors
    if cyclomatic_complexity > 25 {
        score += 3;
    } else if cyclomatic_complexity > 15 {
        score += 2;
    } else if cyclomatic_complexity > 8 {
        score += 1;
    }

    // Nesting factors
    if max_nesting_depth > 5 {
        score += 2;
    } else if max_nesting_depth > 3 {
        score += 1;
    }

    // Change frequency factors
    if change_count > 30 {
        score += 2;
    } else if change_count > 15 {
        score += 1;
    }

    // Coupling factors
    if dependency_count > 15 {
        score += 2;
    } else if dependency_count > 8 {
        score += 1;
    }

    match score {
        0..=2 => HotspotPriority::Low,
        3..=5 => HotspotPriority::Medium,
        6..=8 => HotspotPriority::High,
        _ => HotspotPriority::Critical,
    }
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
                enhanced_metrics: None,
            },
            Hotspot {
                file_path: PathBuf::from("high.rs"),
                relative_path: PathBuf::from("high.rs"),
                lines_of_code: 200,
                change_count: 20,
                dependency_count: 10,
                complexity_score: 9.0,
                hotness_score: 57.0,
                enhanced_metrics: None,
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

    #[test]
    fn test_enhanced_complexity() {
        let score = calculate_enhanced_complexity(100, 5, 10, 3);
        // 100/50 + 5*0.5 + 10*1.0 + 3^2*0.5 = 2 + 2.5 + 10 + 4.5 = 19.0
        assert_eq!(score, 19.0);
    }

    #[test]
    fn test_determine_priority_low() {
        let priority = determine_priority(100, 5, 2, 5, 3);
        assert_eq!(priority, HotspotPriority::Low);
    }

    #[test]
    fn test_determine_priority_critical() {
        let priority = determine_priority(600, 30, 6, 35, 20);
        assert_eq!(priority, HotspotPriority::Critical);
    }
}
