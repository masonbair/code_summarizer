use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tree_sitter::Parser;

/// Represents a public API item (function, struct, trait, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicApi {
    pub name: String,
    pub kind: ApiKind,
    pub signature: String,
    pub file_path: String,
    pub line: usize,
    pub doc_comment: Option<String>,
    pub visibility: Visibility,
}

/// Type of API item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApiKind {
    Function,
    Struct,
    Enum,
    Trait,
    TraitImpl,
    TypeAlias,
    Const,
    Static,
    Module,
    Macro,
}

/// Visibility of an item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    PublicCrate,
    PublicSuper,
    Private,
}

/// A trait/interface with its methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<MethodSignature>,
    pub file_path: String,
    pub line: usize,
    pub doc_comment: Option<String>,
}

/// A struct/class with its fields and methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
    pub kind: TypeDefKind,
    pub fields: Vec<FieldDef>,
    pub methods: Vec<MethodSignature>,
    pub file_path: String,
    pub line: usize,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TypeDefKind {
    Struct,
    Enum,
    Class,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub type_annotation: String,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub signature: String,
    pub is_async: bool,
    pub visibility: Visibility,
}

/// Trait implementation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitImplInfo {
    pub trait_name: String,
    pub impl_type: String,
    pub file_path: String,
    pub line: usize,
}

/// Entry point detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub kind: EntryPointKind,
    pub file_path: String,
    pub line: usize,
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryPointKind {
    MainFunction,
    LibraryRoot,
    ModuleRoot,
    BinaryEntry,
}

/// Semantic analysis results for a file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileSemantics {
    pub file_path: String,
    pub language: String,
    pub public_apis: Vec<PublicApi>,
    pub traits: Vec<TraitDef>,
    pub types: Vec<TypeDef>,
    pub trait_impls: Vec<TraitImplInfo>,
    pub entry_points: Vec<EntryPoint>,
    pub imports: Vec<String>,
    pub re_exports: Vec<String>,
}

/// Semantic analyzer using tree-sitter
pub struct SemanticAnalyzer {
    rust_parser: Option<Parser>,
    ts_parser: Option<Parser>,
    python_parser: Option<Parser>,
    go_parser: Option<Parser>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer with all supported language parsers
    pub fn new() -> Result<Self> {
        let rust_parser = Self::create_rust_parser().ok();
        let ts_parser = Self::create_typescript_parser().ok();
        let python_parser = Self::create_python_parser().ok();
        let go_parser = Self::create_go_parser().ok();

        Ok(Self {
            rust_parser,
            ts_parser,
            python_parser,
            go_parser,
        })
    }

    fn create_rust_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_rust::LANGUAGE;
        parser.set_language(&language.into()).context("Failed to set Rust language")?;
        Ok(parser)
    }

    fn create_typescript_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser.set_language(&language.into()).context("Failed to set TypeScript language")?;
        Ok(parser)
    }

    fn create_python_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into()).context("Failed to set Python language")?;
        Ok(parser)
    }

    fn create_go_parser() -> Result<Parser> {
        let mut parser = Parser::new();
        let language = tree_sitter_go::LANGUAGE;
        parser.set_language(&language.into()).context("Failed to set Go language")?;
        Ok(parser)
    }

    /// Analyze a file and extract semantic information
    pub fn analyze_file(&mut self, path: &Path) -> Result<FileSemantics> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_path = path.to_string_lossy().to_string();

        match ext {
            "rs" => self.analyze_rust(&content, &file_path),
            "ts" | "tsx" => self.analyze_typescript(&content, &file_path),
            "py" => self.analyze_python(&content, &file_path),
            "go" => self.analyze_go(&content, &file_path),
            _ => Ok(FileSemantics {
                file_path,
                language: "Unknown".to_string(),
                ..Default::default()
            }),
        }
    }

    /// Analyze Rust source code
    fn analyze_rust(&mut self, content: &str, file_path: &str) -> Result<FileSemantics> {
        let parser = self.rust_parser.as_mut()
            .context("Rust parser not available")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Rust code")?;

        let root = tree.root_node();
        let mut semantics = FileSemantics {
            file_path: file_path.to_string(),
            language: "Rust".to_string(),
            ..Default::default()
        };

        // Extract items using tree traversal
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "function_item" => {
                    if let Some(api) = self.extract_rust_function(&child, content, file_path) {
                        if api.visibility == Visibility::Public {
                            semantics.public_apis.push(api);
                        }
                    }
                }
                "struct_item" => {
                    if let Some(type_def) = self.extract_rust_struct(&child, content, file_path) {
                        semantics.types.push(type_def);
                    }
                }
                "enum_item" => {
                    if let Some(type_def) = self.extract_rust_enum(&child, content, file_path) {
                        semantics.types.push(type_def);
                    }
                }
                "trait_item" => {
                    if let Some(trait_def) = self.extract_rust_trait(&child, content, file_path) {
                        semantics.traits.push(trait_def);
                    }
                }
                "impl_item" => {
                    if let Some(impl_info) = self.extract_rust_impl(&child, content, file_path) {
                        semantics.trait_impls.push(impl_info);
                    }
                }
                "use_declaration" => {
                    if let Some(import) = self.extract_rust_use(&child, content) {
                        semantics.imports.push(import);
                    }
                }
                "type_item" => {
                    if let Some(api) = self.extract_rust_type_alias(&child, content, file_path) {
                        if api.visibility == Visibility::Public {
                            semantics.public_apis.push(api);
                        }
                    }
                }
                "const_item" | "static_item" => {
                    if let Some(api) = self.extract_rust_const(&child, content, file_path) {
                        if api.visibility == Visibility::Public {
                            semantics.public_apis.push(api);
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for entry points
        if file_path.ends_with("main.rs") {
            if semantics.public_apis.iter().any(|a| a.name == "main") {
                semantics.entry_points.push(EntryPoint {
                    kind: EntryPointKind::MainFunction,
                    file_path: file_path.to_string(),
                    line: 1,
                    exports: vec![],
                });
            }
        } else if file_path.ends_with("lib.rs") {
            let exports: Vec<String> = semantics.public_apis.iter()
                .map(|a| a.name.clone())
                .collect();
            semantics.entry_points.push(EntryPoint {
                kind: EntryPointKind::LibraryRoot,
                file_path: file_path.to_string(),
                line: 1,
                exports,
            });
        } else if file_path.ends_with("mod.rs") {
            let exports: Vec<String> = semantics.public_apis.iter()
                .map(|a| a.name.clone())
                .collect();
            semantics.entry_points.push(EntryPoint {
                kind: EntryPointKind::ModuleRoot,
                file_path: file_path.to_string(),
                line: 1,
                exports,
            });
        }

        Ok(semantics)
    }

    fn extract_rust_function(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let visibility = self.get_rust_visibility(node, content);

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        // Build signature from function declaration
        let params_node = node.child_by_field_name("parameters");
        let return_type = node.child_by_field_name("return_type");

        let params_text = params_node.map(|n| self.node_text(n, content)).unwrap_or("()".to_string());
        let return_text = return_type.map(|n| format!(" -> {}", self.node_text(n, content))).unwrap_or_default();

        let signature = format!("fn {}{}{}", name, params_text, return_text);

        Some(PublicApi {
            name,
            kind: ApiKind::Function,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
            visibility,
        })
    }

    fn extract_rust_struct(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TypeDef> {
        let visibility = self.get_rust_visibility(node, content);
        if visibility == Visibility::Private {
            return None;
        }

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut fields = Vec::new();

        // Look for field_declaration_list
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    if let Some(field_name) = child.child_by_field_name("name") {
                        let field_type = child.child_by_field_name("type")
                            .map(|n| self.node_text(n, content))
                            .unwrap_or_default();
                        let field_vis = self.get_rust_visibility(&child, content);

                        fields.push(FieldDef {
                            name: self.node_text(field_name, content),
                            type_annotation: field_type,
                            visibility: field_vis,
                        });
                    }
                }
            }
        }

        Some(TypeDef {
            name,
            kind: TypeDefKind::Struct,
            fields,
            methods: vec![],
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
        })
    }

    fn extract_rust_enum(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TypeDef> {
        let visibility = self.get_rust_visibility(node, content);
        if visibility == Visibility::Private {
            return None;
        }

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut fields = Vec::new();

        // Look for enum_variant_list
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "enum_variant" {
                    if let Some(variant_name) = child.child_by_field_name("name") {
                        fields.push(FieldDef {
                            name: self.node_text(variant_name, content),
                            type_annotation: String::new(),
                            visibility: Visibility::Public,
                        });
                    }
                }
            }
        }

        Some(TypeDef {
            name,
            kind: TypeDefKind::Enum,
            fields,
            methods: vec![],
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
        })
    }

    fn extract_rust_trait(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TraitDef> {
        let visibility = self.get_rust_visibility(node, content);
        if visibility == Visibility::Private {
            return None;
        }

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut methods = Vec::new();

        // Look for declaration_list
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_signature_item" || child.kind() == "function_item" {
                    if let Some(method_name) = child.child_by_field_name("name") {
                        let params = child.child_by_field_name("parameters")
                            .map(|n| self.node_text(n, content))
                            .unwrap_or("()".to_string());
                        let return_type = child.child_by_field_name("return_type")
                            .map(|n| format!(" -> {}", self.node_text(n, content)))
                            .unwrap_or_default();

                        methods.push(MethodSignature {
                            name: self.node_text(method_name, content),
                            signature: format!("fn {}{}{}", self.node_text(method_name, content), params, return_type),
                            is_async: false,
                            visibility: Visibility::Public,
                        });
                    }
                }
            }
        }

        Some(TraitDef {
            name,
            methods,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
        })
    }

    fn extract_rust_impl(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TraitImplInfo> {
        // Check if this is a trait impl (has trait field)
        let trait_node = node.child_by_field_name("trait")?;
        let type_node = node.child_by_field_name("type")?;

        Some(TraitImplInfo {
            trait_name: self.node_text(trait_node, content),
            impl_type: self.node_text(type_node, content),
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
        })
    }

    fn extract_rust_use(&self, node: &tree_sitter::Node, content: &str) -> Option<String> {
        // Get the argument of the use statement
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "use_tree" || child.kind() == "scoped_identifier" {
                return Some(self.node_text(child, content));
            }
        }
        None
    }

    fn extract_rust_type_alias(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let visibility = self.get_rust_visibility(node, content);

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let type_node = node.child_by_field_name("type");
        let type_text = type_node.map(|n| self.node_text(n, content)).unwrap_or_default();

        let signature = format!("type {} = {}", name, type_text);

        Some(PublicApi {
            name,
            kind: ApiKind::TypeAlias,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
            visibility,
        })
    }

    fn extract_rust_const(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let visibility = self.get_rust_visibility(node, content);

        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let type_node = node.child_by_field_name("type");
        let type_text = type_node.map(|n| self.node_text(n, content)).unwrap_or_default();

        let kind = if node.kind() == "const_item" { ApiKind::Const } else { ApiKind::Static };
        let keyword = if node.kind() == "const_item" { "const" } else { "static" };
        let signature = format!("{} {}: {}", keyword, name, type_text);

        Some(PublicApi {
            name,
            kind,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: self.get_doc_comment(node, content),
            visibility,
        })
    }

    fn get_rust_visibility(&self, node: &tree_sitter::Node, content: &str) -> Visibility {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                let text = self.node_text(child, content);
                if text.contains("crate") {
                    return Visibility::PublicCrate;
                } else if text.contains("super") {
                    return Visibility::PublicSuper;
                } else if text.starts_with("pub") {
                    return Visibility::Public;
                }
            }
        }
        Visibility::Private
    }

    fn get_doc_comment(&self, node: &tree_sitter::Node, content: &str) -> Option<String> {
        // Look for preceding comment nodes
        if let Some(prev) = node.prev_sibling() {
            if prev.kind() == "line_comment" || prev.kind() == "block_comment" {
                let text = self.node_text(prev, content);
                if text.starts_with("///") || text.starts_with("//!") {
                    return Some(text.trim_start_matches('/').trim().to_string());
                }
            }
        }
        None
    }

    fn node_text(&self, node: tree_sitter::Node, content: &str) -> String {
        content[node.byte_range()].to_string()
    }

    // TypeScript/JavaScript analysis
    fn analyze_typescript(&mut self, content: &str, file_path: &str) -> Result<FileSemantics> {
        let parser = self.ts_parser.as_mut()
            .context("TypeScript parser not available")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse TypeScript code")?;

        let root = tree.root_node();
        let mut semantics = FileSemantics {
            file_path: file_path.to_string(),
            language: "TypeScript".to_string(),
            ..Default::default()
        };

        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "export_statement" => {
                    // Handle exported items
                    if let Some(api) = self.extract_ts_export(&child, content, file_path) {
                        semantics.public_apis.push(api);
                    }
                }
                "function_declaration" => {
                    if let Some(api) = self.extract_ts_function(&child, content, file_path) {
                        semantics.public_apis.push(api);
                    }
                }
                "class_declaration" => {
                    if let Some(type_def) = self.extract_ts_class(&child, content, file_path) {
                        semantics.types.push(type_def);
                    }
                }
                "interface_declaration" => {
                    if let Some(trait_def) = self.extract_ts_interface(&child, content, file_path) {
                        semantics.traits.push(trait_def);
                    }
                }
                "type_alias_declaration" => {
                    if let Some(api) = self.extract_ts_type_alias(&child, content, file_path) {
                        semantics.public_apis.push(api);
                    }
                }
                "import_statement" => {
                    if let Some(import) = self.extract_ts_import(&child, content) {
                        semantics.imports.push(import);
                    }
                }
                _ => {}
            }
        }

        Ok(semantics)
    }

    fn extract_ts_export(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_declaration" {
                return self.extract_ts_function(&child, content, file_path);
            }
        }
        None
    }

    fn extract_ts_function(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let params = node.child_by_field_name("parameters")
            .map(|n| self.node_text(n, content))
            .unwrap_or("()".to_string());
        let return_type = node.child_by_field_name("return_type")
            .map(|n| format!(": {}", self.node_text(n, content)))
            .unwrap_or_default();

        let signature = format!("function {}{}{}", name, params, return_type);

        Some(PublicApi {
            name,
            kind: ApiKind::Function,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
            visibility: Visibility::Public,
        })
    }

    fn extract_ts_class(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TypeDef> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "public_field_definition" | "field_definition" => {
                        if let Some(field_name) = child.child_by_field_name("name") {
                            let type_ann = child.child_by_field_name("type")
                                .map(|n| self.node_text(n, content))
                                .unwrap_or_default();
                            fields.push(FieldDef {
                                name: self.node_text(field_name, content),
                                type_annotation: type_ann,
                                visibility: Visibility::Public,
                            });
                        }
                    }
                    "method_definition" => {
                        if let Some(method_name) = child.child_by_field_name("name") {
                            let params = child.child_by_field_name("parameters")
                                .map(|n| self.node_text(n, content))
                                .unwrap_or("()".to_string());
                            methods.push(MethodSignature {
                                name: self.node_text(method_name, content),
                                signature: format!("{}()", self.node_text(method_name, content)),
                                is_async: false,
                                visibility: Visibility::Public,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Some(TypeDef {
            name,
            kind: TypeDefKind::Class,
            fields,
            methods,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
        })
    }

    fn extract_ts_interface(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TraitDef> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "method_signature" {
                    if let Some(method_name) = child.child_by_field_name("name") {
                        methods.push(MethodSignature {
                            name: self.node_text(method_name, content),
                            signature: self.node_text(child, content),
                            is_async: false,
                            visibility: Visibility::Public,
                        });
                    }
                }
            }
        }

        Some(TraitDef {
            name,
            methods,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
        })
    }

    fn extract_ts_type_alias(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);
        let signature = self.node_text(*node, content);

        Some(PublicApi {
            name,
            kind: ApiKind::TypeAlias,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
            visibility: Visibility::Public,
        })
    }

    fn extract_ts_import(&self, node: &tree_sitter::Node, content: &str) -> Option<String> {
        Some(self.node_text(*node, content))
    }

    // Python analysis
    fn analyze_python(&mut self, content: &str, file_path: &str) -> Result<FileSemantics> {
        let parser = self.python_parser.as_mut()
            .context("Python parser not available")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Python code")?;

        let root = tree.root_node();
        let mut semantics = FileSemantics {
            file_path: file_path.to_string(),
            language: "Python".to_string(),
            ..Default::default()
        };

        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Some(api) = self.extract_python_function(&child, content, file_path) {
                        // In Python, top-level non-underscore functions are public
                        if !api.name.starts_with('_') {
                            semantics.public_apis.push(api);
                        }
                    }
                }
                "class_definition" => {
                    if let Some(type_def) = self.extract_python_class(&child, content, file_path) {
                        if !type_def.name.starts_with('_') {
                            semantics.types.push(type_def);
                        }
                    }
                }
                "import_statement" | "import_from_statement" => {
                    semantics.imports.push(self.node_text(child, content));
                }
                _ => {}
            }
        }

        Ok(semantics)
    }

    fn extract_python_function(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let params = node.child_by_field_name("parameters")
            .map(|n| self.node_text(n, content))
            .unwrap_or("()".to_string());
        let return_type = node.child_by_field_name("return_type")
            .map(|n| format!(" -> {}", self.node_text(n, content)))
            .unwrap_or_default();

        let signature = format!("def {}{}{}", name, params, return_type);

        Some(PublicApi {
            name,
            kind: ApiKind::Function,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
            visibility: Visibility::Public,
        })
    }

    fn extract_python_class(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TypeDef> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        let mut methods = Vec::new();

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_definition" {
                    if let Some(method_name) = child.child_by_field_name("name") {
                        let method_name_text = self.node_text(method_name, content);
                        if !method_name_text.starts_with('_') || method_name_text == "__init__" {
                            let params = child.child_by_field_name("parameters")
                                .map(|n| self.node_text(n, content))
                                .unwrap_or("()".to_string());
                            methods.push(MethodSignature {
                                name: method_name_text.clone(),
                                signature: format!("def {}{}", method_name_text, params),
                                is_async: false,
                                visibility: if method_name_text.starts_with('_') {
                                    Visibility::Private
                                } else {
                                    Visibility::Public
                                },
                            });
                        }
                    }
                }
            }
        }

        Some(TypeDef {
            name,
            kind: TypeDefKind::Class,
            fields: vec![],
            methods,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
        })
    }

    // Go analysis
    fn analyze_go(&mut self, content: &str, file_path: &str) -> Result<FileSemantics> {
        let parser = self.go_parser.as_mut()
            .context("Go parser not available")?;

        let tree = parser.parse(content, None)
            .context("Failed to parse Go code")?;

        let root = tree.root_node();
        let mut semantics = FileSemantics {
            file_path: file_path.to_string(),
            language: "Go".to_string(),
            ..Default::default()
        };

        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(api) = self.extract_go_function(&child, content, file_path) {
                        if api.visibility == Visibility::Public {
                            semantics.public_apis.push(api);
                        }
                    }
                }
                "type_declaration" => {
                    // Handle struct and interface types
                    let mut inner_cursor = child.walk();
                    for inner in child.children(&mut inner_cursor) {
                        if inner.kind() == "type_spec" {
                            if let Some(type_def) = self.extract_go_type(&inner, content, file_path) {
                                semantics.types.push(type_def);
                            }
                        }
                    }
                }
                "import_declaration" => {
                    semantics.imports.push(self.node_text(child, content));
                }
                _ => {}
            }
        }

        Ok(semantics)
    }

    fn extract_go_function(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<PublicApi> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        // In Go, exported (public) names start with uppercase
        let visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        let params = node.child_by_field_name("parameters")
            .map(|n| self.node_text(n, content))
            .unwrap_or("()".to_string());
        let result = node.child_by_field_name("result")
            .map(|n| format!(" {}", self.node_text(n, content)))
            .unwrap_or_default();

        let signature = format!("func {}{}{}", name, params, result);

        Some(PublicApi {
            name,
            kind: ApiKind::Function,
            signature,
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
            visibility,
        })
    }

    fn extract_go_type(&self, node: &tree_sitter::Node, content: &str, file_path: &str) -> Option<TypeDef> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(name_node, content);

        // In Go, exported (public) names start with uppercase
        if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return None;
        }

        let type_node = node.child_by_field_name("type")?;
        let kind = match type_node.kind() {
            "struct_type" => TypeDefKind::Struct,
            "interface_type" => TypeDefKind::Interface,
            _ => TypeDefKind::Struct,
        };

        Some(TypeDef {
            name,
            kind,
            fields: vec![],
            methods: vec![],
            file_path: file_path.to_string(),
            line: node.start_position().row + 1,
            doc_comment: None,
        })
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create SemanticAnalyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_rust_function_extraction() {
        let mut analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}

fn private_fn() {}
"#;
        let result = analyzer.analyze_rust(code, "test.rs").unwrap();

        assert_eq!(result.public_apis.len(), 1);
        assert_eq!(result.public_apis[0].name, "hello");
        assert!(result.public_apis[0].signature.contains("fn hello"));
    }

    #[test]
    fn test_rust_struct_extraction() {
        let mut analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
pub struct User {
    pub name: String,
    pub age: u32,
    password: String,
}
"#;
        let result = analyzer.analyze_rust(code, "test.rs").unwrap();

        assert_eq!(result.types.len(), 1);
        assert_eq!(result.types[0].name, "User");
        assert!(result.types[0].fields.len() >= 2);
    }

    #[test]
    fn test_rust_trait_extraction() {
        let mut analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
pub trait Greeter {
    fn greet(&self) -> String;
    fn say_goodbye(&self);
}
"#;
        let result = analyzer.analyze_rust(code, "test.rs").unwrap();

        assert_eq!(result.traits.len(), 1);
        assert_eq!(result.traits[0].name, "Greeter");
        assert_eq!(result.traits[0].methods.len(), 2);
    }

    #[test]
    fn test_rust_impl_trait_extraction() {
        let mut analyzer = SemanticAnalyzer::new().unwrap();
        let code = r#"
impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}
"#;
        let result = analyzer.analyze_rust(code, "test.rs").unwrap();

        assert_eq!(result.trait_impls.len(), 1);
        assert_eq!(result.trait_impls[0].trait_name, "Display");
        assert_eq!(result.trait_impls[0].impl_type, "User");
    }
}
