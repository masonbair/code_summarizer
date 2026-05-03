# code-summarizer

Context map generator that creates hierarchical, AI-optimized markdown summaries of codebase architecture to reduce token usage by 80%

## Overview

`code-summarizer` is a CLI tool that generates intelligent, hierarchical summaries of your codebase architecture. Instead of AI agents reading thousands of tokens of raw code to understand your project, they can read pre-generated summaries of 200-500 tokens that capture the essential structure and design.

### Key Features

- **Multi-level summaries**: Architecture → Module → File → Function hierarchy
- **Semantic analysis**: Tree-sitter parsing for Rust, TypeScript, Python, and Go
- **API extraction**: Automatically extracts public APIs, types, traits, and interfaces
- **Dependency visualization**: Import graphs, call hierarchies in markdown format
- **Hotspot detection**: Identifies complex/frequently-changed areas needing attention
- **Auto-regeneration**: Detects stale summaries via git commits and updates automatically
- **Template-based**: Customizable output format using Tera templates
- **CodeIndex integration**: Leverages `code-index` (optional) for enhanced structural data
- **Optional LLM enhancement**: Use local Llama/Ollama for natural language descriptions
- **Token budget control**: Three tiers (minimal/standard/full) or custom token limits

### How It Helps AI Agents

- **Reduces context usage by 80%**: High-level overview in ~200-500 tokens vs. thousands
- **Faster comprehension**: Agents understand project structure immediately
- **Better decision making**: Identifies hot files, dependencies, and entry points
- **Persistent knowledge**: Summaries persist across AI sessions

## Installation

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Git repository (optional, for change tracking and hotspot detection)

**Optional but recommended:**
- `code-index` tool for enhanced dependency analysis (see [code-index](../code-index/))
- If code-index is unavailable, code-summarizer falls back to tree-sitter semantic parsing

### Build from Source

```bash
# Clone or navigate to the code-summarizer directory
cd code-summarizer

# Build in release mode
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .

# Verify installation
code-summarizer --version
```

### Package Installation (Arch Linux)

```bash
# Coming soon: AUR package
yay -S code-summarizer
```

## Usage

### Basic Usage

```bash
# Generate all summaries for current project
code-summarizer generate

# Generate with specific project root
code-summarizer generate --project-root /path/to/project

# Output to custom directory
code-summarizer generate --output /custom/output/

# Control output size with tiers (minimal ~200 tokens, standard ~1000, full ~3000)
code-summarizer generate --tier minimal
code-summarizer generate --tier standard  # default
code-summarizer generate --tier full

# Set custom token budget
code-summarizer generate --token-budget 500

# Disable semantic extraction (faster, but less detailed)
code-summarizer generate --semantic false
```

### Update Stale Summaries

```bash
# Check if summaries are stale, regenerate if needed
code-summarizer update --if-stale

# Force regeneration
code-summarizer update --force
```

### Module-Specific Summaries

```bash
# Generate detailed summary for a specific module
code-summarizer module --path src/frontend --depth detailed

# Generate minimal summary
code-summarizer module --path src/backend --depth minimal
```

### LLM-Enhanced Summaries

```bash
# Use local Ollama for natural language descriptions
code-summarizer generate --llm ollama --model llama3.3

# Specify custom LLM endpoint
code-summarizer generate --llm-endpoint http://localhost:11434

# Disable LLM (structural analysis only)
code-summarizer generate --no-llm
```

### Semantic Analysis

code-summarizer uses tree-sitter for deep semantic parsing of your codebase:

**Supported languages:**
- Rust: Functions, structs, enums, traits, impls, type aliases, constants
- TypeScript/JavaScript: Functions, classes, interfaces, type aliases
- Python: Functions, classes, methods
- Go: Functions, structs, interfaces

**Extracted information:**
- Public APIs and their signatures
- Type definitions with fields and methods
- Trait/interface definitions
- Trait implementations
- Entry points (main.rs, lib.rs, etc.)
- Import/export relationships

```bash
# Enable semantic extraction (default)
code-summarizer generate --semantic true

# Disable for faster processing (basic file structure only)
code-summarizer generate --semantic false
```

### Analysis & Statistics

```bash
# Show project statistics
code-summarizer stats

# Show dependency graph
code-summarizer deps --format ascii

# List hot files (complex/frequently changed)
code-summarizer hotspots --limit 10 --sort-by complexity
```

### Output Structure

After running `code-summarizer generate`, you'll find:

```
.ai/context/
├── ARCHITECTURE.md           # High-level system design
├── API.md                    # Public API documentation (functions, methods)
├── TYPES.md                  # Type definitions (structs, enums, traits, interfaces)
├── MODULE_MAPS/
│   ├── frontend.md           # Frontend module breakdown
│   ├── backend.md            # Backend services
│   └── shared.md             # Shared utilities
├── DEPENDENCY_GRAPH.md       # Visual dependency graph
└── HOTSPOTS.md               # Complex/risky areas
```

**Token counts vary by tier:**
- Minimal: ~200-500 tokens total
- Standard: ~1000-2000 tokens total (default)
- Full: ~3000+ tokens total

## Examples

### Example: Architecture Summary

```bash
$ code-summarizer generate
✓ Analyzing project structure...
✓ Performing semantic analysis with tree-sitter...
✓ Querying code-index for dependencies...
✓ Calculating hotness scores from git history...
✓ Rendering templates...
✓ Generated .ai/context/ARCHITECTURE.md (387 tokens)
✓ Generated .ai/context/API.md (842 tokens)
✓ Generated .ai/context/TYPES.md (521 tokens)
✓ Generated .ai/context/DEPENDENCY_GRAPH.md (156 tokens)
✓ Generated .ai/context/HOTSPOTS.md (223 tokens)
✓ Generated 3 module summaries

Summary complete! AI agents can now use .ai/context/ for efficient context.
Total: ~2,129 tokens (standard tier)
```

### Example: Hotspot Detection

```bash
$ code-summarizer hotspots --limit 5
Top 5 Hotspots:

1. src/services/task_processor.py
   - Lines: 450
   - Changes: 18
   - Dependencies: 8
   - Hotness: 47.5
   - Reason: High complexity, frequently modified

2. src/components/TaskBoard.tsx
   - Lines: 320
   - Changes: 23
   - Dependencies: 12
   - Hotness: 52.2
   - Reason: Complex state management, many changes

...
```

## Development

**Languages:** Rust
**Project Type:** CLI tool
**Core Dependencies:**
- tree-sitter (semantic parsing)
- git2 (commit history analysis)
- tera (template rendering)
- walkdir (directory traversal)
- clap (CLI argument parsing)

**Optional Dependencies:**
- code-index (enhanced dependency analysis via CLI)
- reqwest (LLM integration)

### AI Agent Support

This project is configured for AI agent workflows. See:
- `CLAUDE.md` - Comprehensive AI agent instructions and specifications
- `.ai/TOOLS.md` - Available custom tooling
- `.ai/ARCHITECTURE.md` - System architecture
- `.ai/CONVENTIONS.md` - Coding conventions

### Building

```bash
# Development build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- generate

# Check code quality
cargo fmt
cargo clippy
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_full_summary_generation

# Run with output
cargo test -- --nocapture
```

### Technical Implementation

**Semantic Analysis:**
- Uses tree-sitter parsers for Rust, TypeScript, Python, and Go
- Extracts AST (Abstract Syntax Tree) information
- Identifies public APIs, types, traits, and interfaces
- Tracks visibility modifiers and doc comments

**code-index Integration:**
- Invokes code-index via CLI commands (not direct database access)
- Falls back gracefully to tree-sitter if code-index is unavailable
- Uses `code-index query` commands for symbols and dependencies

**Architecture:**
- Modular design: analyzer, summarizer, templates, git integration
- Template rendering with Tera (Jinja2-like syntax)
- Incremental updates via git staleness detection

## Configuration

Create `~/.config/ai-tools/config.toml`:

```toml
[code-summarizer]
output_dir = ".ai/context"
use_llm = false
llm_endpoint = "http://localhost:11434"
llm_model = "llama3.3"

[code-summarizer.generation]
architecture_max_tokens = 500
module_max_tokens = 300
include_hotspots = true
include_dependencies = true
hotspot_limit = 10

[code-summarizer.staleness]
auto_update = true
max_age_hours = 24
```

## Integration with Other Tools

### code-index (Optional)
`code-summarizer` can leverage `code-index` for enhanced dependency analysis. If code-index is not available, it automatically falls back to tree-sitter semantic parsing.

```bash
# Start code-index daemon (optional, but provides better dependency graphs)
cd /path/to/project
code-index daemon start --watch .

# Run code-summarizer (works with or without code-index)
code-summarizer generate
```

**With code-index:**
- More accurate cross-file dependency tracking
- Better call graph generation
- Enhanced hotspot detection

**Without code-index (tree-sitter only):**
- Still extracts all public APIs and types
- Imports/exports tracked within files
- Faster analysis for smaller projects

### ai-init
When using `ai-init` to create a project, you can generate summaries immediately:

```bash
ai-init myproject
cd myproject
code-summarizer generate
```

### Future Tools
- `context-query` - Will use these summaries for faster search
- `context-packer` - Will include summaries in context assembly

## Performance

- Generates architecture summary in < 5 seconds for 500-file projects
- Hotspot identification in < 3 seconds
- Staleness checks in < 100ms
- Output optimized for minimal token usage

## Troubleshooting

### "code-index not found" or "code-index failed"
This is not an error - code-summarizer will automatically fall back to tree-sitter semantic parsing. If you want to use code-index for enhanced dependency analysis:
```bash
# Install code-index
cargo install code-index

# Start the daemon
code-index daemon start --watch .
code-index status
```

### "Git repository not found"
Git integration is optional. If no git repo exists, change tracking and hotspot features will be disabled. Initialize git:
```bash
git init
```

### Slow semantic analysis
For very large projects, disable semantic extraction for faster processing:
```bash
code-summarizer generate --semantic false
```
This will generate basic structure summaries without detailed API extraction.

### "Template rendering failed"
Ensure templates directory exists and is readable. Check:
```bash
ls -la ~/.config/ai-tools/templates/
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Follow Rust coding conventions
4. Add tests for new features
5. Submit a pull request

## Related Tools

- [ai-init](../ai-init/) - AI-ready project initializer
- [code-index](../code-index/) - Persistent semantic cache (required)
- context-query - Structure-aware search (coming soon)
- context-packer - Smart context assembly (coming soon)
