# code-summarizer - Context Map Generator

## Project Overview

Generates hierarchical, AI-optimized markdown summaries of codebase architecture. Reduces token usage by 80% vs. reading raw code by providing high-level context maps.

**Primary Languages:** Rust
**Project Type:** CLI tool
**Created:** 2026-04-28
**Target Platform:** Arch Linux (portable to other Linux distros)

---

## Purpose & Problem Statement

### Core Problem Solved
AI agents waste significant context tokens reading entire files to understand structure. A pre-generated architecture map provides high-level context in ~200-500 tokens instead of thousands of tokens for raw code.

### Key Value Propositions
- **Multi-level summaries**: Architecture → Module → File → Function hierarchy
- **Dependency visualization**: Import graphs, call hierarchies
- **Hotspot detection**: Identifies complex/frequently-changed areas
- **Auto-regeneration**: Detects stale summaries via git commits
- **Template-based**: Customizable output format

---

## Required Features

### 1. Architecture Summary Generation
- Create high-level system overview (~200-500 tokens)
- List major components and their purposes
- Describe data flow and critical dependencies
- Identify entry points and main routes
- Calculate project statistics (file counts, language breakdown)

### 2. Module Breakdown
- Generate per-module summaries (frontend, backend, shared, etc.)
- List key files and their roles
- Document public APIs and interfaces
- Track "hot files" (frequently changed, high complexity)

### 3. Dependency Graph Visualization
- Create ASCII/markdown dependency diagrams
- Show import relationships between modules
- Identify circular dependencies
- Map external dependencies

### 4. Hotspot Detection
- Calculate complexity scores (lines of code, cyclomatic complexity)
- Track change frequency via git history
- Combine metrics into "hotness" score
- Highlight risky or complex areas needing attention

### 5. Staleness Detection
- Compare summary timestamp with latest git commits
- Auto-regenerate when code changes significantly
- Support manual regeneration
- Incremental updates where possible

### 6. Optional LLM Integration
- Support local LLM (Ollama/Llama) for natural language descriptions
- Generate human-readable component summaries
- Fall back to structural analysis if LLM unavailable

---

## Architecture

### High-Level Flow
```
Input: Codebase
  │
  ├─> CodeIndex query (symbols, deps, metadata)
  │
  ├─> File structure analysis (walkdir)
  │
  ├─> Git history analysis (change frequency)
  │
  ├─> Optional: LLM summarization (local Llama)
  │
  ├─> Template rendering (Tera)
  │
  ▼
Output: .ai/context/*.md files
```

### Component Breakdown

**Analyzer Module:**
- Queries code-index for symbols and dependencies
- Analyzes project structure
- Calculates complexity and hotness metrics
- Builds dependency graphs

**Template Renderer:**
- Uses Tera for markdown generation
- Customizable templates
- Consistent formatting
- Token counting for optimization

**Git Integration:**
- Analyzes commit history
- Tracks file change frequency
- Detects when re-generation needed
- Blames for ownership tracking

**LLM Client (Optional):**
- Connects to local Ollama instance
- Generates natural language summaries
- Gracefully degrades if unavailable

---

## Output Structure

```
.ai/context/
├── ARCHITECTURE.md           # High-level system design (200-500 tokens)
├── MODULE_MAPS/
│   ├── frontend.md           # Frontend module breakdown
│   ├── backend.md            # Backend services
│   └── shared.md             # Shared utilities
├── DEPENDENCY_GRAPH.md       # Visual dependency graph
└── HOTSPOTS.md               # Complex/risky areas
```

### Sample Output: ARCHITECTURE.md

```markdown
# MyApp - Architecture Overview

**Generated:** 2026-04-28 14:32
**Files Indexed:** 247
**Languages:** TypeScript (68%), Python (32%)

## System Structure

```
MyApp/
├── frontend/          (React SPA)
│   ├── components/    45 components, 12 shared
│   ├── pages/         8 routes
│   └── state/         Redux store, 6 slices
├── backend/           (FastAPI)
│   ├── api/           12 endpoints
│   ├── services/      5 business logic modules
│   └── db/            PostgreSQL models
└── shared/            Shared types, utilities
```

## Key Components

### Frontend (src/frontend/)
- **Entry point:** src/index.tsx
- **Routing:** React Router, 8 main routes
- **State:** Redux Toolkit, async thunks for API calls
- **Hot files:** src/components/TaskList.tsx (changed 23 times)

### Backend (src/backend/)
- **Entry point:** src/main.py
- **Framework:** FastAPI
- **Database:** PostgreSQL via SQLAlchemy
- **Auth:** JWT tokens, OAuth2 flow
- **Hot files:** src/api/tasks.py (changed 18 times)

## Data Flow

1. User action in React component
2. Redux action dispatched
3. API call via axios
4. FastAPI endpoint processes
5. Service layer handles business logic
6. Database via SQLAlchemy ORM
7. Response back through layers

## Critical Dependencies

- Frontend → Backend: 12 API endpoints
- Backend → Database: 8 models
- Shared: TypeScript types used by both

## Complexity Hotspots

- `src/services/task_processor.py` - 450 lines, 8 dependencies
- `src/components/TaskBoard.tsx` - Complex state, 12 props
```

---

## CLI Interface Specification

### Command Structure
```bash
code-summarizer <SUBCOMMAND> [OPTIONS]
```

### Subcommands

#### 1. Generate Summaries
```bash
# Generate all summaries (architecture, modules, dependencies, hotspots)
code-summarizer generate

# Generate with project root specified
code-summarizer generate --project-root /path/to/project

# Output to custom directory
code-summarizer generate --output /custom/path/

# Use specific template
code-summarizer generate --template custom-template.md

# Verbose output
code-summarizer generate --verbose
```

#### 2. Update Stale Summaries
```bash
# Check if summaries are stale, regenerate if needed
code-summarizer update --if-stale

# Force regeneration even if not stale
code-summarizer update --force

# Update specific summary type
code-summarizer update --type architecture
code-summarizer update --type modules
code-summarizer update --type dependencies
code-summarizer update --type hotspots
```

#### 3. Module-Specific Generation
```bash
# Generate summary for specific module
code-summarizer module --path src/frontend --depth detailed

# Generate with different detail levels
code-summarizer module --path src/backend --depth minimal
code-summarizer module --path src/shared --depth standard
```

#### 4. LLM-Enhanced Summaries
```bash
# Use local LLM for descriptions
code-summarizer generate --llm ollama --model llama3.3

# Specify custom LLM endpoint
code-summarizer generate --llm-endpoint http://localhost:11434

# Disable LLM (structural analysis only)
code-summarizer generate --no-llm
```

#### 5. Analysis & Statistics
```bash
# Show project statistics
code-summarizer stats
# Output:
# - Total files: 247
# - Languages: TypeScript (68%), Python (32%)
# - Total symbols: 1,834
# - Dependencies: 156 internal, 42 external
# - Hottest files: [list top 5]

# Show dependency graph
code-summarizer deps --format ascii
code-summarizer deps --format json

# List hot files
code-summarizer hotspots --limit 10 --sort-by complexity
code-summarizer hotspots --limit 10 --sort-by changes
```

#### 6. Template Management
```bash
# List available templates
code-summarizer templates list

# Create new template
code-summarizer templates create --name my-template

# Validate template
code-summarizer templates validate --file template.md
```

### Output Formats
- Default: Markdown files in `.ai/context/`
- `--json`: JSON output (for programmatic use)
- `--format=compact`: Minimal token usage
- `--format=detailed`: Comprehensive summaries

### Global Options
```bash
--project-root <PATH>   # Override default project root (default: current dir)
--output <PATH>         # Output directory (default: .ai/context/)
--config <PATH>         # Custom config file
--verbose, -v           # Verbose logging
--quiet, -q             # Suppress output except errors
--help, -h              # Show help
--version, -V           # Show version
```

---

## Rust Implementation Guide

### Recommended Crate Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
# Core functionality
walkdir = "2.4"                # Directory traversal
clap = { version = "4.5", features = ["derive"] }  # CLI parsing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"             # JSON serialization
anyhow = "1.0"                 # Error handling
thiserror = "1.0"              # Custom errors

# Template rendering
tera = "1.19"                  # Jinja2-like templating

# Git integration
git2 = "0.18"                  # Git history analysis

# Code-index client
rusqlite = { version = "0.31", features = ["bundled"] }  # Query code-index DB

# Optional LLM integration
reqwest = { version = "0.11", features = ["json"] }  # HTTP client for LLM API
tokio = { version = "1.36", features = ["full"] }    # Async runtime

# Logging
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
tempfile = "3.10"              # Temp directories for tests
assert_cmd = "2.0"             # CLI testing
predicates = "3.1"             # Test assertions
```

### Module Structure

```
src/
├── main.rs              # CLI entry point, command routing
├── analyzer/
│   ├── mod.rs           # Analyzer orchestration
│   ├── structure.rs     # Project structure analysis
│   ├── hotspots.rs      # Complexity and change frequency
│   ├── dependencies.rs  # Dependency graph building
│   └── stats.rs         # Project statistics
├── code_index/
│   ├── mod.rs           # CodeIndex client interface
│   └── client.rs        # Query code-index database
├── summarizer/
│   ├── mod.rs           # Summary generation orchestration
│   ├── architecture.rs  # Architecture summary
│   ├── modules.rs       # Module summaries
│   ├── deps_graph.rs    # Dependency graph visualization
│   └── hotspots.rs      # Hotspot summary
├── templates/
│   ├── mod.rs           # Template management
│   ├── renderer.rs      # Tera integration
│   └── builtin.rs       # Built-in templates
├── git/
│   ├── mod.rs           # Git integration
│   ├── history.rs       # Commit history analysis
│   └── staleness.rs     # Detect when regeneration needed
├── llm/
│   ├── mod.rs           # LLM client (optional)
│   └── ollama.rs        # Ollama integration
├── config.rs            # Configuration management
├── error.rs             # Custom error types
└── utils.rs             # Utility functions

templates/               # Built-in template files
├── architecture.md.tera
├── module.md.tera
├── dependencies.md.tera
└── hotspots.md.tera

tests/
├── integration_test.rs  # End-to-end tests
├── analyzer_test.rs     # Analyzer tests
└── template_test.rs     # Template rendering tests
```

### Key Implementation Details

#### 1. Analyzer Module (`src/analyzer/structure.rs`)
```rust
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProjectStructure {
    pub root: PathBuf,
    pub modules: Vec<Module>,
    pub total_files: usize,
    pub language_breakdown: HashMap<String, usize>,
    pub total_lines: usize,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<FileInfo>,
    pub subdirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub lines: usize,
    pub language: String,
    pub hotness: f64,
}

pub struct CodebaseAnalyzer {
    index_client: CodeIndexClient,
    git_client: Option<GitClient>,
}

impl CodebaseAnalyzer {
    pub fn new(project_root: &Path) -> anyhow::Result<Self> {
        let index_client = CodeIndexClient::connect(project_root)?;
        let git_client = GitClient::open(project_root).ok();
        Ok(Self { index_client, git_client })
    }

    pub fn analyze_structure(&self) -> anyhow::Result<ProjectStructure> {
        // Walk directory tree
        // Group files into modules
        // Calculate statistics
        // Query code-index for symbols
        Ok(structure)
    }

    pub fn identify_hotspots(&self) -> anyhow::Result<Vec<Hotspot>> {
        // Query code-index for complexity metrics
        // Get git history for change frequency
        // Combine into hotness score
        Ok(hotspots)
    }

    pub fn build_dependency_graph(&self) -> anyhow::Result<DependencyGraph> {
        // Query code-index for dependencies
        // Build graph structure
        // Detect cycles
        Ok(graph)
    }
}
```

#### 2. CodeIndex Client (`src/code_index/client.rs`)
```rust
use rusqlite::{Connection, params};

pub struct CodeIndexClient {
    conn: Connection,
}

impl CodeIndexClient {
    pub fn connect(project_root: &Path) -> anyhow::Result<Self> {
        // Look for code-index database
        let db_path = find_index_db(project_root)?;
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn query_symbols(&self, file_path: &Path) -> anyhow::Result<Vec<Symbol>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM symbols WHERE file_path = ?1"
        )?;
        let symbols = stmt.query_map([file_path.to_str()], |row| {
            Ok(Symbol {
                name: row.get(1)?,
                kind: row.get(2)?,
                line_start: row.get(4)?,
                line_end: row.get(5)?,
                signature: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        Ok(symbols)
    }

    pub fn query_dependencies(&self, file_path: &Path) -> anyhow::Result<Vec<Dependency>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM dependencies WHERE source_file = ?1"
        )?;
        // Map to Dependency structs
        Ok(deps)
    }

    pub fn get_hot_files(&self, limit: usize) -> anyhow::Result<Vec<FileMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM files ORDER BY hotness_score DESC LIMIT ?1"
        )?;
        // Map to FileMetadata structs
        Ok(files)
    }
}
```

#### 3. Template Renderer (`src/templates/renderer.rs`)
```rust
use tera::{Tera, Context};

pub struct TemplateRenderer {
    tera: Tera,
}

impl TemplateRenderer {
    pub fn new() -> anyhow::Result<Self> {
        let mut tera = Tera::default();

        // Load built-in templates
        tera.add_raw_template("architecture", include_str!("../../templates/architecture.md.tera"))?;
        tera.add_raw_template("module", include_str!("../../templates/module.md.tera"))?;
        tera.add_raw_template("dependencies", include_str!("../../templates/dependencies.md.tera"))?;
        tera.add_raw_template("hotspots", include_str!("../../templates/hotspots.md.tera"))?;

        Ok(Self { tera })
    }

    pub fn render_architecture(&self, structure: &ProjectStructure) -> anyhow::Result<String> {
        let mut context = Context::new();
        context.insert("project", structure);
        context.insert("generated_at", &chrono::Utc::now().to_rfc3339());

        let rendered = self.tera.render("architecture", &context)?;
        Ok(rendered)
    }

    pub fn render_module(&self, module: &Module) -> anyhow::Result<String> {
        let mut context = Context::new();
        context.insert("module", module);

        let rendered = self.tera.render("module", &context)?;
        Ok(rendered)
    }
}
```

#### 4. Git Integration (`src/git/history.rs`)
```rust
use git2::{Repository, DiffOptions};

pub struct GitClient {
    repo: Repository,
}

impl GitClient {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let repo = Repository::discover(path)?;
        Ok(Self { repo })
    }

    pub fn get_file_change_count(&self, file_path: &Path) -> anyhow::Result<usize> {
        // Count commits that modified this file
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let mut count = 0;
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            // Check if this commit touched the file
            if self.commit_modified_file(&commit, file_path)? {
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn is_summary_stale(&self, summary_path: &Path) -> anyhow::Result<bool> {
        // Get summary file timestamp
        let summary_mtime = fs::metadata(summary_path)?.modified()?;

        // Get latest commit timestamp
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let latest_commit_time = commit.time();

        // Compare timestamps
        Ok(summary_mtime < latest_commit_time.into())
    }
}
```

#### 5. Hotspot Calculation (`src/analyzer/hotspots.rs`)
```rust
#[derive(Debug, Clone)]
pub struct Hotspot {
    pub file_path: PathBuf,
    pub lines_of_code: usize,
    pub change_count: usize,
    pub complexity_score: f64,
    pub hotness_score: f64,
}

pub fn calculate_hotness(
    lines: usize,
    change_count: usize,
    dependency_count: usize,
) -> f64 {
    // Weighted formula:
    // hotness = (change_count * 2.0) + (lines / 100.0) + (dependency_count * 1.5)

    let change_weight = change_count as f64 * 2.0;
    let size_weight = lines as f64 / 100.0;
    let dep_weight = dependency_count as f64 * 1.5;

    change_weight + size_weight + dep_weight
}

pub fn identify_hotspots(
    files: &[FileInfo],
    git_client: &GitClient,
    index_client: &CodeIndexClient,
) -> anyhow::Result<Vec<Hotspot>> {
    let mut hotspots = Vec::new();

    for file in files {
        let change_count = git_client.get_file_change_count(&file.path)?;
        let deps = index_client.query_dependencies(&file.path)?;

        let hotness = calculate_hotness(
            file.lines,
            change_count,
            deps.len(),
        );

        hotspots.push(Hotspot {
            file_path: file.path.clone(),
            lines_of_code: file.lines,
            change_count,
            complexity_score: file.lines as f64 / 50.0,  // Simple metric
            hotness_score: hotness,
        });
    }

    // Sort by hotness descending
    hotspots.sort_by(|a, b| b.hotness_score.partial_cmp(&a.hotness_score).unwrap());

    Ok(hotspots)
}
```

---

## Testing Requirements

### Unit Tests
- **Analyzer tests**: Project structure parsing, statistics calculation
- **Template tests**: Verify rendering with sample data
- **Git tests**: Change count, staleness detection
- **Hotspot tests**: Scoring algorithm validation

### Integration Tests
```rust
#[test]
fn test_full_summary_generation() {
    // 1. Create temp directory with test project
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_project(&temp_dir);

    // 2. Initialize code-index (mock or real)
    setup_code_index(&temp_dir);

    // 3. Run code-summarizer
    let output = run_summarizer(&temp_dir).unwrap();

    // 4. Verify output files exist
    assert!(temp_dir.path().join(".ai/context/ARCHITECTURE.md").exists());
    assert!(temp_dir.path().join(".ai/context/DEPENDENCY_GRAPH.md").exists());
    assert!(temp_dir.path().join(".ai/context/HOTSPOTS.md").exists());

    // 5. Verify content quality
    let arch_content = fs::read_to_string(temp_dir.path().join(".ai/context/ARCHITECTURE.md")).unwrap();
    assert!(arch_content.contains("System Structure"));
    assert!(arch_content.contains("Key Components"));

    // 6. Verify token count is reasonable (< 500 tokens)
    let token_count = estimate_tokens(&arch_content);
    assert!(token_count < 500);
}

#[test]
fn test_staleness_detection() {
    // Create summary
    // Make git commit
    // Verify summary is now stale
    // Regenerate
    // Verify summary is fresh
}
```

### Performance Benchmarks
- Generate architecture summary < 5 seconds for 500-file project
- Hotspot identification < 3 seconds
- Staleness check < 100ms

---

## Best Practices & Code Quality

### Error Handling
- Use `anyhow::Result` for functions that can fail
- Use `thiserror` for custom error types
- Never use `.unwrap()` in production code
- Provide helpful error messages with context
- Log errors with `log::error!`

### Code Style
- Follow Rust standard naming conventions
- Use `rustfmt` for formatting
- Use `clippy` for linting - fix all warnings
- Add documentation comments (`///`) for public APIs
- Keep functions small and focused (< 50 lines)

### Logging
- `error!` - Critical failures (missing code-index, template errors)
- `warn!` - Recoverable issues (missing git repo, LLM unavailable)
- `info!` - High-level operations (starting generation, writing files)
- `debug!` - Detailed flow (processing each module)
- `trace!` - Very verbose (template variable values)

### Performance
- Cache code-index queries where possible
- Use iterators instead of collecting intermediate vectors
- Avoid unnecessary file reads
- Batch git operations

### Security
- Validate file paths (prevent directory traversal)
- Sanitize template variables
- Limit output file sizes
- Validate LLM responses if using external API

---

## Configuration

Default config file: `~/.config/ai-tools/config.toml`

```toml
[code-summarizer]
output_dir = ".ai/context"
use_llm = false              # Enable local LLM for descriptions
llm_endpoint = "http://localhost:11434"  # Ollama endpoint
llm_model = "llama3.3"

[code-summarizer.generation]
architecture_max_tokens = 500
module_max_tokens = 300
include_hotspots = true
include_dependencies = true
hotspot_limit = 10           # Top N hotspots to include

[code-summarizer.staleness]
auto_update = true           # Auto-regenerate if stale
max_age_hours = 24          # Regenerate if older than this

[code-summarizer.templates]
custom_template_dir = "~/.config/ai-tools/templates"
architecture_template = "architecture.md.tera"
module_template = "module.md.tera"
```

---

## Integration with ai-init

When `ai-init` creates a project, it can suggest running code-summarizer:

```bash
# After project setup
ai-init myproject

# Then generate initial summaries
cd myproject
code-summarizer generate
```

The generated summaries in `.ai/context/` are ready for AI agents to consume.

---

## Integration with code-index

code-summarizer **requires** code-index to be available:

1. **Check for code-index**: Verify code-index database exists
2. **Query symbols and dependencies**: Use code-index for structural data
3. **Fall back gracefully**: If code-index unavailable, use basic file analysis

```rust
// Example integration
let index_client = match CodeIndexClient::connect(&project_root) {
    Ok(client) => Some(client),
    Err(e) => {
        warn!("code-index not available: {}. Using basic analysis.", e);
        None
    }
};
```

---

## Success Criteria

- [ ] Generates ARCHITECTURE.md with < 500 tokens
- [ ] Creates module summaries for all major directories
- [ ] Builds accurate dependency graphs
- [ ] Identifies top hotspots correctly
- [ ] Detects staleness via git commits
- [ ] Integrates with code-index seamlessly
- [ ] Optional LLM integration works with Ollama
- [ ] All tests pass with >80% code coverage
- [ ] Performance benchmarks met
- [ ] CLI interface is intuitive and well-documented

---

## Development Guidelines for AI Agents

**When working on this codebase:**

1. **Read `.ai/ARCHITECTURE.md` first** - Understand system structure
2. **Check `.ai/TOOLS.md`** for available development tools
3. **Follow `.ai/CONVENTIONS.md`** for code style and patterns
4. **Run tests before committing** - `cargo test`
5. **Update documentation** - Keep CLAUDE.md and `.ai/` files in sync with code
6. **Ask clarifying questions** - If requirements are unclear, ask before implementing
7. **Commit frequently** - Small, atomic commits with clear messages
8. **Think incrementally** - Build core features first, then optimize

### Build Order for Implementation

1. **Phase 1: Core Analysis**
   - Project structure analyzer
   - CodeIndex client integration
   - Basic statistics calculation

2. **Phase 2: Template System**
   - Set up Tera templating
   - Create built-in templates
   - Architecture summary generation

3. **Phase 3: Advanced Features**
   - Git integration for change tracking
   - Hotspot identification
   - Dependency graph generation

4. **Phase 4: Polish**
   - Staleness detection
   - Optional LLM integration
   - CLI refinement and testing

---

## Next Steps After Implementation

Once code-summarizer is built and tested:
1. Test with real projects (verify token savings)
2. Integrate with `ai-init` (optional flag: `ai-init myproject --generate-summaries`)
3. Build `context-query` (next tool in chain)
4. Build `context-packer` (uses summaries from this tool)

---

## Best Coding Practices

### Industry Standards
- **Clean Code**: Self-documenting code with clear variable names
- **DRY Principle**: Don't repeat yourself - extract common logic
- **SOLID Principles**: Single responsibility, open/closed, etc.
- **Error Handling**: Comprehensive error handling with context
- **Testing**: Unit tests, integration tests, edge case coverage
- **Documentation**: Inline comments for complex logic, API documentation
- **Performance**: Profile before optimizing, avoid premature optimization
- **Security**: Input validation, safe file operations, no command injection

### Rust-Specific Best Practices
- Leverage the type system for compile-time guarantees
- Use `Result` and `Option` instead of panicking
- Prefer owned types over references when ownership is unclear
- Use iterator chains instead of manual loops
- Implement `From`/`Into` traits for type conversions
- Use `derive` macros for common traits
- Follow the Rust API Guidelines

---

**Ready to build!** This spec provides everything needed to implement code-summarizer. Focus on core functionality first (analysis, template rendering, basic summaries), then add advanced features (git integration, hotspots, LLM support).
