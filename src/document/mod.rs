//! 文档模型模块 - 提供文档结构、文本缓冲区和编辑历史管理
//!
//! 核心组件：
//! - `Buffer`: 文本缓冲区，封装底层字符串操作
//! - `History`: 编辑历史，支持撤销/重做
//! - `Document`: 文档模型，组合缓冲区和历史记录

mod buffer;
mod history;

pub use buffer::Buffer;
pub use history::{EditOp, History};

use std::path::PathBuf;

/// 文档模型 - 表示一个打开的 Markdown 文档
///
/// 包含文件路径、文本缓冲区、修改标志和编辑历史。
/// 当文档未关联文件时（新建文档），path 为 None。
pub struct Document {
    /// 文件路径，None 表示未保存的新文档
    pub path: Option<PathBuf>,
    /// 文本缓冲区
    pub buffer: Buffer,
    /// 是否已修改（未保存）
    pub modified: bool,
    /// 编辑历史（撤销/重做）
    pub history: History,
}

impl Document {
    /// 创建空白文档
    pub fn new() -> Self {
        Self {
            path: None,
            buffer: Buffer::new(String::new()),
            modified: false,
            history: History::new(),
        }
    }

    /// 从文件路径和内容创建文档
    pub fn from_file(path: PathBuf, content: String) -> Self {
        Self {
            path: Some(path),
            buffer: Buffer::new(content),
            modified: false,
            history: History::new(),
        }
    }

    /// 获取文档内容
    pub fn content(&self) -> &str {
        self.buffer.as_str()
    }

    /// 应用一次编辑操作
    /// 在缓冲区指定位置替换文本，并记录到编辑历史
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
