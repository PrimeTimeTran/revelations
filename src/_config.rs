use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::analyzer::{AnalyzerOptions, NodeContext, ownership::{AstNodeKind, NodeClassification, ResolvedNode}};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeConfig {
    pub level: LogLevel,
    pub show: bool,
    pub show_in_status_bar: bool,
    pub show_in_output_channel: bool,
    pub show_in_notifications: bool,
    pub show_in_problems: bool,
    pub show_in_debug_console: bool,
    pub show_in_terminal: bool,
    pub show_in_editor: bool,
    pub show_in_editor_gutter: bool,
    pub show_in_editor_inline: bool,
    pub show_in_editor_hover: bool,
    pub show_in_editor_peek: bool,
    pub show_in_editor_code_lens: bool,
    pub show_in_editor_inlay_hints: bool,
    pub show_in_editor_decorations: bool,
    pub show_in_editor_minimap: bool,
    pub show_in_editor_overview_ruler: bool,
    pub show_in_editor_glyph_margin: bool,
    pub show_in_editor_line_numbers: bool,
    pub show_in_editor_whitespace: bool,
    pub show_in_editor_indent_guides: bool,
}

impl AnalyzeConfig {
    pub fn new(file_path: Option<&Path>, opts: &AnalyzerOptions) -> Self {
        let project_root = Self::find_project_root(file_path.unwrap());
        Self::load_with_overrides(file_path, opts)
    }

    /// The master builder: cascades and merges all layers from bottom to top.
    pub fn load_with_overrides(project_path: Option<&Path>, opts: &AnalyzerOptions) -> Self {
        // Step 1: Base layer (Defaults)
        let mut config = Self::default();
        // Step 2: Merge Global User Config (if it exists)
        if let Some(global_cfg) = Self::load_global() {
            config.merge(&global_cfg);
        }
        // Step 3: Merge Project-Specific Config (if it exists)
        if let Some(path) = project_path {
            if let Some(project_cfg) = Self::load_project(path) {
                config.merge(&project_cfg);
            }
        }
        // Step 4: Apply Runtime / CLI overrides (highest precedence)
        // config.apply_options(opts);
        config
    }

    /// Merges another config layer, overriding only explicitly set fields (using Options).
    pub fn merge(&mut self, other: &AnalyzerOptions) {
        // if let Some(level) = other.level {
        //     self.level = level;
        // }
        // if let Some(show) = other.show {
        //     self.show = show;
        // }
        // ... repeat for other fields
    }

    /// Applies immediate runtime flags passed via code or CLI.
    pub fn apply_options(&mut self, opts: &AnalyzerOptions) {
        // if let Some(ref level) = opts.level {
        //     self.set_level(level.clone());
        // }
        // if let Some(show) = opts.show {
        //     self.show = show;
        // }
        todo!("apply opts")
    }

    // --- File Loaders ---
    fn load_global() -> Option<AnalyzerOptions> {
        // Read from standard user config directory (e.g. ~/.config/loi/config.json)
        None
    }

    fn load_project(root: &Path) -> Option<AnalyzerOptions> {
        // Read from workspace root (e.g. root.join(".loi.json"))
        None
    }
    pub fn find_project_root(file_path: &Path) -> Option<PathBuf> {
        let mut current = file_path;
        if current.is_file() {
            current = current.parent()?;
        }
        while let Some(parent) = current.parent() {
            // Look for project markers (e.g., .git, .loi.json, Cargo.toml, etc.)
            if parent.join(".loi.json").exists() || parent.join(".git").exists() {
                return Some(parent.to_path_buf());
            }
            current = parent;
        }
        // Fallback: just use the file's immediate parent directory if no root marker is found
        file_path.parent().map(|p| p.to_path_buf())
    }
}
impl Default for AnalyzeConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            show: true,
            show_in_status_bar: true,
            show_in_output_channel: true,
            show_in_notifications: false,
            show_in_problems: true,
            show_in_debug_console: false,
            show_in_terminal: false,
            show_in_editor: true,
            show_in_editor_gutter: true,
            show_in_editor_inline: false,
            show_in_editor_hover: true,
            show_in_editor_peek: false,
            show_in_editor_code_lens: false,
            show_in_editor_inlay_hints: false,
            show_in_editor_decorations: true,
            show_in_editor_minimap: false,
            show_in_editor_overview_ruler: false,
            show_in_editor_glyph_margin: false,
            show_in_editor_line_numbers: false,
            show_in_editor_whitespace: false,
            show_in_editor_indent_guides: false,
        }
    }
}
impl AnalyzeConfig {
    pub fn set_level(&mut self, val: impl Into<LogLevel>) {
        self.level = val.into();
    }
}
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum LogLevel {
    Named(String),
    Numeric(u8),
}
impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Named("info".to_string())
    }
}
impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s {
            "0" | "debug" | "trace" => LogLevel::Numeric(0),
            "1" | "info" => LogLevel::Numeric(1),
            "2" | "warn" => LogLevel::Numeric(2),
            "3" | "error" => LogLevel::Numeric(3),
            "4" | "fatal" => LogLevel::Numeric(4),
            _ => LogLevel::Named(s.to_string()),
        }
    }
}
impl From<u8> for LogLevel {
    fn from(n: u8) -> Self {
        LogLevel::Numeric(n)
    }
}

pub struct Logger<'a> {
    pub opts: &'a AnalyzerOptions,
    pub cfg: &'a mut AnalyzeConfig,
}
impl<'a> Logger<'a> {
    pub fn new(opts: &'a AnalyzerOptions, cfg: &'a mut AnalyzeConfig) -> Self {
        Self { opts, cfg }
    }
    pub fn print(&self, title: &str, vals: Vals) {
            println!("{}:", title);
            
            // Print file path if present
            if let Some(path) = vals.file_path {
                println!("  file: {:?}", path);
            }
            if let Some(context) = vals.subject {
                println!("  context: {:?}", context);
            }
            if let Some(subject) = vals.subject {
                println!("  subject: {:?}", subject);
            }
            if let Some(ancestor) = vals.ancestor {
                println!("  ancestor: {:?}", ancestor);
            }
            if let Some(classification) = vals.classification {
                println!("  classification: {:?}", classification);
            }
            // Safely unwrap and check options
            if let Some(options) = vals.options {
                // Example: if options has a line field
                if let Some(line) = options.line {
                    println!("  line: {:?}", line);
                }
                
                // Example: checking column
                if options.column.is_some() {
                    println!("  column: {:?}", options.column);
                }
            }
        }
}
pub struct Vals<'a> {
    pub options: Option<&'a AnalyzerOptions>,
    pub file_path: Option<&'a PathBuf>,
    pub context: Option<&'a NodeContext>,
    pub classification: Option<&'a NodeClassification>,
    pub ancestor: Option<&'a AstNodeKind>,
    pub subject: Option<&'a ResolvedNode>,
}
impl<'a> Vals<'a> {
    // Base constructor starts with everything as None
    pub fn new() -> Self {
        Self {
            subject: None,
            ancestor: None,
            options: None,
            file_path: None,
            context: None,
            classification: None,
        }
    }
    // Chaining methods to easily attach fields
    pub fn file_path(mut self, path: &'a PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }
    pub fn options(mut self, opts: &'a AnalyzerOptions) -> Self {
        self.options = Some(opts);
        self
    }
    pub fn context(mut self, ctx: &'a NodeContext) -> Self {
        self.context = Some(ctx);
        self
    }
    pub fn classification(mut self, class: &'a NodeClassification) -> Self {
        self.classification = Some(class);
        self
    }
    pub fn ancestors(mut self, ancesstor: &'a AstNodeKind) -> Self {
        self.ancestor = Some(ancesstor);
        self
    }
    pub fn subject(mut self, subject: &'a ResolvedNode) -> Self {
        self.subject = Some(subject);
        self
    }
}