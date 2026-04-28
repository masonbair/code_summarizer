mod structure;
mod hotspots;
mod dependencies;
mod stats;

pub use structure::*;
pub use hotspots::*;
pub use dependencies::*;
pub use stats::*;

use anyhow::Result;
use std::path::Path;

/// Show project statistics
pub fn show_stats(project_root: &Path) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let stats = analyzer.calculate_stats()?;

    println!("Project Statistics");
    println!("==================");
    println!("Total files: {}", stats.total_files);
    println!("Total lines: {}", stats.total_lines);
    println!("\nLanguage breakdown:");
    for (lang, count) in &stats.language_breakdown {
        let pct = (*count as f64 / stats.total_files as f64) * 100.0;
        println!("  {}: {} files ({:.1}%)", lang, count, pct);
    }

    Ok(())
}

/// Show dependency graph
pub fn show_deps(project_root: &Path, format: &str) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let graph = analyzer.build_dependency_graph()?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&graph)?);
        }
        _ => {
            println!("{}", graph.to_ascii());
        }
    }

    Ok(())
}

/// Show hotspot files
pub fn show_hotspots(project_root: &Path, limit: usize, sort_by: &str) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let mut hotspots = analyzer.identify_hotspots()?;

    // Sort based on criteria
    match sort_by {
        "complexity" => hotspots.sort_by(|a, b| b.complexity_score.partial_cmp(&a.complexity_score).unwrap()),
        "changes" => hotspots.sort_by(|a, b| b.change_count.cmp(&a.change_count)),
        _ => hotspots.sort_by(|a, b| b.hotness_score.partial_cmp(&a.hotness_score).unwrap()),
    }

    println!("Top {} Hotspots (sorted by {}):", limit, sort_by);
    println!();

    for (i, hotspot) in hotspots.iter().take(limit).enumerate() {
        println!("{}. {}", i + 1, hotspot.file_path.display());
        println!("   - Lines: {}", hotspot.lines_of_code);
        println!("   - Changes: {}", hotspot.change_count);
        println!("   - Complexity: {:.1}", hotspot.complexity_score);
        println!("   - Hotness: {:.1}", hotspot.hotness_score);
        println!();
    }

    Ok(())
}
