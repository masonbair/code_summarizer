# CodeSummarizer Enhancement Specification

**Tool Location:** `/home/mason/.cargo/bin/code-summarizer`
**Purpose:** Generate AI-optimized context summaries of codebases
**Current Version Issues:** Output is too shallow - provides file lists but lacks semantic understanding

---

## Problem Statement

The current CodeSummarizer generates:
- File inventories (name, line count, language)
- Basic hotspot metrics (LOC + change count)
- Directory structure trees

**What's missing:**
- Actual code semantics (function signatures, type definitions, public APIs)
- Relationship information (what calls what, what implements what)
- Intelligent summarization (what does this module DO, not just what files it contains)
- Token-efficient output (currently ~3,800 tokens for basic info that could be richer)

---

## Required Enhancements

### 1. Semantic Extraction

**Current behavior:** Lists files with line counts
**Required behavior:** Extract and display actual code semantics

#### 1.1 Public API Extraction

For each module, extract and display:

```markdown
## src/search - Public API

### Traits
- `Searcher` - Common interface for search implementations
  - `fn search(&self, query: &Query) -> Result<Vec<SearchResult>>`
  - `fn can_handle(&self, query: &Query) -> bool`
  - `fn name(&self) -> &'static str`

### Structs
- `SearchCoordinator` - Dispatches queries to appropriate searchers
  - `fn new(config: &Config) -> Self`
  - `fn search(&self, query: &Query) -> Result<Vec<SearchResult>>`

### Re-exports
- `TextSearcher`, `StructuralSearcher`, `GraphSearcher`, `HybridSearcher`
```

**Implementation approach:**
1. Use tree-sitter to parse each file
2. Extract items marked `pub` (Rust), `export` (TS/JS), or top-level (Python)
3. For traits/interfaces, include method signatures
4. For structs/classes, include public methods
5. Group by module/directory

#### 1.2 Type Definitions Summary

Extract key type definitions:

```markdown
## Core Types (src/types.rs)

### SearchResult
```rust
pub struct SearchResult {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub relevance_score: f64,
    pub context: ResultContext,
}
```

### SearchMode
```rust
pub enum SearchMode {
    Text,       // Ripgrep-based pattern matching
    Structural, // Tree-sitter AST queries
    Graph,      // Code-index relationship traversal
    Hybrid,     // Combined strategies
}
```
```

**Implementation approach:**
1. Identify type definition files (often `types.rs`, `models.py`, `types.ts`)
2. Extract struct/class/enum definitions with their fields
3. Include doc comments if present
4. Limit to public types only

### 2. Module Purpose Inference

**Current behavior:** "Module containing 24 files"
**Required behavior:** Describe what the module DOES

#### 2.1 Heuristic-Based Purpose Detection

Analyze module contents to infer purpose:

```markdown
## src/search - Search Engine Implementations

**Purpose:** Provides multiple search strategies for code querying

**Components:**
- `text.rs` - Fast text pattern matching using ripgrep
- `structural.rs` - AST-based search using tree-sitter
- `graph.rs` - Relationship traversal using code-index
- `hybrid.rs` - Combines multiple search strategies
- `mod.rs` - SearchCoordinator that dispatches to appropriate engine

**Key Patterns:**
- Strategy pattern: All searchers implement `Searcher` trait
- Coordinator pattern: `SearchCoordinator` manages strategy selection
```

**Implementation approach:**
1. Read module's `mod.rs` or `__init__.py` to understand exports
2. Scan for common patterns:
   - Trait/interface definitions → "Defines abstraction for X"
   - Multiple implementations → "Strategy pattern for X"
   - Database/IO operations → "Data access layer for X"
   - CLI/argument parsing → "Command-line interface"
3. Use doc comments from module-level documentation
4. Analyze import patterns to understand dependencies

#### 2.2 Entry Point Detection

Identify and highlight entry points:

```markdown
## Entry Points

### CLI Entry: src/main.rs:16
```rust
fn main() -> ExitCode {
    // Parses CLI args, executes search, formats output
}
```

### Library Entry: src/lib.rs
- Primary exports: `Query`, `SearchCoordinator`, `Config`
- Usage: `let results = SearchCoordinator::new(&config).search(&query)?`
```

### 3. Relationship Mapping

**Current behavior:** No relationship information
**Required behavior:** Show how modules connect

#### 3.1 Import/Dependency Graph

```markdown
## Module Dependencies

```
src/main.rs
  └── src/cli.rs
  └── src/config.rs
  └── src/search/mod.rs
        └── src/search/text.rs
        └── src/search/structural.rs
        └── src/search/graph.rs
              └── src/index/client.rs
        └── src/search/hybrid.rs
  └── src/rank/mod.rs
  └── src/format/mod.rs
```

### Cross-Module Dependencies
- `search/*` depends on `query/*` for Query type
- `search/graph.rs` depends on `index/client.rs` for database access
- All modules depend on `error.rs` for Result type
```

**Implementation approach:**
1. Parse `use`/`import` statements from each file
2. Resolve relative imports to absolute paths
3. Build directed graph of dependencies
4. Render as tree or dependency matrix

#### 3.2 Trait Implementation Map

```markdown
## Trait Implementations

### Searcher (src/search/mod.rs:24)
Implemented by:
- `TextSearcher` (src/search/text.rs:171)
- `StructuralSearcher` (src/search/structural.rs:318)
- `GraphSearcher` (src/search/graph.rs:330)
- `HybridSearcher` (src/search/hybrid.rs:132)

### Formatter (src/format/mod.rs:22)
Implemented by:
- `JsonFormatter` (src/format/json.rs:35)
- `HumanFormatter` (src/format/human.rs:55)
- `CompactFormatter` (src/format/compact.rs:25)
```

### 4. Intelligent Hotspot Analysis

**Current behavior:** `hotness = changes * 2 + lines/100 + deps * 1.5`
**Required behavior:** More nuanced complexity metrics

#### 4.1 Enhanced Metrics

```markdown
## Hotspot Analysis

### src/search/graph.rs - HIGH PRIORITY
| Metric | Value | Concern |
|--------|-------|---------|
| Lines | 514 | Large file |
| Cyclomatic Complexity | 23 | High branching |
| Nesting Depth (max) | 5 | Deep nesting |
| Dependencies | 8 | Moderately coupled |
| Public API Surface | 12 functions | Large interface |
| Test Coverage | 45% | Below target |

**Recommendations:**
- Consider splitting into `graph_search.rs` and `graph_traversal.rs`
- Reduce nesting in `find_callers_recursive`
- Add tests for edge cases in circular dependency detection
```

**Implementation approach:**
1. Calculate cyclomatic complexity (count branches: if, match, for, while)
2. Track maximum nesting depth
3. Count public vs private items
4. If test files exist, estimate coverage
5. Generate actionable recommendations

### 5. Token-Efficient Output Modes

**Current behavior:** Single verbose output format
**Required behavior:** Multiple output modes for different token budgets

#### 5.1 Output Tiers

```bash
# Minimal (~200 tokens) - Just structure
code-summarizer generate --tier minimal

# Standard (~1000 tokens) - Structure + key types
code-summarizer generate --tier standard

# Full (~3000 tokens) - Everything including APIs
code-summarizer generate --tier full

# Custom token budget
code-summarizer generate --token-budget 1500
```

#### 5.2 Minimal Output Example

```markdown
# project-name

**Type:** Rust CLI Tool
**Entry:** src/main.rs
**Modules:** cli, config, search, rank, format, query, index, types, error

**Key Types:** Query, SearchResult, SearchMode, Searcher (trait)
**Key Functions:** SearchCoordinator::search(), Query::builder()

**Architecture:** Strategy pattern with SearchCoordinator dispatching to TextSearcher|StructuralSearcher|GraphSearcher
```

### 6. Incremental Updates

**Current behavior:** Regenerates everything on each run
**Required behavior:** Only update changed files

#### 6.1 Change Detection

```bash
# Only regenerate summaries for changed files
code-summarizer update

# Output:
# Checking for changes...
# Modified: src/search/text.rs (2 minutes ago)
# Modified: src/search/mod.rs (2 minutes ago)
# Regenerating: MODULE_MAPS/src.md
# Skipping: ARCHITECTURE.md (no structural changes)
# Done in 0.3s
```

**Implementation approach:**
1. Store file hashes in `.ai/context/.cache.json`
2. On update, compare current hashes to cached
3. Only regenerate affected summaries
4. Track structural changes (new files/dirs) vs content changes

---

## New Commands

```bash
# Generate with semantic extraction (new default)
code-summarizer generate --semantic

# Generate public API documentation
code-summarizer api

# Generate dependency graph
code-summarizer deps --format mermaid

# Show trait/interface implementations
code-summarizer impls Searcher

# Quick project overview (minimal tokens)
code-summarizer overview
```

---

## Output File Changes

### Current Files
- `ARCHITECTURE.md` - Basic structure
- `DEPENDENCY_GRAPH.md` - Import lists
- `HOTSPOTS.md` - Simple metrics
- `MODULE_MAPS/*.md` - File lists

### New/Enhanced Files
- `ARCHITECTURE.md` - Enhanced with entry points, key patterns
- `API.md` - **NEW** Public API reference
- `TYPES.md` - **NEW** Core type definitions
- `DEPENDENCY_GRAPH.md` - Enhanced with visual graph
- `HOTSPOTS.md` - Enhanced with complexity metrics
- `MODULE_MAPS/*.md` - Enhanced with purpose + key functions

---

## Implementation Priority

1. **HIGH:** Semantic extraction (public APIs, type definitions)
2. **HIGH:** Module purpose inference
3. **MEDIUM:** Enhanced hotspot metrics
4. **MEDIUM:** Token-efficient output tiers
5. **LOW:** Incremental updates
6. **LOW:** Relationship mapping (can leverage code-index)

---

## Success Criteria

After enhancement, CodeSummarizer output should enable an AI agent to:
1. Understand what the project DOES without reading source files
2. Know the public API signatures for any module
3. Understand the architectural patterns in use
4. Identify where to make changes for a given task
5. Stay within token budget while maintaining understanding

---

## Technical Notes

- Use tree-sitter for parsing (already a dependency in the ecosystem)
- Consider integrating with code-index for relationship data
- Output should be valid Markdown with proper code blocks
- Include generation timestamp and invalidation hints
- Support Rust, TypeScript, Python, Go initially

---

## Additional Enhancements from Testing & Roadmap

### Priority 1: Improved Summary Output Quality (CRITICAL)

**Problem**: Summaries lack concrete code examples
**Current**: High-level descriptions only
**Required**: Include code snippets and API examples

**Enhanced ARCHITECTURE.md Format**:
```markdown
## Searcher Module
High-level search execution engine.

**Example Usage**:
```rust
let searcher = SearcherBuilder::new()
    .binary_detection(BinaryDetection::quit(b'\x00'))
    .build();
searcher.search_path(&matcher, &path, printer)?;
```

**Key API**:
- `SearcherBuilder::new()` - Create searcher
- `search_path()` - Search single file
- `search_reader()` - Search from reader
```

**Implementation**:
- Extract representative code snippets during summarization
- Use tree-sitter to find well-formed examples
- Include function signatures with examples
- Add "Quick Start" sections to summaries

**Impact**: Better AI agent understanding, faster onboarding

---

### Priority 2: Git-Aware Staleness Detection

**Problem**: Manual summary regeneration, no auto-detection
**Required**: Automatic detection of significant changes

**New Commands**:
```bash
# Check if summaries are stale
code-summarizer check-freshness

# Output:
# ⚠️  Summaries are stale:
#   - 23 files changed since last generation
#   - 5 hotspot files modified
#   - Estimated impact: HIGH
#   - Recommend: Regenerate now

# Auto-regenerate on significant changes
code-summarizer daemon start --auto-regenerate-threshold high
```

**Implementation**:
```rust
pub struct StalenessAnalyzer {
    git_repo: Repository,
    last_generation: DateTime,
}

impl StalenessAnalyzer {
    pub fn analyze(&self) -> StalenessReport {
        let changed_files = self.files_changed_since(self.last_generation);
        let impact = self.calculate_impact(&changed_files);

        StalenessReport {
            files_changed: changed_files.len(),
            hotspot_files_changed: self.count_hotspot_changes(&changed_files),
            impact_level: impact,  // Low | Medium | High | Critical
            recommendation: self.recommend(impact),
        }
    }

    fn calculate_impact(&self, files: &[PathBuf]) -> ImpactLevel {
        // High impact if:
        // - Hotspot files changed
        // - Core architecture files changed
        // - Many files changed (>20%)
        // - Dependencies changed significantly
    }
}
```

**Git Hooks Integration**:
```bash
# .git/hooks/post-merge
#!/bin/bash
if code-summarizer check-freshness --threshold medium; then
    echo "⚠️  Summaries are stale. Regenerating..."
    code-summarizer update --force
fi
```

---

### Priority 3: Watch Mode (Quick Win)

**Implementation**:
```bash
# Auto-regenerate on file changes
code-summarizer watch

# Like nodemon but for summaries
# Useful during active development
```

**Uses**: `notify` crate with debouncing

---

### Future Enhancements

**Diff & Change Analysis**:
```bash
# Compare current state to last commit
code-summarizer diff HEAD~1

# Output:
# Symbols added:
#   + src/auth/oauth.rs::handle_oauth (line 42)
# Symbols removed:
#   - src/auth/basic.rs::basic_auth (line 23)
# Hotspot changes:
#   src/api/routes.rs: complexity 234 → 312 (+33%)

# Summarize changes for AI
code-summarizer diff --since HEAD~1 --output changes.md
```

**Documentation Indexing**:
```bash
# Index documentation alongside code
code-summarizer generate --include-docs

# Indexes:
# - README.md, CONTRIBUTING.md, etc.
# - Docstrings / doc comments
# - Architecture Decision Records (ADRs)
# - Wiki pages (if linked)

# Search across code AND docs
code-summarizer query --text "authentication" --include docs
```

**Module-Level Summaries with Examples**:
- Extract key function signatures
- Include representative code samples
- Document common usage patterns
- Link to related modules

---

### Enhanced Output Files

**New Files to Generate**:
- `ARCHITECTURE.md` - Enhanced with code examples and entry points
- `API.md` - **NEW** Public API reference with examples
- `TYPES.md` - **NEW** Core type definitions with usage
- `EXAMPLES.md` - **NEW** Code snippets showing common patterns
- `HOTSPOTS.md` - Enhanced with actionable recommendations

**Token Optimization**:
```bash
# Generate with different detail levels
code-summarizer generate --tier minimal    # ~200 tokens
code-summarizer generate --tier standard   # ~1000 tokens
code-summarizer generate --tier full       # ~3000 tokens
code-summarizer generate --token-budget 1500  # Custom
```

---

*This specification should be fed to an AI agent to implement the enhancements.*
