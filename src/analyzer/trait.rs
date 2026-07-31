use std::path::PathBuf;

use crate::{analyzer::*, ir::*};

pub struct Analyze {
    pub target: AnalysisTarget,
    pub subject: Option<AnalyzeSubject>,
}

#[derive(Debug, Clone)]
pub struct AnalyzeSubject {
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub enum AnalysisTarget {
    File(PathBuf),
    Workspace(PathBuf),
}

#[derive(Clone, Debug)]
pub struct AnalyzerOptions {
    pub include_private: bool,
    pub include_tests: bool,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub mode: Option<String>,
}
impl Default for AnalyzerOptions {
   fn default()-> Self{
       Self {
           mode: Some("default".to_string()),
           line: Some(0),
           column: Some(0),
           include_tests: false,
           include_private: false,
       }
   } 
}

pub trait Analyzer {
    fn analyze(
        &self,
        request: Analyze,
        options: &AnalyzerOptions,
    ) -> Result<Workspace, AnalysisError>;
}
