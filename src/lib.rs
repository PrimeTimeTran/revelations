#![allow(warnings)]

// https://doc.rust-lang.org/rustc/lints/levels.html
// #![allow(warnings)] // Silences all warnings
// #![allow(unused_imports)] // Silences unused imports

// #![allow(dead_code)]
// #![allow(unused_must_use)]
// #![allow(unused_variables)]

// Add warnings
// #![warn(dead_code)]
// #![warn(unused_mut)]
// #![warn(unused_parens)]
// #![warn(unused_braces)]
// #![warn(unused_must_use)]
// #![warn(unused_assignments)]
// #![warn(unused_imports)]
// #![warn(unused_variables)]

pub mod _config;
pub mod _scope;
pub mod _semantic;
pub mod _workspace;
pub mod analyzer;
pub mod config;
pub mod context;
pub mod detector;
pub mod evaluator;
pub mod extract;
pub mod format;
pub mod ir;
pub mod mode;

pub mod parser;
pub mod render;
pub mod scanner;
pub mod todo;
pub mod ui;
pub mod writer;
