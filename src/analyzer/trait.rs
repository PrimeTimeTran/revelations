use std::path::PathBuf;

use crate::{analyzer::*, ir::*};

pub struct Analyze {
    pub target: AnalysisTarget,
    pub subject: Option<AnalyzeSubject>,
}

#[derive(Debug, Clone)]
pub enum AnalysisTarget {
    File(PathBuf),
    Workspace(PathBuf),
}

pub struct AnalyzeSubject {
    pub offset: usize,
}

#[derive(Debug)]
pub struct AnalyzerOptions {
    pub include_private: bool,
    pub include_tests: bool,
}

pub trait Analyzer {
    fn analyze(
        &self,
        request: Analyze,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError>;
}
