mod architecture;
mod modules;
mod deps_graph;
mod hotspots_gen;


use anyhow::{Context, Result};
use log::{info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::analyzer::ProjectAnalyzer;
use crate::git::GitClient;
use crate::llm;
use crate::templates::TemplateRenderer;

/// Output tier for controlling token usage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTier {
    /// Minimal output (~200 tokens) - just structure
    Minimal,
    /// Standard output (~1000 tokens) - structure + key types
    Standard,
    /// Full output (~3000 tokens) - everything including APIs
    Full,
}

impl FromStr for OutputTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" => Ok(OutputTier::Minimal),
            "standard" => Ok(OutputTier::Standard),
            "full" => Ok(OutputTier::Full),
            _ => Err(format!("Unknown tier: {}", s)),
        }
    }
}

impl Default for OutputTier {
    fn default() -> Self {
        OutputTier::Standard
    }
}

/// Options for controlling generation output
#[derive(Debug, Clone)]
pub struct GenerationOptions {
    /// Output tier (minimal, standard, full)
    pub tier: OutputTier,
    /// Custom token budget (overrides tier)
    pub token_budget: Option<usize>,
    /// Whether to perform semantic extraction
    pub semantic_extraction: bool,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            tier: OutputTier::Standard,
            token_budget: None,
            semantic_extraction: true,
        }
    }
}

/// Generate all summaries for a project
pub fn generate_all(
    project_root: &Path,
    output_dir: &Path,
    custom_template: Option<PathBuf>,
    llm_provider: Option<String>,
    llm_model: String,
    llm_endpoint: Option<String>,
    no_llm: bool,
    options: GenerationOptions,
) -> Result<()> {
    info!("Generating summaries for: {:?}", project_root);
    info!("Output tier: {:?}, Token budget: {:?}", options.tier, options.token_budget);

    // Create output directory
    let output_path = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        project_root.join(output_dir)
    };
    fs::create_dir_all(&output_path)
        .with_context(|| format!("Failed to create output directory: {:?}", output_path))?;

    // Create analyzer
    let analyzer = ProjectAnalyzer::new(project_root)?;

    // Create template renderer
    let mut renderer = TemplateRenderer::new()?;
    if let Some(template_path) = custom_template {
        renderer.load_custom_template("custom", &template_path)?;
    }

    // Optionally create LLM client
    let _llm_client: Option<Box<dyn llm::LlmClient>> = if !no_llm {
        if let Some(provider) = llm_provider {
            match llm::create_client(&provider, llm_endpoint.as_deref(), &llm_model) {
                Ok(client) => {
                    if client.is_available() {
                        info!("Using LLM: {} ({})", provider, llm_model);
                        Some(client)
                    } else {
                        warn!("LLM not available, using structural analysis only");
                        None
                    }
                }
                Err(e) => {
                    warn!("Failed to create LLM client: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Generate architecture summary
    println!("Analyzing project structure...");
    let structure = analyzer.analyze_structure()?;
    let stats = analyzer.calculate_stats()?;

    println!("Generating ARCHITECTURE.md...");
    let arch_content = renderer.render_architecture(&structure, &stats)?;
    let arch_path = output_path.join("ARCHITECTURE.md");
    fs::write(&arch_path, &arch_content)?;
    let token_estimate = estimate_tokens(&arch_content);
    println!("  Generated {} ({} tokens estimated)", arch_path.display(), token_estimate);

    // Generate dependency graph
    println!("Building dependency graph...");
    let dep_graph = analyzer.build_dependency_graph()?;
    let deps_content = renderer.render_dependencies(&dep_graph)?;
    let deps_path = output_path.join("DEPENDENCY_GRAPH.md");
    fs::write(&deps_path, &deps_content)?;
    let token_estimate = estimate_tokens(&deps_content);
    println!("  Generated {} ({} tokens estimated)", deps_path.display(), token_estimate);

    // Generate hotspots
    println!("Identifying hotspots...");
    let hotspots = analyzer.identify_hotspots()?;
    let hotspots_content = renderer.render_hotspots(&hotspots, 10)?;
    let hotspots_path = output_path.join("HOTSPOTS.md");
    fs::write(&hotspots_path, &hotspots_content)?;
    let token_estimate = estimate_tokens(&hotspots_content);
    println!("  Generated {} ({} tokens estimated)", hotspots_path.display(), token_estimate);

    // Generate API reference (only for standard and full tiers)
    if options.tier == OutputTier::Standard || options.tier == OutputTier::Full {
        println!("Generating API reference...");
        let api_content = renderer.render_api(&structure.modules)?;
        let api_path = output_path.join("API.md");
        fs::write(&api_path, &api_content)?;
        let token_estimate = estimate_tokens(&api_content);
        println!("  Generated {} ({} tokens estimated)", api_path.display(), token_estimate);
    }

    // Generate type definitions (only for full tier)
    if options.tier == OutputTier::Full {
        println!("Generating type definitions...");
        let types_content = renderer.render_types(&structure.modules)?;
        let types_path = output_path.join("TYPES.md");
        fs::write(&types_path, &types_content)?;
        let token_estimate = estimate_tokens(&types_content);
        println!("  Generated {} ({} tokens estimated)", types_path.display(), token_estimate);
    }

    // Generate module summaries (only for standard and full tiers)
    if options.tier == OutputTier::Standard || options.tier == OutputTier::Full {
        println!("Generating module summaries...");
        let module_dir = output_path.join("MODULE_MAPS");
        fs::create_dir_all(&module_dir)?;

        let mut module_count = 0;
        for module in &structure.modules {
            if module.name == "(root)" {
                continue;
            }

            let module_hotspots: Vec<_> = hotspots
                .iter()
                .filter(|h| {
                    h.relative_path
                        .to_string_lossy()
                        .starts_with(&module.name)
                })
                .cloned()
                .collect();

            let module_content = renderer.render_module(module, &module_hotspots)?;
            let module_path = module_dir.join(format!("{}.md", module.name));
            fs::write(&module_path, &module_content)?;
            module_count += 1;
        }
        println!("  Generated {} module summaries", module_count);
    }

    println!("\nSummary generation complete!");
    println!("AI agents can now use {:?} for efficient context.", output_path);

    Ok(())
}

/// Update summaries if stale
pub fn update_summaries(
    project_root: &Path,
    output_dir: &Path,
    if_stale: bool,
    force: bool,
    summary_type: Option<String>,
) -> Result<()> {
    let output_path = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        project_root.join(output_dir)
    };

    // Check staleness if requested
    if if_stale && !force {
        if let Ok(git) = GitClient::open(project_root) {
            let arch_path = output_path.join("ARCHITECTURE.md");
            if arch_path.exists() {
                let staleness = git.is_summary_stale(&arch_path)?;
                if !staleness.is_stale {
                    println!("Summaries are up to date (no commits since last generation)");
                    return Ok(());
                }
                println!(
                    "Summaries are stale: {} commits since last generation",
                    staleness.commits_since
                );
            }
        }
    }

    // Determine what to regenerate
    match summary_type.as_deref() {
        Some("architecture") => {
            println!("Regenerating architecture summary...");
            regenerate_architecture(project_root, &output_path)?;
        }
        Some("modules") => {
            println!("Regenerating module summaries...");
            regenerate_modules(project_root, &output_path)?;
        }
        Some("dependencies") => {
            println!("Regenerating dependency graph...");
            regenerate_dependencies(project_root, &output_path)?;
        }
        Some("hotspots") => {
            println!("Regenerating hotspots...");
            regenerate_hotspots(project_root, &output_path)?;
        }
        _ => {
            // Regenerate all
            generate_all(project_root, output_dir, None, None, String::new(), None, true, GenerationOptions::default())?;
        }
    }

    Ok(())
}

/// Generate a summary for a specific module
pub fn generate_module(
    project_root: &Path,
    output_dir: &Path,
    module_path: &Path,
    depth: &str,
) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let renderer = TemplateRenderer::new()?;

    let full_module_path = if module_path.is_absolute() {
        module_path.to_path_buf()
    } else {
        project_root.join(module_path)
    };

    let module = analyzer.analyze_module(&full_module_path)?;
    let hotspots = analyzer.identify_hotspots()?;

    let module_hotspots: Vec<_> = hotspots
        .iter()
        .filter(|h| h.file_path.starts_with(&full_module_path))
        .cloned()
        .collect();

    let content = renderer.render_module(&module, &module_hotspots)?;

    let output_path = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        project_root.join(output_dir)
    };

    fs::create_dir_all(&output_path)?;

    let output_file = output_path.join(format!("{}.md", module.name));
    fs::write(&output_file, &content)?;

    println!("Generated module summary: {:?}", output_file);
    println!("Depth: {} (note: depth levels not yet implemented)", depth);

    Ok(())
}

/// Estimate token count for a string (rough approximation)
fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 characters per token for English text
    // This is a simplified estimate; actual tokenization varies by model
    text.len() / 4
}

/// Regenerate just the architecture summary
fn regenerate_architecture(project_root: &Path, output_path: &Path) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let renderer = TemplateRenderer::new()?;

    let structure = analyzer.analyze_structure()?;
    let stats = analyzer.calculate_stats()?;

    let content = renderer.render_architecture(&structure, &stats)?;
    fs::write(output_path.join("ARCHITECTURE.md"), &content)?;

    Ok(())
}

/// Regenerate just the module summaries
fn regenerate_modules(project_root: &Path, output_path: &Path) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let renderer = TemplateRenderer::new()?;

    let structure = analyzer.analyze_structure()?;
    let hotspots = analyzer.identify_hotspots()?;

    let module_dir = output_path.join("MODULE_MAPS");
    fs::create_dir_all(&module_dir)?;

    for module in &structure.modules {
        if module.name == "(root)" {
            continue;
        }

        let module_hotspots: Vec<_> = hotspots
            .iter()
            .filter(|h| h.relative_path.to_string_lossy().starts_with(&module.name))
            .cloned()
            .collect();

        let content = renderer.render_module(module, &module_hotspots)?;
        fs::write(module_dir.join(format!("{}.md", module.name)), &content)?;
    }

    Ok(())
}

/// Regenerate just the dependency graph
fn regenerate_dependencies(project_root: &Path, output_path: &Path) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let renderer = TemplateRenderer::new()?;

    let graph = analyzer.build_dependency_graph()?;
    let content = renderer.render_dependencies(&graph)?;
    fs::write(output_path.join("DEPENDENCY_GRAPH.md"), &content)?;

    Ok(())
}

/// Regenerate just the hotspots
fn regenerate_hotspots(project_root: &Path, output_path: &Path) -> Result<()> {
    let analyzer = ProjectAnalyzer::new(project_root)?;
    let renderer = TemplateRenderer::new()?;

    let hotspots = analyzer.identify_hotspots()?;
    let content = renderer.render_hotspots(&hotspots, 10)?;
    fs::write(output_path.join("HOTSPOTS.md"), &content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();

        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/test.rs"), "#[test]\nfn it_works() {}\n").unwrap();

        fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

        temp
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);

        // Longer text
        let text = "This is a longer piece of text for testing token estimation.";
        let estimate = estimate_tokens(text);
        assert!(estimate > 10);
        assert!(estimate < 30);
    }

    #[test]
    fn test_generate_all() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        let result = generate_all(
            temp.path(),
            &output_dir,
            None,
            None,
            String::new(),
            None,
            true,
            GenerationOptions::default(),
        );

        assert!(result.is_ok(), "generate_all failed: {:?}", result.err());

        // Verify output files exist
        assert!(output_dir.join("ARCHITECTURE.md").exists());
        assert!(output_dir.join("DEPENDENCY_GRAPH.md").exists());
        assert!(output_dir.join("HOTSPOTS.md").exists());
        assert!(output_dir.join("MODULE_MAPS").exists());
    }

    #[test]
    fn test_generate_all_creates_module_maps() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        generate_all(
            temp.path(),
            &output_dir,
            None,
            None,
            String::new(),
            None,
            true,
            GenerationOptions::default(),
        )
        .unwrap();

        let module_maps = output_dir.join("MODULE_MAPS");
        assert!(module_maps.exists());

        // Should have src.md and tests.md
        assert!(module_maps.join("src.md").exists());
        assert!(module_maps.join("tests.md").exists());
    }

    #[test]
    fn test_generate_module() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        let result = generate_module(
            temp.path(),
            &output_dir,
            Path::new("src"),
            "standard",
        );

        assert!(result.is_ok());
        assert!(output_dir.join("src.md").exists());
    }

    #[test]
    fn test_architecture_content_has_expected_sections() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        generate_all(
            temp.path(),
            &output_dir,
            None,
            None,
            String::new(),
            None,
            true,
            GenerationOptions::default(),
        )
        .unwrap();

        let content = fs::read_to_string(output_dir.join("ARCHITECTURE.md")).unwrap();

        assert!(content.contains("Architecture Overview"));
        assert!(content.contains("Generated:"));
        assert!(content.contains("Files Indexed:"));
    }

    #[test]
    fn test_hotspots_content_has_formula() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        generate_all(
            temp.path(),
            &output_dir,
            None,
            None,
            String::new(),
            None,
            true,
            GenerationOptions::default(),
        )
        .unwrap();

        let content = fs::read_to_string(output_dir.join("HOTSPOTS.md")).unwrap();

        assert!(content.contains("Hotness Formula"));
        assert!(content.contains("change_count"));
    }

    #[test]
    fn test_minimal_tier_generates_less_output() {
        let temp = create_test_project();
        let output_dir = temp.path().join(".ai/context");

        let options = GenerationOptions {
            tier: OutputTier::Minimal,
            token_budget: None,
            semantic_extraction: true,
        };

        generate_all(
            temp.path(),
            &output_dir,
            None,
            None,
            String::new(),
            None,
            true,
            options,
        )
        .unwrap();

        // Minimal tier should not create MODULE_MAPS or API.md
        assert!(output_dir.join("ARCHITECTURE.md").exists());
        assert!(!output_dir.join("API.md").exists());
        assert!(!output_dir.join("MODULE_MAPS").exists());
    }
}
