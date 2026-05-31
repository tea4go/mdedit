/// 文本缓冲区 - 文档内容的底层存储
///
/// 封装 String 类型，提供安全的文本操作接口，
/// 包括切片、替换等操作，供 Document 和 History 模块使用。
pub struct Buffer {
    /// 实际存储的文本内容
    text: String,
}

impl Buffer {
    /// 创建新的缓冲区
    pub fn new(text: String) -> Self {
        Self { text }
    }

    /// 获取缓冲区内容的不可变引用
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// 获取缓冲区内容的可变引用，用于直接修改文本
    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.text
    }

    /// 获取指定范围的文本切片
    /// 注意：调用方需确保 start..end 是有效的字符边界
    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.text[start..end]
    }

    /// 在指定偏移量处替换文本
    /// offset: 替换起始位置, old_len: 被替换的长度, new_text: 新文本
    pub fn replace(&mut self, offset: usize, old_len: usize, new_text: &str) {
        self.text.replace_range(offset..offset + old_len, new_text);
    }

    /// 返回文本的字节长度
    pub fn len(&self) -> usize {
        self.text.len()
    }
}
