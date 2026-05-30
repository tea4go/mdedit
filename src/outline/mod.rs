pub struct OutlineItem {
    pub level: u8,
    pub title: String,
    pub line: usize,
}

pub fn extract_outline(content: &str) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    let mut in_code_block = false;
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level <= 6 {
                let title = trimmed[level..].trim_start().to_string();
                if !title.is_empty() {
                    items.push(OutlineItem {
                        level: level as u8,
                        title,
                        line: line_idx,
                    });
                }
            }
        }
    }
    items
}

#[derive(Clone, Copy, PartialEq)]
pub enum NumberFormat {
    Dot,    // 1. / 1.1.
    None,   // 1 / 1.1
    Comma,  // 1、 / 1.1、
}

pub struct OutlineState {
    pub expand_level: u8,
    pub show_numbers: bool,
    pub number_format: NumberFormat,
}

impl OutlineState {
    pub fn new() -> Self {
        Self {
            expand_level: 6,
            show_numbers: false,
            number_format: NumberFormat::Dot,
        }
    }

    pub fn is_visible(&self, items: &[OutlineItem], index: usize) -> bool {
        let item_level = items[index].level;
        if item_level <= self.expand_level {
            return true;
        }
        // 如果有展开到更深级别的祖先，也可见
        if index > 0 {
            for i in (0..index).rev() {
                if items[i].level < item_level {
                    if items[i].level <= self.expand_level {
                        return true;
                    }
                    break;
                }
            }
        }
        false
    }

    pub fn generate_number(&self, items: &[OutlineItem], index: usize) -> String {
        if !self.show_numbers { return String::new(); }
        let mut counters = [0u32; 6];
        for (i, item) in items.iter().enumerate() {
            if i > index { break; }
            let lvl = (item.level - 1) as usize;
            counters[lvl] += 1;
            // 重置子级计数器
            for j in (lvl + 1)..6 {
                counters[j] = 0;
            }
        }
        // 构建编号字符串
        let item_level = items[index].level;
        let mut parts = Vec::new();
        for i in 0..item_level as usize {
            if counters[i] > 0 {
                parts.push(counters[i].to_string());
            }
        }
        if parts.is_empty() { return String::new(); }
        let num = parts.join(".");
        match self.number_format {
            NumberFormat::Dot => format!("{}.", num),
            NumberFormat::None => num,
            NumberFormat::Comma => format!("{}、", num),
        }
    }

    /// 返回大纲项的缩进量（px）
    pub fn indent(&self, level: u8, font_size: f32) -> f32 {
        if level <= 1 { return 0.0; }
        (level - 1) as f32 * (font_size + 6.0)
    }

    /// 返回大纲项的字号，统一使用 base_size 保证左对齐整齐
    pub fn font_size(&self, _level: u8, base_size: f32) -> f32 {
        base_size
    }

    /// 返回大纲项的字重
    pub fn font_weight(&self, level: u8) -> u16 {
        match level {
            1 => 700,
            2 => 600,
            _ => 400,
        }
    }
}
