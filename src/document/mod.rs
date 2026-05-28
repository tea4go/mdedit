mod buffer;
mod history;

pub use buffer::Buffer;
pub use history::{EditOp, History};

use std::path::PathBuf;

pub struct Document {
    pub path: Option<PathBuf>,
    pub buffer: Buffer,
    pub modified: bool,
    pub history: History,
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            buffer: Buffer::new(String::new()),
            modified: false,
            history: History::new(),
        }
    }

    pub fn from_file(path: PathBuf, content: String) -> Self {
        Self {
            path: Some(path),
            buffer: Buffer::new(content),
            modified: false,
            history: History::new(),
        }
    }

    pub fn content(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn apply_edit(&mut self, offset: usize, old_len: usize, new_text: &str) {
        let old_text = self.buffer.slice(offset, offset + old_len).to_string();
        let op = EditOp {
            offset,
            old_text: old_text.clone(),
            new_text: new_text.to_string(),
        };
        self.buffer.replace(offset, old_len, new_text);
        self.history.push(op);
        self.modified = true;
    }
}
