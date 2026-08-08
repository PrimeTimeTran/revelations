use crate::{
	analyzer::*,
	config::Config,
	detector::LanguageDetector,
	ir::Language,
	render::{FileRenderer, rust::RustFileRenderer, ts::TypeScriptFileRenderer},
	scanner::FileScanner,
	writer::{MarkdownWriter, OutputWriter},
};
use std::collections::HashMap;
use std::{fs, path::PathBuf};
pub struct AstContext {}
pub struct Evaluator {
	config: Config,
	scanner: FileScanner,
	renderers: HashMap<Language, Box<dyn FileRenderer>>,
	writer: Box<dyn OutputWriter>,
}
impl Default for Evaluator {
	fn default() -> Self {
		Self::new(Config::default())
	}
}
impl Evaluator {
	pub fn new(config: Config) -> Self {
		let root = config
			.analysis_root
			.canonicalize()
			.unwrap_or_else(|_| config.analysis_root.clone());
		let mut renderers: HashMap<Language, Box<dyn FileRenderer>> = HashMap::new();
		let rust_renderer = Box::new(RustFileRenderer {
			config: config.clone(),
		});
		renderers.insert(Language::Rust, rust_renderer.clone());
		renderers.insert(Language::Unknown, rust_renderer);
		renderers.insert(
			Language::TypeScript,
			Box::new(TypeScriptFileRenderer {
				config: config.clone(),
			}),
		);
		Self {
			config,
			scanner: FileScanner::new(root),
			renderers,
			writer: Box::new(MarkdownWriter),
		}
	}
	pub fn evaluate_fs(&mut self) {
		eprint!("Hi there evaluate");
		let files = self.scanner.scan();
		let mut rendered = vec![];
		eprint!("rendered {:?}", rendered);
		for file in files {
			let lang = LanguageDetector::detect(&file);
			let renderer = self.renderers.get(&lang).unwrap_or(
				self
					.renderers
					.get(&Language::Unknown)
					.expect("Missing Unknown renderer"),
			);
			let relative_path = file
				.strip_prefix(&self.config.analysis_root)
				.unwrap_or(&file);
			let src = fs::read_to_string(&file).unwrap_or_default();
			rendered.push(renderer.render(relative_path, &src));
		}
		let output = self.writer.write_file(rendered, &self.config);
		fs::write(&self.config.output_name, output).unwrap();
		// println!("Wrote {:?}", self.config.output_name);
	}
	pub fn evaluate_subject(&mut self, path: PathBuf) -> Result<Workspace, AnalysisError> {
		eprint!("evaluate_from_subject");
		// 1. File
		self.evaluate_file(path)
		// 2. Graph
		// let files = self.scanner.scan();
		// let rendered = vec![];
		// for file in files {
		//     self.evaluate_file(file);
		// }
		// let output = self.writer.write_file(rendered, &self.config);
	}
	pub fn evaluate_file(&mut self, file: PathBuf) -> Result<Workspace, AnalysisError> {
		let analyzer = RustAnalyzer;
		let request = Analyze {
			target: AnalysisTarget::File(file),
			subject: None,
		};
		analyzer.analyze(
			request,
			&AnalyzerOptions {
				line: Some(0),
				column: Some(0),
				mode: Some("default".to_string()),
				include_private: true,
				include_tests: true,
			},
		)
	}
}
