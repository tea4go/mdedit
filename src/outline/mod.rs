//! 大纲导航模块 - 从 Markdown 文档中提取标题大纲并管理导航状态
//!
//! 提供：
//! - 标题提取（跳过代码块内的标题）
//! - 大纲可见性控制（按级别展开/折叠）
//! - 编号生成（支持 1. / 1 / 1、 三种格式）

/// 大纲项 - 表示一个文档标题
pub struct OutlineItem {
    /// 标题级别 (1-6)
    pub level: u8,
    /// 标题文本
    pub title: String,
    /// 标题所在行号（0-based）
    pub line: usize,
}

/// 从 Markdown 内容中提取大纲项列表
/// 自动跳过代码块内的 # 开头行（避免误识别为标题）
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

/// 编号格式枚举
#[derive(Clone, Copy, PartialEq)]
pub enum NumberFormat {
    Dot,    // 1. / 1.1. - 带点号
    None,   // 1 / 1.1  - 无点号
    Comma,  // 1、 / 1.1、 - 中文顿号
}

/// 大纲导航状态
pub struct OutlineState {
    /// 展开到的最大级别（1-6）
    pub expand_level: u8,
    /// 是否显示编号
    pub show_numbers: bool,
    /// 编号格式
    pub number_format: NumberFormat,
}

impl OutlineState {
    /// 创建默认大纲状态（展开到6级，不显示编号）
    pub fn new() -> Self {
        Self {
            expand_level: 6,
            show_numbers: false,
            number_format: NumberFormat::Dot,
        }
    }

    /// 判断指定索引的大纲项是否应该可见
    /// 规则：级别 <= expand_level 的项始终可见；
    /// 更深层级的项如果其祖先已展开，也可见
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

    /// 生成大纲编号字符串（如 "1.2.3."）
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
