use std::fmt;

#[derive(Debug)]
pub enum AnalysisError {
    Parse(String),
    UnsupportedLanguage(String),
    Io(String),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisError::Parse(msg) => {
                write!(f, "parse error: {}", msg)
            }
            AnalysisError::UnsupportedLanguage(lang) => {
                write!(f, "unsupported language: {}", lang)
            }
            AnalysisError::Io(msg) => {
                write!(f, "io error: {}", msg)
            }
        }
    }
}

impl std::error::Error for AnalysisError {}
