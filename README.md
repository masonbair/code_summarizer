# code-summarizer

Context map generator that creates hierarchical, AI-optimized markdown summaries of codebase architecture to reduce token usage by 80%

## Overview

`code-summarizer` is a CLI tool that generates intelligent, hierarchical summaries of your codebase architecture. Instead of AI agents reading thousands of tokens of raw code to understand your project, they can read pre-generated summaries of 200-500 tokens that capture the essential structure and design.

### Key Features

- **Multi-level summaries**: Architecture → Module → File → Function hierarchy
- **Dependency visualization**: Import graphs, call hierarchies in markdown format
- **Hotspot detection**: Identifies complex/frequently-changed areas needing attention
- **Auto-regeneration**: Detects stale summaries via git commits and updates automatically
- **Template-based**: Customizable output format using Tera templates
- **CodeIndex integration**: Leverages `code-index` for fast, accurate structural data
- **Optional LLM enhancement**: Use local Llama/Ollama for natural language descriptions

### How It Helps AI Agents

- **Reduces context usage by 80%**: High-level overview in ~200-500 tokens vs. thousands
- **Faster comprehension**: Agents understand project structure immediately
- **Better decision making**: Identifies hot files, dependencies, and entry points
- **Persistent knowledge**: Summaries persist across AI sessions

## Installation

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- `code-index` tool installed and indexed (see [code-index](../code-index/))
- Git repository (optional, for change tracking)

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
├── ARCHITECTURE.md           # High-level system design (200-500 tokens)
├── MODULE_MAPS/
│   ├── frontend.md           # Frontend module breakdown
│   ├── backend.md            # Backend services
│   └── shared.md             # Shared utilities
├── DEPENDENCY_GRAPH.md       # Visual dependency graph
└── HOTSPOTS.md               # Complex/risky areas
```

## Examples

### Example: Architecture Summary

```bash
$ code-summarizer generate
✓ Analyzing project structure...
✓ Querying code-index for symbols and dependencies...
✓ Calculating hotness scores...
✓ Rendering templates...
✓ Generated .ai/context/ARCHITECTURE.md (387 tokens)
✓ Generated .ai/context/DEPENDENCY_GRAPH.md (156 tokens)
✓ Generated .ai/context/HOTSPOTS.md (223 tokens)
✓ Generated 3 module summaries

Summary complete! AI agents can now use .ai/context/ for efficient context.
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
**Dependencies:** code-index, git2, tera, walkdir

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

### code-index
`code-summarizer` requires `code-index` to be running and indexed:

```bash
# Start code-index daemon
cd /path/to/project
code-index daemon start --watch .

# Then run code-summarizer
code-summarizer generate
```

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

### "code-index database not found"
```bash
# Ensure code-index is installed and indexed
code-index daemon start --watch .
code-index status
```

### "Git repository not found"
Git integration is optional. If no git repo exists, change tracking features will be disabled. Initialize git:
```bash
git init
```

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
