use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use crate::{_config::AnalyzeConfig, analyzer::*, ir::*};
use quote::ToTokens;
use syn::{
	File,
	visit::{self, Visit},
};

#[derive(Clone, Debug)]
pub struct AnalysisResult {
	pub workspace: Workspace,
	pub metrics: AnalysisMetrics,
}

#[derive(Clone, Debug, Default)]
pub struct AnalysisMetrics {
	pub workspace: WorkspaceMetrics,
	pub packages: Vec<PackageMetrics>,
	pub modules: Vec<ModuleMetrics>,
	pub files: Vec<FileMetrics>,
}
impl AnalysisMetrics {
	pub fn new(workspace: WorkspaceMetrics) -> Self {
		Self {
			workspace,
			packages: Vec::new(),
			modules: Vec::new(),
			files: Vec::new(),
		}
	}
	pub fn with_packages(mut self, packages: Vec<PackageMetrics>) -> Self {
		self.packages = packages;
		self
	}
	pub fn with_modules(mut self, modules: Vec<ModuleMetrics>) -> Self {
		self.modules = modules;
		self
	}
	pub fn with_files(mut self, files: Vec<FileMetrics>) -> Self {
		self.files = files;
		self
	}
}
#[derive(Clone, Debug, Default)]
pub struct WorkspaceMetrics {
	pub files: usize,
	pub packages: usize,
	pub symbols: usize,
	pub functions: usize,
	pub types: usize,
	pub imports: usize,
	// pub config: AnalyzeConfig,
}

impl WorkspaceMetrics {
	pub fn new(workspace: &Workspace) -> Self {
		// let config = AnalyzeConfig::default();
		Self {
			// config,
			files: workspace.files.len(),
			packages: workspace.packages.len(),
			symbols: workspace.symbols.len(),
			functions: 0,
			types: 0,
			imports: 0,
		}
	}
}
#[derive(Clone, Debug, Default)]
pub struct PackageMetrics {
	pub name: String,
	pub symbols: usize,
	pub packages: usize,
	pub modules: usize,
	pub functions: usize,
	pub implementations: usize,
	pub types: usize,

	pub files: usize,
	pub imports: usize,
}
impl PackageMetrics {
	pub fn new(name: String) -> Self {
		Self {
			name,
			files: 0,
			symbols: 0,
			functions: 0,
			imports: 0,
			packages: 0,
			implementations: 0,
			modules: 0,
			types: 0,
		}
	}
}
#[derive(Clone, Debug, Default)]
pub struct ModuleMetrics {
	pub name: String,
	pub files: usize,
	pub symbols: usize,
	pub functions: usize,
	pub types: usize,
}
impl ModuleMetrics {
	pub fn new(name: String) -> Self {
		Self {
			name,
			files: 0,
			symbols: 0,
			functions: 0,
			types: 0,
		}
	}
}
#[derive(Clone, Debug, Default)]
pub struct FileMetrics {
	pub path: PathBuf,
	pub name: String,
	pub symbols: usize,
	pub functions: usize,
	pub imports: usize,
	pub types: usize,
}
impl FileMetrics {
	pub fn new(path: PathBuf) -> Self {
		let name = path
			.file_name()
			.unwrap_or_default()
			.to_string_lossy()
			.to_string();
		Self {
			path,
			name,
			imports: 0,
			symbols: 0,
			functions: 0,
			types: 0,
		}
	}
}
