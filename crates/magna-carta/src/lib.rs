pub mod generator;

use crate::generator::Generator;
use crate::generator::jvm::KotlinJVMGenerator;
use crate::generator::native::KotlinNativeGenerator;
use clap::ValueEnum;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};

struct AnnotationInfo<'a> {
    node: tree_sitter::Node<'a>,
    name: Option<String>,
    value_args: Option<tree_sitter::Node<'a>>,
}

/// A group of manifests.
#[derive(Debug, Clone)]
pub struct ScriptManifest {
    items: Vec<ManifestItem>,
}

impl Default for ScriptManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptManifest {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: ManifestItem) {
        self.items.push(item);
    }

    pub fn items(&self) -> &[ManifestItem] {
        &self.items
    }
}

/// Represents a single script class. This struct contains all the necessary information to generate
/// a manifest
#[derive(Debug, Clone)]
pub struct ManifestItem {
    /// Fully qualified class name
    ///
    /// Example: `foo.bar.Enemy`
    fqcn: String,
    /// Simple name of the class
    ///
    /// Example: `Enemy`
    simple_name: String,
    /// Tags to identify the class
    ///
    /// Example: `["goomba", "shell"]`
    tags: Vec<String>,
    /// Path to the source file in reference to the project root
    ///
    /// Example: `/home/tirbofish/project2/src/commonMain/kotlin/foo/bar/Enemy.kt`
    file_path: PathBuf,
}

impl ManifestItem {
    /// Creates a new manifest item from an fqcn (fully qualified class name), simple name, tags
    /// and file_path.
    pub fn new(fqcn: String, simple_name: String, tags: Vec<String>, file_path: PathBuf) -> Self {
        Self {
            fqcn,
            simple_name,
            tags,
            file_path,
        }
    }

    pub fn fqcn(&self) -> &str {
        &self.fqcn
    }

    pub fn simple_name(&self) -> &str {
        &self.simple_name
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

/// Processer for Kotlin files.
pub struct KotlinProcessor {
    /// Tree-sitter parser.
    parser: Parser,
}

impl KotlinProcessor {
    /// Creates a new processor.
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_kotlin::language())?;
        Ok(Self { parser })
    }

    /// Processes the file for `@Runnable` annotations, and check if that
    /// class inherits the `System()` abstract class.
    pub fn process_file(
        &mut self,
        source_code: &str,
        file_path: PathBuf,
    ) -> anyhow::Result<Option<ManifestItem>> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse source code"))?;

        let root_node = tree.root_node();

        let package = self.extract_package(root_node, source_code)?;

        if let Some(class_info) = self.extract_class_info(root_node, source_code)? {
            let (class_name, tags) = class_info;

            let fqcn = if package.is_empty() {
                class_name.clone()
            } else {
                format!("{}.{}", package, class_name)
            };

            return Ok(Some(ManifestItem::new(fqcn, class_name, tags, file_path)));
        }

        if std::env::var("MAGNA_CARTA_DEBUG").is_ok() && source_code.contains("@Runnable") {
            let class_names = self.collect_class_names(root_node, source_code)?;
            eprintln!(
                "magna-carta: @Runnable found but no manifest item for {}. Classes seen: {:?}",
                file_path.display(),
                class_names
            );
            self.debug_dump_annotations(root_node, source_code, &file_path)?;
        }

        Ok(None)
    }

    fn extract_package(
        &self,
        root_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<String> {
        let query = Query::new(
            &tree_sitter_kotlin::language(),
            r#"
            (package_header
              (identifier) @package)
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, root_node, source.as_bytes());

        if let Some(match_) = matches.next()
            && let Some(capture) = match_.captures.first()
        {
            let package_node = capture.node;
            let package_text = package_node.utf8_text(source.as_bytes())?;

            return Ok(package_text.replace('\n', "").trim().to_string());
        }

        Ok(String::new())
    }

    fn extract_class_info(
        &self,
        root_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Option<(String, Vec<String>)>> {
        let query = Query::new(
            &tree_sitter_kotlin::language(),
            r#"
        ; Case 1: @Runnable (no parentheses)
        (class_declaration
          (modifiers
            (annotation
              (user_type
                (type_identifier) @annotation_name)
              (#eq? @annotation_name "Runnable")))
          (type_identifier) @class_name)

        ; Case 2: @Runnable(...) (with parentheses)
        (class_declaration
          (modifiers
            (annotation
              (constructor_invocation
                (user_type
                (type_identifier) @annotation_name2)
                    (value_arguments)? @value_args)
                (#eq? @annotation_name2 "Runnable")))
            (type_identifier) @class_name2)
        "#,
        )?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, source.as_bytes());

        let annotation_name_idx = query.capture_index_for_name("annotation_name");
        let annotation_name2_idx = query.capture_index_for_name("annotation_name2");
        let class_name_idx = query.capture_index_for_name("class_name");
        let class_name2_idx = query.capture_index_for_name("class_name2");
        let value_args_idx = query.capture_index_for_name("value_args");

        for match_ in matches {
            let mut class_name = String::new();
            let mut found_runnable = false;
            let mut value_args_node = None;

            for capture in match_.captures {
                let text = capture.node.utf8_text(source.as_bytes())?;

                // case 1 (no brackets)
                if let Some(idx) = annotation_name_idx
                    && capture.index == idx
                    && text == "Runnable"
                {
                    found_runnable = true;
                }

                // case 2 (with brackets)
                if let Some(idx) = annotation_name2_idx
                    && capture.index == idx
                    && text == "Runnable"
                {
                    found_runnable = true;
                }

                // class names
                if let Some(idx) = class_name_idx
                    && capture.index == idx
                {
                    class_name = text.to_string();
                }

                if let Some(idx) = class_name2_idx
                    && capture.index == idx
                {
                    class_name = text.to_string();
                }

                // case 2 value args
                if let Some(idx) = value_args_idx
                    && capture.index == idx
                {
                    value_args_node = Some(capture.node);
                }
            }

            if found_runnable && !class_name.is_empty() {
                let tags = if let Some(value_args) = value_args_node {
                    self.extract_tags_from_value_args(value_args, source)?
                } else {
                    Vec::new()
                };

                return Ok(Some((class_name, tags)));
            }
        }

        if let Some(result) = self.extract_class_info_fallback(root_node, source)? {
            return Ok(Some(result));
        }

        self.extract_class_info_loose(root_node, source)
    }

    fn extract_class_info_fallback(
        &self,
        root_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Option<(String, Vec<String>)>> {
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "class_declaration" {
                if let Some(class_name) = self.class_name_from_node(node, source)? {
                    if let Some(modifiers) = self.first_child_by_kind(node, "modifiers") {
                        let mut mod_cursor = modifiers.walk();
                        for modifier in modifiers.children(&mut mod_cursor) {
                            if modifier.kind() != "annotation" {
                                continue;
                            }

                            let (annotation_name, value_args) =
                                self.annotation_name_and_args(modifier, source)?;
                            if annotation_name.as_deref() == Some("Runnable") {
                                let tags = if let Some(args) = value_args {
                                    self.extract_tags_from_value_args(args, source)?
                                } else {
                                    let raw = modifier.utf8_text(source.as_bytes())?;
                                    self.extract_tags_from_raw_annotation(raw)
                                };

                                return Ok(Some((class_name, tags)));
                            }
                        }
                    }
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok(None)
    }

    fn class_name_from_node(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Option<String>> {
        let mut cursor = class_node.walk();
        for child in class_node.children(&mut cursor) {
            if child.kind() == "type_identifier" {
                let text = child.utf8_text(source.as_bytes())?;
                return Ok(Some(text.to_string()));
            }
        }

        Ok(None)
    }

    fn annotation_name_and_args<'a>(
        &self,
        annotation_node: tree_sitter::Node<'a>,
        source: &str,
    ) -> anyhow::Result<(Option<String>, Option<tree_sitter::Node<'a>>)> {
        let mut name: Option<String> = None;
        let mut value_args: Option<tree_sitter::Node> = None;
        let mut stack = vec![annotation_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "value_arguments" && value_args.is_none() {
                value_args = Some(node);
            }

            if node.kind() == "type_identifier" {
                let text = node.utf8_text(source.as_bytes())?;
                name = Some(text.to_string());
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok((name, value_args))
    }

    fn first_child_by_kind<'a>(
        &self,
        node: tree_sitter::Node<'a>,
        kind: &str,
    ) -> Option<tree_sitter::Node<'a>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return Some(child);
            }
        }

        None
    }

    fn extract_class_info_loose<'a>(
        &self,
        root_node: tree_sitter::Node<'a>,
        source: &str,
    ) -> anyhow::Result<Option<(String, Vec<String>)>> {
        let annotations = self.collect_annotation_nodes(root_node, source)?;
        let classes = self.collect_class_nodes(root_node);

        for annotation in annotations {
            if annotation.name.as_deref() != Some("Runnable") {
                continue;
            }

            let Some((class_node, _)) = classes
                .iter()
                .filter(|(node, _)| node.start_byte() > annotation.node.end_byte())
                .min_by_key(|(node, _)| node.start_byte().saturating_sub(annotation.node.end_byte()))
            else {
                continue;
            };

            if let Some(class_name) = self.class_name_from_node(*class_node, source)? {
                let tags = if let Some(args) = annotation.value_args {
                    self.extract_tags_from_value_args(args, source)?
                } else {
                    let raw = annotation.node.utf8_text(source.as_bytes())?;
                    let mut tags = self.extract_tags_from_raw_annotation(raw);
                    if tags.is_empty() {
                        tags = self.extract_tags_from_source_slice(
                            source,
                            annotation.node.start_byte(),
                            class_node.start_byte(),
                        );
                    }
                    tags
                };

                return Ok(Some((class_name, tags)));
            }
        }

        Ok(None)
    }

    fn collect_class_nodes<'a>(
        &self,
        root_node: tree_sitter::Node<'a>,
    ) -> Vec<(tree_sitter::Node<'a>, usize)> {
        let mut nodes = Vec::new();
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "class_declaration" {
                nodes.push((node, node.start_byte()));
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        nodes
    }

    fn collect_annotation_nodes<'a>(
        &self,
        root_node: tree_sitter::Node<'a>,
        source: &str,
    ) -> anyhow::Result<Vec<AnnotationInfo<'a>>> {
        let mut nodes = Vec::new();
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "annotation" {
                let (name, value_args) = self.annotation_name_and_args(node, source)?;
                nodes.push(AnnotationInfo {
                    node,
                    name,
                    value_args,
                });
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok(nodes)
    }

    fn extract_tags_from_raw_annotation(&self, raw: &str) -> Vec<String> {
        let mut tags = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut quote_char = '\0';

        for ch in raw.chars() {
            if !in_string {
                if ch == '"' || ch == '\'' {
                    in_string = true;
                    quote_char = ch;
                    current.clear();
                }
                continue;
            }

            if ch == quote_char {
                if !current.is_empty() {
                    tags.push(current.clone());
                }
                in_string = false;
                quote_char = '\0';
                current.clear();
                continue;
            }

            current.push(ch);
        }

        tags
    }

    fn extract_tags_from_source_slice(
        &self,
        source: &str,
        start: usize,
        end: usize,
    ) -> Vec<String> {
        let Some(slice) = source.get(start..end) else {
            return Vec::new();
        };

        self.extract_tags_from_raw_annotation(slice)
    }

    fn collect_class_names(
        &self,
        root_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut names = Vec::new();
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "class_declaration" {
                if let Some(name) = self.class_name_from_node(node, source)? {
                    names.push(name);
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok(names)
    }

    fn debug_dump_annotations(
        &self,
        root_node: tree_sitter::Node,
        source: &str,
        file_path: &Path,
    ) -> anyhow::Result<()> {
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if node.kind() == "class_declaration" {
                let class_name = self
                    .class_name_from_node(node, source)?
                    .unwrap_or_else(|| "<unknown>".to_string());
                let annotations = self.collect_annotation_debug(node, source)?;
                eprintln!(
                    "magna-carta: {} class {} annotations: {:?}",
                    file_path.display(),
                    class_name,
                    annotations
                );
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        Ok(())
    }

    fn collect_annotation_debug(
        &self,
        class_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut results = Vec::new();

        let Some(modifiers) = self.first_child_by_kind(class_node, "modifiers") else {
            return Ok(results);
        };

        let mut mod_cursor = modifiers.walk();
        for modifier in modifiers.children(&mut mod_cursor) {
            if modifier.kind() != "annotation" {
                continue;
            }

            let raw = modifier.utf8_text(source.as_bytes())?;
            let (annotation_name, _) = self.annotation_name_and_args(modifier, source)?;
            let name = annotation_name.unwrap_or_else(|| "<unknown>".to_string());
            results.push(format!("{} => {}", name, raw.trim()));
        }

        Ok(results)
    }

    fn extract_tags_from_value_args(
        &self,
        value_args_node: tree_sitter::Node,
        source: &str,
    ) -> anyhow::Result<Vec<String>> {
        let mut tags = Vec::new();

        let mut cursor = value_args_node.walk();
        for value_arg in value_args_node.children(&mut cursor) {
            if value_arg.kind() == "value_argument" {
                let mut arg_cursor = value_arg.walk();
                for child in value_arg.children(&mut arg_cursor) {
                    // Case 1: Direct string literal (vararg style)
                    if child.kind() == "string_literal" {
                        let text = child.utf8_text(source.as_bytes())?;
                        let clean_tag = text.trim_matches(|c| c == '"' || c == '\'').to_string();
                        if !clean_tag.is_empty() {
                            tags.push(clean_tag);
                        }
                    } else if child.kind() == "collection_literal" {
                        let mut collection_cursor = child.walk();
                        for collection_item in child.children(&mut collection_cursor) {
                            if collection_item.kind() == "string_literal" {
                                let text = collection_item.utf8_text(source.as_bytes())?;
                                let clean_tag =
                                    text.trim_matches(|c| c == '"' || c == '\'').to_string();
                                if !clean_tag.is_empty() {
                                    tags.push(clean_tag);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(tags)
    }
}

/// The target
#[derive(ValueEnum, Clone, Debug)]
pub enum Target {
    Jvm,
    Native,
}

/// Walks through all the input kotlin files and generates a manifest file for the target platform
/// at the directory provided by output.
///
/// Identically the same thing as the executable, except as a function in a Rust library instead.
///
/// # Target Behaviours
/// - [Target::Jvm] - Stores the manifest in `{output}/RunnableRegistry.kt`
/// - [Target::Native] - Stored the manifest in `{output}/ScriptManifest.kt`
pub fn parse(
    input: impl AsRef<Path>,
    target: Target,
    output: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let input = input.as_ref().to_path_buf();
    let output = output.as_ref().to_path_buf();

    let mut processor = KotlinProcessor::new()?;
    let mut manifest = ScriptManifest::new();

    if !input.exists() {
        return Err(anyhow::anyhow!(
            "Input directory does not exist: {:?}",
            input
        ));
    }

    visit_kotlin_files(&input, &mut processor, &mut manifest)?;

    let generated_content = match target {
        Target::Jvm => {
            let generator = KotlinJVMGenerator;
            generator.generate(&manifest)?
        }
        Target::Native => {
            let generator = KotlinNativeGenerator;
            generator.generate(&manifest)?
        }
    };

    fs::create_dir_all(&output)?;

    let filename = match target {
        Target::Jvm => "RunnableRegistry.kt",
        Target::Native => "ScriptManifest.kt",
    };
    let output_path = output.join(filename);
    fs::write(&output_path, generated_content)?;
    log::info!(
        "Generated {:?} manifest at: {}",
        target,
        output_path.display()
    );

    log::debug!("Found {} script classes", manifest.items().len());
    Ok(())
}

/// Helper function that visits all kotlin files in a directory recursively and processes them with
/// the [KotlinProcesser]
pub fn visit_kotlin_files(
    dir: &PathBuf,
    processor: &mut KotlinProcessor,
    manifest: &mut ScriptManifest,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_kotlin_files(&path, processor, manifest)?;
            } else if path.extension() == Some(std::ffi::OsStr::new("kt")) {
                let source_code = fs::read_to_string(&path)?;

                if let Some(item) = processor.process_file(&source_code, path.clone())? {
                    manifest.add_item(item);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_player_class() {
        let source = r#"
package com.dropbear

@Runnable(["player", "movement"])
class Player: System {
    override fun load(engine: DropbearEngine) {
        TODO("Not yet implemented")
    }
    override fun update(engine: DropbearEngine, deltaTime: Float) {

        TODO("Not yet implemented")

    }

    override fun destroy(engine: DropbearEngine) {

        TODO("Not yet implemented")

    }

}
"#;

        let mut processor = KotlinProcessor::new().unwrap();
        let result = processor
            .process_file(
                source,
                PathBuf::from("src/main/kotlin/com/dropbear/Player.kt"),
            )
            .unwrap();

        assert!(result.is_some());
        let item = result.unwrap();

        assert_eq!(item.fqcn(), "com.dropbear.Player");
        assert_eq!(item.simple_name(), "Player");
        assert_eq!(item.tags(), &["player", "movement"]);
        assert_eq!(
            item.file_path(),
            &PathBuf::from("src/main/kotlin/com/dropbear/Player.kt")
        );
    }
}
