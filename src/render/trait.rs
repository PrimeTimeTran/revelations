use crate::render::RenderedFile;
use std::{any::Any, path::Path};

pub trait FileRenderer: Any + 'static {
    fn render(&self, path: &Path, source: &str) -> RenderedFile;
}

impl dyn FileRenderer {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}
