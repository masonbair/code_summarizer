use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

use super::builtin;
use crate::analyzer::{DependencyGraph, Hotspot, Module, ProjectStats, ProjectStructure};

/// Template renderer using Tera
pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    /// Create a new template renderer with built-in templates
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Load built-in templates
        tera.add_raw_template("architecture", builtin::ARCHITECTURE_TEMPLATE)
            .context("Failed to load architecture template")?;
        tera.add_raw_template("module", builtin::MODULE_TEMPLATE)
            .context("Failed to load module template")?;
        tera.add_raw_template("dependencies", builtin::DEPENDENCIES_TEMPLATE)
            .context("Failed to load dependencies template")?;
        tera.add_raw_template("hotspots", builtin::HOTSPOTS_TEMPLATE)
            .context("Failed to load hotspots template")?;
        tera.add_raw_template("api", builtin::API_TEMPLATE)
            .context("Failed to load api template")?;
        tera.add_raw_template("types", builtin::TYPES_TEMPLATE)
            .context("Failed to load types template")?;

        Ok(Self { tera })
    }

    /// Load a custom template from a file
    pub fn load_custom_template(&mut self, name: &str, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read template: {:?}", path))?;

        self.tera
            .add_raw_template(name, &content)
            .with_context(|| format!("Failed to parse template: {:?}", path))?;

        Ok(())
    }

    /// Render the architecture summary
    pub fn render_architecture(
        &self,
        structure: &ProjectStructure,
        stats: &ProjectStats,
    ) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("project", structure);
        context.insert("stats", stats);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("architecture", &context)
            .context("Failed to render architecture template")
    }

    /// Render a module summary
    pub fn render_module(&self, module: &Module, hotspots: &[Hotspot]) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("module", module);
        context.insert("hotspots", hotspots);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("module", &context)
            .context("Failed to render module template")
    }

    /// Render the dependency graph
    pub fn render_dependencies(&self, graph: &DependencyGraph) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("graph", graph);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("dependencies", &context)
            .context("Failed to render dependencies template")
    }

    /// Render the hotspots summary
    pub fn render_hotspots(&self, hotspots: &[Hotspot], limit: usize) -> Result<String> {
        let mut context = TeraContext::new();
        let limited: Vec<_> = hotspots.iter().take(limit).collect();
        context.insert("hotspots", &limited);
        context.insert("total_count", &hotspots.len());
        context.insert("limit", &limit);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("hotspots", &context)
            .context("Failed to render hotspots template")
    }

    /// Render the public API reference
    pub fn render_api(&self, modules: &[Module]) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("modules", modules);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("api", &context)
            .context("Failed to render api template")
    }

    /// Render the type definitions reference
    pub fn render_types(&self, modules: &[Module]) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("modules", modules);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render("types", &context)
            .context("Failed to render types template")
    }

    /// Render a custom template with arbitrary data
    pub fn render_custom<T: Serialize>(&self, template_name: &str, data: &T) -> Result<String> {
        let mut context = TeraContext::new();
        context.insert("data", data);
        context.insert("generated_at", &chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string());

        self.tera
            .render(template_name, &context)
            .with_context(|| format!("Failed to render template: {}", template_name))
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateRenderer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_structure() -> ProjectStructure {
        ProjectStructure {
            root: PathBuf::from("/test/project"),
            modules: vec![
                Module {
                    name: "src".to_string(),
                    path: PathBuf::from("/test/project/src"),
                    files: vec![],
                    subdirs: vec!["utils".to_string()],
                    description: Some("Source code".to_string()),
                    purpose: None,
                    public_apis: vec![],
                    types: vec![],
                    traits: vec![],
                },
            ],
            total_files: 10,
            total_lines: 500,
            language_breakdown: {
                let mut map = HashMap::new();
                map.insert("Rust".to_string(), 8);
                map.insert("TOML".to_string(), 2);
                map
            },
            entry_points: vec![],
            all_traits: vec![],
            all_trait_impls: vec![],
        }
    }

    fn create_test_stats() -> ProjectStats {
        ProjectStats {
            total_files: 10,
            total_lines: 500,
            total_size_bytes: 15000,
            language_breakdown: {
                let mut map = HashMap::new();
                map.insert("Rust".to_string(), 8);
                map.insert("TOML".to_string(), 2);
                map
            },
            module_count: 2,
            avg_file_size_lines: 50.0,
            largest_file: None,
            smallest_file: None,
        }
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = TemplateRenderer::new();
        assert!(renderer.is_ok());
    }

    #[test]
    fn test_render_architecture() {
        let renderer = TemplateRenderer::new().unwrap();
        let structure = create_test_structure();
        let stats = create_test_stats();

        let result = renderer.render_architecture(&structure, &stats);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Architecture"));
        assert!(output.contains("10")); // total files
    }

    #[test]
    fn test_render_module() {
        let renderer = TemplateRenderer::new().unwrap();
        let module = Module {
            name: "src".to_string(),
            path: PathBuf::from("/test/src"),
            files: vec![],
            subdirs: vec![],
            description: Some("Source module".to_string()),
            purpose: None,
            public_apis: vec![],
            types: vec![],
            traits: vec![],
        };

        let result = renderer.render_module(&module, &[]);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("src"));
    }

    #[test]
    fn test_render_dependencies() {
        let renderer = TemplateRenderer::new().unwrap();
        let graph = DependencyGraph::new();

        let result = renderer.render_dependencies(&graph);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("Dependenc")); // Dependencies or Dependency
    }

    #[test]
    fn test_render_hotspots() {
        let renderer = TemplateRenderer::new().unwrap();
        let hotspots = vec![
            Hotspot {
                file_path: PathBuf::from("src/main.rs"),
                relative_path: PathBuf::from("src/main.rs"),
                lines_of_code: 100,
                change_count: 10,
                dependency_count: 5,
                complexity_score: 7.5,
                hotness_score: 32.5,
                enhanced_metrics: None,
            },
        ];

        let result = renderer.render_hotspots(&hotspots, 10);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("main.rs"));
        assert!(output.contains("100")); // lines
    }

    #[test]
    fn test_load_custom_template() {
        let temp = TempDir::new().unwrap();
        let template_path = temp.path().join("custom.md.tera");
        std::fs::write(&template_path, "# Custom: {{ data.name }}").unwrap();

        let mut renderer = TemplateRenderer::new().unwrap();
        let result = renderer.load_custom_template("custom", &template_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_custom_template_not_found() {
        let mut renderer = TemplateRenderer::new().unwrap();
        let result = renderer.load_custom_template("custom", Path::new("/nonexistent.tera"));
        assert!(result.is_err());
    }

    #[test]
    fn test_render_custom() {
        let temp = TempDir::new().unwrap();
        let template_path = temp.path().join("custom.md.tera");
        std::fs::write(&template_path, "Name: {{ data.name }}").unwrap();

        let mut renderer = TemplateRenderer::new().unwrap();
        renderer.load_custom_template("custom", &template_path).unwrap();

        #[derive(Serialize)]
        struct Data {
            name: String,
        }

        let data = Data {
            name: "Test".to_string(),
        };

        let result = renderer.render_custom("custom", &data);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Test"));
    }
}
