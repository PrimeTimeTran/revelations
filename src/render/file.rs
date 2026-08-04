use std::path::PathBuf;

#[derive(Debug)]
pub struct RenderedFile {
	pub path: PathBuf,
	pub header: String,
	pub body: String,
	pub is_empty: bool,
}
