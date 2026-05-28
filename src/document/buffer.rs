pub struct Buffer {
    text: String,
}

impl Buffer {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.text
    }

    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.text[start..end]
    }

    pub fn replace(&mut self, offset: usize, old_len: usize, new_text: &str) {
        self.text.replace_range(offset..offset + old_len, new_text);
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
}
