pub struct Editor {
    pub cursor_line: usize,
    pub editing_block: Option<usize>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            cursor_line: 0,
            editing_block: None,
        }
    }

    pub fn line_to_block_index(&self, line: usize, block_lines: &[usize]) -> Option<usize> {
        for (i, &bl) in block_lines.iter().enumerate().rev() {
            if line >= bl {
                return Some(i);
            }
        }
        None
    }
}
