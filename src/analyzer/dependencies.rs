use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::structure::ProjectAnalyzer;

/// Represents a dependency between two files/modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub source: PathBuf,
    pub target: PathBuf,
    pub dep_type: DependencyType,
}

/// Type of dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyType {
    Import,
    Include,
    Use,
    Require,
    From,
    Unknown,
}

/// Graph of dependencies in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<PathBuf>,
    pub edges: Vec<Dependency>,
    pub internal_count: usize,
    pub external_count: usize,
    pub cycles: Vec<Vec<PathBuf>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            internal_count: 0,
            external_count: 0,
            cycles: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, path: PathBuf) {
        if !self.nodes.contains(&path) {
            self.nodes.push(path);
        }
    }

    /// Add an edge (dependency) to the graph
    pub fn add_edge(&mut self, dep: Dependency) {
        if !self.edges.contains(&dep) {
            self.edges.push(dep);
        }
    }

    /// Convert to ASCII representation
    pub fn to_ascii(&self) -> String {
        let mut output = String::new();
        output.push_str("Dependency Graph\n");
        output.push_str("================\n\n");

        output.push_str(&format!("Nodes: {}\n", self.nodes.len()));
        output.push_str(&format!("Edges: {}\n", self.edges.len()));
        output.push_str(&format!("Internal dependencies: {}\n", self.internal_count));
        output.push_str(&format!("External dependencies: {}\n", self.external_count));

        if !self.cycles.is_empty() {
            output.push_str(&format!("\nCircular dependencies: {}\n", self.cycles.len()));
            for (i, cycle) in self.cycles.iter().enumerate() {
                output.push_str(&format!(
                    "  {}. {}\n",
                    i + 1,
                    cycle
                        .iter()
                        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ));
            }
        }

        output.push_str("\nDependency List:\n");
        output.push_str("----------------\n");

        // Group by source file
        let mut by_source: HashMap<&PathBuf, Vec<&Dependency>> = HashMap::new();
        for edge in &self.edges {
            by_source.entry(&edge.source).or_default().push(edge);
        }

        for (source, deps) in by_source {
            let source_name = source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            output.push_str(&format!("\n{}\n", source_name));
            for dep in deps {
                let target_name = dep
                    .target
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                output.push_str(&format!("  └─> {}\n", target_name));
            }
        }

        output
    }

    /// Detect circular dependencies in the graph
    pub fn detect_cycles(&mut self) {
        // Build adjacency list using indices to avoid lifetime issues
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

        // Create node index map
        let node_to_idx: HashMap<&PathBuf, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect();

        for edge in &self.edges {
            if let (Some(&src_idx), Some(&tgt_idx)) =
                (node_to_idx.get(&edge.source), node_to_idx.get(&edge.target))
            {
                adj.entry(src_idx).or_default().push(tgt_idx);
            }
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        let mut found_cycles = Vec::new();

        for i in 0..self.nodes.len() {
            if !visited.contains(&i) {
                Self::dfs_cycle_helper(
                    i,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &self.nodes,
                    &mut found_cycles,
                );
            }
        }

        self.cycles = found_cycles;
    }

    fn dfs_cycle_helper(
        node_idx: usize,
        adj: &HashMap<usize, Vec<usize>>,
        visited: &mut HashSet<usize>,
        rec_stack: &mut HashSet<usize>,
        path: &mut Vec<usize>,
        nodes: &[PathBuf],
        found_cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(node_idx);
        rec_stack.insert(node_idx);
        path.push(node_idx);

        if let Some(neighbors) = adj.get(&node_idx) {
            for &neighbor_idx in neighbors {
                if !visited.contains(&neighbor_idx) {
                    Self::dfs_cycle_helper(
                        neighbor_idx,
                        adj,
                        visited,
                        rec_stack,
                        path,
                        nodes,
                        found_cycles,
                    );
                } else if rec_stack.contains(&neighbor_idx) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|&p| p == neighbor_idx).unwrap_or(0);
                    let mut cycle: Vec<PathBuf> = path[cycle_start..]
                        .iter()
                        .map(|&idx| nodes[idx].clone())
                        .collect();
                    cycle.push(nodes[neighbor_idx].clone());
                    found_cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(&node_idx);
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectAnalyzer {
    /// Build the dependency graph for the project
    pub fn build_dependency_graph(&self) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph::new();

        // If we have code-index, use it for accurate dependencies
        if let Some(index) = self.index_client() {
            let deps = index.get_all_dependencies()?;

            for dep in deps {
                graph.add_node(dep.source.clone());
                graph.add_node(dep.target.clone());
                graph.add_edge(dep);
            }

            // Count internal vs external
            let project_root = self.project_root();
            for edge in &graph.edges {
                if edge.target.starts_with(project_root) {
                    graph.internal_count += 1;
                } else {
                    graph.external_count += 1;
                }
            }
        } else {
            // Fall back to basic analysis - just list files without dependencies
            let structure = self.analyze_structure()?;
            for module in &structure.modules {
                for file in &module.files {
                    graph.add_node(file.relative_path.clone());
                }
            }
        }

        // Detect cycles
        graph.detect_cycles();

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert_eq!(graph.internal_count, 0);
        assert_eq!(graph.external_count, 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("src/main.rs"));
        graph.add_node(PathBuf::from("src/lib.rs"));

        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_add_node_no_duplicates() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("src/main.rs"));
        graph.add_node(PathBuf::from("src/main.rs"));

        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DependencyGraph::new();
        let dep = Dependency {
            source: PathBuf::from("src/main.rs"),
            target: PathBuf::from("src/lib.rs"),
            dep_type: DependencyType::Use,
        };
        graph.add_edge(dep);

        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_add_edge_no_duplicates() {
        let mut graph = DependencyGraph::new();
        let dep = Dependency {
            source: PathBuf::from("src/main.rs"),
            target: PathBuf::from("src/lib.rs"),
            dep_type: DependencyType::Use,
        };
        graph.add_edge(dep.clone());
        graph.add_edge(dep);

        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_to_ascii_contains_basic_info() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("src/main.rs"));
        graph.add_node(PathBuf::from("src/lib.rs"));
        graph.add_edge(Dependency {
            source: PathBuf::from("src/main.rs"),
            target: PathBuf::from("src/lib.rs"),
            dep_type: DependencyType::Use,
        });
        graph.internal_count = 1;

        let ascii = graph.to_ascii();
        assert!(ascii.contains("Dependency Graph"));
        assert!(ascii.contains("Nodes: 2"));
        assert!(ascii.contains("Edges: 1"));
        assert!(ascii.contains("Internal dependencies: 1"));
    }

    #[test]
    fn test_detect_cycles_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("a.rs"));
        graph.add_node(PathBuf::from("b.rs"));
        graph.add_node(PathBuf::from("c.rs"));
        graph.add_edge(Dependency {
            source: PathBuf::from("a.rs"),
            target: PathBuf::from("b.rs"),
            dep_type: DependencyType::Use,
        });
        graph.add_edge(Dependency {
            source: PathBuf::from("b.rs"),
            target: PathBuf::from("c.rs"),
            dep_type: DependencyType::Use,
        });

        graph.detect_cycles();
        assert!(graph.cycles.is_empty());
    }

    #[test]
    fn test_detect_cycles_with_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_node(PathBuf::from("a.rs"));
        graph.add_node(PathBuf::from("b.rs"));
        graph.add_edge(Dependency {
            source: PathBuf::from("a.rs"),
            target: PathBuf::from("b.rs"),
            dep_type: DependencyType::Use,
        });
        graph.add_edge(Dependency {
            source: PathBuf::from("b.rs"),
            target: PathBuf::from("a.rs"),
            dep_type: DependencyType::Use,
        });

        graph.detect_cycles();
        assert!(!graph.cycles.is_empty());
    }

    #[test]
    fn test_dependency_type_variants() {
        assert_eq!(
            format!("{:?}", DependencyType::Import),
            "Import"
        );
        assert_eq!(
            format!("{:?}", DependencyType::Use),
            "Use"
        );
        assert_eq!(
            format!("{:?}", DependencyType::Require),
            "Require"
        );
    }
}
