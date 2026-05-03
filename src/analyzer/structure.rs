use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::code_index::CodeIndexClient;
use crate::git::GitClient;
use super::semantic::{SemanticAnalyzer, FileSemantics, PublicApi, TraitDef, TypeDef, TraitImplInfo, EntryPoint};

/// Represents the analyzed structure of a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    pub root: PathBuf,
    pub modules: Vec<Module>,
    pub total_files: usize,
    pub total_lines: usize,
    pub language_breakdown: HashMap<String, usize>,
    pub entry_points: Vec<EntryPoint>,
    pub all_traits: Vec<TraitDef>,
    pub all_trait_impls: Vec<TraitImplInfo>,
}

/// Represents a module (directory) in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<FileInfo>,
    pub subdirs: Vec<String>,
    pub description: Option<String>,
    pub purpose: Option<ModulePurpose>,
    pub public_apis: Vec<PublicApi>,
    pub types: Vec<TypeDef>,
    pub traits: Vec<TraitDef>,
}

/// Detected purpose of a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePurpose {
    pub summary: String,
    pub patterns: Vec<String>,
    pub key_components: Vec<String>,
}

/// Information about a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub lines: usize,
    pub language: String,
    pub size_bytes: u64,
    pub semantics: Option<FileSemantics>,
}

/// Main analyzer for project structure
pub struct ProjectAnalyzer {
    project_root: PathBuf,
    index_client: Option<CodeIndexClient>,
    git_client: Option<GitClient>,
    semantic_analyzer: Option<SemanticAnalyzer>,
}

impl ProjectAnalyzer {
    /// Create a new analyzer for the given project root
    pub fn new(project_root: &Path) -> Result<Self> {
        let project_root = project_root
            .canonicalize()
            .with_context(|| format!("Failed to resolve project root: {:?}", project_root))?;

        // Try to connect to code-index (optional)
        let index_client = CodeIndexClient::new(&project_root).ok();
        if index_client.is_none() {
            log::warn!("code-index not available, using basic analysis");
        }

        // Try to open git repo (optional)
        let git_client = GitClient::open(&project_root).ok();
        if git_client.is_none() {
            log::warn!("Git repository not found, change tracking disabled");
        }

        // Create semantic analyzer
        let semantic_analyzer = SemanticAnalyzer::new().ok();
        if semantic_analyzer.is_none() {
            log::warn!("Semantic analyzer not available");
        }

        Ok(Self {
            project_root,
            index_client,
            git_client,
            semantic_analyzer,
        })
    }

    /// Analyze the project structure
    pub fn analyze_structure(&self) -> Result<ProjectStructure> {
        let mut modules = Vec::new();
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut language_breakdown: HashMap<String, usize> = HashMap::new();
        let mut all_entry_points = Vec::new();
        let mut all_traits = Vec::new();
        let mut all_trait_impls = Vec::new();

        // Find top-level directories as modules
        let entries: Vec<_> = std::fs::read_dir(&self.project_root)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| !self.should_ignore(&e.path()))
            .collect();

        for entry in entries {
            let module_path = entry.path();
            let module = self.analyze_module(&module_path)?;

            total_files += module.files.len();
            total_lines += module.files.iter().map(|f| f.lines).sum::<usize>();

            for file in &module.files {
                *language_breakdown.entry(file.language.clone()).or_insert(0) += 1;

                // Collect entry points, traits, and impls from file semantics
                if let Some(ref sem) = file.semantics {
                    all_entry_points.extend(sem.entry_points.clone());
                    all_traits.extend(sem.traits.clone());
                    all_trait_impls.extend(sem.trait_impls.clone());
                }
            }

            modules.push(module);
        }

        // Also analyze files in root directory
        let root_files: Vec<FileInfo> = std::fs::read_dir(&self.project_root)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| !self.should_ignore(&e.path()))
            .filter_map(|e| self.analyze_file(&e.path()).ok())
            .collect();

        total_files += root_files.len();
        total_lines += root_files.iter().map(|f| f.lines).sum::<usize>();

        for file in &root_files {
            *language_breakdown.entry(file.language.clone()).or_insert(0) += 1;

            // Collect entry points, traits, and impls
            if let Some(ref sem) = file.semantics {
                all_entry_points.extend(sem.entry_points.clone());
                all_traits.extend(sem.traits.clone());
                all_trait_impls.extend(sem.trait_impls.clone());
            }
        }

        if !root_files.is_empty() {
            let mut root_public_apis = Vec::new();
            let mut root_types = Vec::new();
            let mut root_traits = Vec::new();

            for file in &root_files {
                if let Some(ref sem) = file.semantics {
                    root_public_apis.extend(sem.public_apis.clone());
                    root_types.extend(sem.types.clone());
                    root_traits.extend(sem.traits.clone());
                }
            }

            modules.insert(
                0,
                Module {
                    name: "(root)".to_string(),
                    path: self.project_root.clone(),
                    files: root_files,
                    subdirs: vec![],
                    description: Some("Root directory files".to_string()),
                    purpose: None,
                    public_apis: root_public_apis,
                    types: root_types,
                    traits: root_traits,
                },
            );
        }

        Ok(ProjectStructure {
            root: self.project_root.clone(),
            modules,
            total_files,
            total_lines,
            language_breakdown,
            entry_points: all_entry_points,
            all_traits,
            all_trait_impls,
        })
    }

    /// Analyze a specific module (directory)
    pub fn analyze_module(&self, module_path: &Path) -> Result<Module> {
        let name = module_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut files = Vec::new();
        let mut subdirs = Vec::new();
        let mut module_public_apis = Vec::new();
        let mut module_types = Vec::new();
        let mut module_traits = Vec::new();

        for entry in WalkDir::new(module_path)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| !self.should_ignore(e.path()))
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(file_info) = self.analyze_file(path) {
                    // Collect module-level semantic info
                    if let Some(ref sem) = file_info.semantics {
                        module_public_apis.extend(sem.public_apis.clone());
                        module_types.extend(sem.types.clone());
                        module_traits.extend(sem.traits.clone());
                    }
                    files.push(file_info);
                }
            } else if path.is_dir() && path != module_path {
                if let Some(dir_name) = path.file_name() {
                    let relative = path.strip_prefix(module_path).unwrap_or(path);
                    if relative.components().count() == 1 {
                        subdirs.push(dir_name.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Infer module purpose
        let purpose = self.infer_module_purpose(&name, &files, &module_public_apis, &module_traits);

        Ok(Module {
            name,
            path: module_path.to_path_buf(),
            files,
            subdirs,
            description: purpose.as_ref().map(|p| p.summary.clone()),
            purpose,
            public_apis: module_public_apis,
            types: module_types,
            traits: module_traits,
        })
    }

    /// Infer the purpose of a module based on its contents
    fn infer_module_purpose(
        &self,
        name: &str,
        files: &[FileInfo],
        public_apis: &[PublicApi],
        traits: &[TraitDef],
    ) -> Option<ModulePurpose> {
        let mut patterns = Vec::new();
        let mut key_components = Vec::new();

        // Check for common module patterns based on name
        let summary = match name {
            "cli" | "args" | "commands" => {
                patterns.push("Command-line interface".to_string());
                "Handles CLI argument parsing and command execution".to_string()
            }
            "config" | "settings" | "configuration" => {
                patterns.push("Configuration management".to_string());
                "Manages application configuration and settings".to_string()
            }
            "api" | "routes" | "handlers" | "endpoints" => {
                patterns.push("API layer".to_string());
                "Defines API endpoints and request handlers".to_string()
            }
            "db" | "database" | "models" | "schema" => {
                patterns.push("Data layer".to_string());
                "Database models and data access".to_string()
            }
            "auth" | "authentication" | "security" => {
                patterns.push("Security layer".to_string());
                "Handles authentication and security".to_string()
            }
            "utils" | "helpers" | "common" | "shared" => {
                patterns.push("Utilities".to_string());
                "Shared utility functions and helpers".to_string()
            }
            "tests" | "test" => {
                patterns.push("Test suite".to_string());
                "Contains test files and test utilities".to_string()
            }
            "templates" => {
                patterns.push("Template system".to_string());
                "Template files for code generation".to_string()
            }
            "error" | "errors" => {
                patterns.push("Error handling".to_string());
                "Error types and error handling utilities".to_string()
            }
            "types" => {
                patterns.push("Type definitions".to_string());
                "Core type definitions and data structures".to_string()
            }
            _ => {
                // Analyze based on content
                self.analyze_module_content_purpose(files, public_apis, traits, &mut patterns, &mut key_components)
            }
        };

        // Extract key components from public APIs
        for api in public_apis.iter().take(5) {
            key_components.push(api.name.clone());
        }

        // Check for trait definitions -> abstraction layer
        if !traits.is_empty() {
            patterns.push("Defines abstractions".to_string());
            for trait_def in traits.iter().take(3) {
                key_components.push(format!("{} (trait)", trait_def.name));
            }
        }

        Some(ModulePurpose {
            summary,
            patterns,
            key_components,
        })
    }

    fn analyze_module_content_purpose(
        &self,
        files: &[FileInfo],
        public_apis: &[PublicApi],
        traits: &[TraitDef],
        patterns: &mut Vec<String>,
        key_components: &mut Vec<String>,
    ) -> String {
        // Check for patterns based on file names and content
        let file_names: Vec<&str> = files.iter()
            .filter_map(|f| f.path.file_stem())
            .filter_map(|s| s.to_str())
            .collect();

        // Check for mod.rs or __init__.py (module entry)
        let has_mod_entry = files.iter().any(|f| {
            f.path.file_name().map(|n| n == "mod.rs" || n == "__init__.py").unwrap_or(false)
        });

        // Check for strategy pattern (trait + multiple implementations)
        if !traits.is_empty() {
            let trait_count = traits.len();
            patterns.push(format!("Defines {} trait(s)", trait_count));
        }

        // Check for coordinator/dispatcher pattern
        let has_coordinator = public_apis.iter().any(|api| {
            api.name.to_lowercase().contains("coordinator") ||
            api.name.to_lowercase().contains("dispatcher") ||
            api.name.to_lowercase().contains("manager")
        });

        if has_coordinator {
            patterns.push("Coordinator/Manager pattern".to_string());
        }

        // Check for builder pattern
        let has_builder = public_apis.iter().any(|api| {
            api.name.to_lowercase().contains("builder")
        });

        if has_builder {
            patterns.push("Builder pattern".to_string());
        }

        // Generate summary based on analysis
        if !traits.is_empty() && has_mod_entry {
            format!("Module providing {} type(s) with {} public API(s)",
                traits.len(), public_apis.len())
        } else if !public_apis.is_empty() {
            format!("Module with {} public function(s)/type(s)", public_apis.len())
        } else {
            format!("Module containing {} file(s)", files.len())
        }
    }

    /// Analyze a single file
    fn analyze_file(&self, path: &Path) -> Result<FileInfo> {
        let metadata = std::fs::metadata(path)?;
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let lines = content.lines().count();
        let language = detect_language(path);

        let relative_path = path
            .strip_prefix(&self.project_root)
            .unwrap_or(path)
            .to_path_buf();

        // Perform semantic analysis if available
        let semantics = self.analyze_file_semantics(path);

        Ok(FileInfo {
            path: path.to_path_buf(),
            relative_path,
            lines,
            language,
            size_bytes: metadata.len(),
            semantics,
        })
    }

    /// Perform semantic analysis on a file
    fn analyze_file_semantics(&self, path: &Path) -> Option<FileSemantics> {
        // Create a new semantic analyzer for thread safety
        // (the analyzer mutates internal parser state)
        let mut analyzer = SemanticAnalyzer::new().ok()?;
        analyzer.analyze_file(path).ok()
    }

    /// Check if a path should be ignored
    fn should_ignore(&self, path: &Path) -> bool {
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

        // Ignore common non-source directories (check by name, not existence)
        let ignored_dirs = [
            ".git",
            "node_modules",
            "target",
            "dist",
            "build",
            "__pycache__",
            ".venv",
            "venv",
            ".idea",
            ".vscode",
            ".ai",
            "vendor",
        ];

        // Check if name matches ignored directories
        if ignored_dirs.iter().any(|d| name == *d) {
            return true;
        }

        // For files only: check extensions and hidden status
        if path.is_file() || !path.exists() {
            // Ignore common non-source files
            let ignored_extensions = ["lock", "sum", "map"];
            if let Some(ext) = path.extension() {
                if ignored_extensions.iter().any(|e| ext == *e) {
                    return true;
                }
            }

            // Ignore hidden files (but not directories, as .git is already handled)
            if !path.is_dir() && name.starts_with('.') {
                return true;
            }
        }

        false
    }

    /// Get the code-index client if available
    pub fn index_client(&self) -> Option<&CodeIndexClient> {
        self.index_client.as_ref()
    }

    /// Get the git client if available
    pub fn git_client(&self) -> Option<&GitClient> {
        self.git_client.as_ref()
    }

    /// Get the project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
}

/// Detect programming language from file extension
pub fn detect_language(path: &Path) -> String {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "tsx" => "TypeScript (React)",
        "jsx" => "JavaScript (React)",
        "go" => "Go",
        "java" => "Java",
        "c" => "C",
        "cpp" | "cc" | "cxx" => "C++",
        "h" | "hpp" => "C/C++ Header",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "kt" | "kts" => "Kotlin",
        "scala" => "Scala",
        "cs" => "C#",
        "fs" | "fsx" => "F#",
        "hs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "ex" | "exs" => "Elixir",
        "erl" => "Erlang",
        "clj" | "cljs" => "Clojure",
        "lua" => "Lua",
        "sh" | "bash" => "Shell",
        "zsh" => "Zsh",
        "ps1" => "PowerShell",
        "sql" => "SQL",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" | "sass" => "SCSS",
        "less" => "Less",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "md" | "markdown" => "Markdown",
        "txt" => "Text",
        _ => "Other",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        // Create src directory with Rust files
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();

        // Create tests directory
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/integration.rs"), "#[test]\nfn test_something() {}\n").unwrap();

        // Create root files
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(root.join("README.md"), "# Test Project\n").unwrap();

        // Create .git to simulate git repo (should be ignored)
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join(".git/config"), "").unwrap();

        temp
    }

    #[test]
    fn test_detect_language_rust() {
        assert_eq!(detect_language(Path::new("src/main.rs")), "Rust");
        assert_eq!(detect_language(Path::new("lib.rs")), "Rust");
    }

    #[test]
    fn test_detect_language_javascript() {
        assert_eq!(detect_language(Path::new("app.js")), "JavaScript");
        assert_eq!(detect_language(Path::new("component.jsx")), "JavaScript (React)");
    }

    #[test]
    fn test_detect_language_typescript() {
        assert_eq!(detect_language(Path::new("app.ts")), "TypeScript");
        assert_eq!(detect_language(Path::new("component.tsx")), "TypeScript (React)");
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(detect_language(Path::new("main.py")), "Python");
    }

    #[test]
    fn test_detect_language_unknown() {
        assert_eq!(detect_language(Path::new("file.xyz")), "Other");
        assert_eq!(detect_language(Path::new("no_extension")), "Other");
    }

    #[test]
    fn test_analyze_structure() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let structure = analyzer.analyze_structure().unwrap();

        assert_eq!(structure.root, temp.path().canonicalize().unwrap());
        assert!(structure.total_files > 0);
        assert!(structure.total_lines > 0);
        assert!(structure.language_breakdown.contains_key("Rust"));
    }

    #[test]
    fn test_analyze_module() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let module = analyzer.analyze_module(&temp.path().join("src")).unwrap();

        assert_eq!(module.name, "src");
        assert_eq!(module.files.len(), 2); // main.rs and lib.rs
        assert!(module.files.iter().all(|f| f.language == "Rust"));
    }

    #[test]
    fn test_should_ignore_git_directory() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();

        assert!(analyzer.should_ignore(&temp.path().join(".git")));
        assert!(analyzer.should_ignore(&temp.path().join("node_modules")));
        assert!(analyzer.should_ignore(&temp.path().join("target")));
    }

    #[test]
    fn test_should_not_ignore_source_directories() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();

        assert!(!analyzer.should_ignore(&temp.path().join("src")));
        assert!(!analyzer.should_ignore(&temp.path().join("tests")));
    }

    #[test]
    fn test_file_info_contains_correct_data() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let file_info = analyzer.analyze_file(&temp.path().join("src/main.rs")).unwrap();

        assert_eq!(file_info.language, "Rust");
        assert_eq!(file_info.lines, 3);
        assert!(file_info.size_bytes > 0);
        assert_eq!(file_info.relative_path, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_language_breakdown_counts_correctly() {
        let temp = create_test_project();
        let analyzer = ProjectAnalyzer::new(temp.path()).unwrap();
        let structure = analyzer.analyze_structure().unwrap();

        // Should have 3 Rust files: main.rs, lib.rs, integration.rs
        assert_eq!(structure.language_breakdown.get("Rust"), Some(&3));
        // Should have 1 TOML file: Cargo.toml
        assert_eq!(structure.language_breakdown.get("TOML"), Some(&1));
        // Should have 1 Markdown file: README.md
        assert_eq!(structure.language_breakdown.get("Markdown"), Some(&1));
    }
}
