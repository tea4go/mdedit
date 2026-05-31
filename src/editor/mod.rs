//! 编辑器模块 - Markdown 文本块分割与渲染
//!
//! 负责：
//! - 将 Markdown 文本分割为语义块（标题、段落、代码块、引用、列表等）
//! - 渲染富文本块（带样式的预览模式）
//! - 行内格式解析（加粗、斜体、行内代码、链接）

use egui;
use crate::theme::Theme;

/// 文本块 - 表示一个 Markdown 语义单元
#[derive(Debug, Clone)]
pub struct TextBlock {
    /// 起始行号（0-based）
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 原始 Markdown 文本
    pub source: String,
    /// 块类型
    pub kind: BlockKind,
}

/// 块类型枚举 - Markdown 的各种语义块
#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    /// 标题，参数为级别 (1-6)
    Heading(u8),
    /// 普通段落
    Paragraph,
    /// 代码块，参数为语言标识
    CodeBlock(String),
    /// 引用块
    Quote,
    /// 列表，参数为是否有序列表
    List(bool),
    /// 表格
    Table,
    /// 水平分割线
    Rule,
    /// 空行
    Empty,
}

/// 将 Markdown 文本按语义分割为文本块列表
/// 解析规则：标题→代码块→引用→列表→表格→分割线→段落→空行
pub fn split_blocks(content: &str) -> Vec<TextBlock> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() {
            blocks.push(TextBlock {
                start_line: i,
                end_line: i,
                source: line.to_string(),
                kind: BlockKind::Empty,
            });
            i += 1;
        } else if line.starts_with('#') {
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            blocks.push(TextBlock {
                start_line: i,
                end_line: i,
                source: line.to_string(),
                kind: BlockKind::Heading(level.min(6)),
            });
            i += 1;
        } else if line.starts_with("```") {
            let lang = line[3..].trim().to_string();
            let start = i;
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                i += 1;
            }
            let end = if i < lines.len() { i } else { i - 1 };
            let source = lines[start..=end].join("\n");
            blocks.push(TextBlock {
                start_line: start,
                end_line: end,
                source,
                kind: BlockKind::CodeBlock(lang),
            });
            i = end + 1;
        } else if line.starts_with('>') {
            let start = i;
            while i < lines.len() && lines[i].starts_with('>') {
                i += 1;
            }
            let source = lines[start..i].join("\n");
            blocks.push(TextBlock {
                start_line: start,
                end_line: i - 1,
                source,
                kind: BlockKind::Quote,
            });
        } else if line.starts_with("- ")
            || line.starts_with("* ")
            || line.starts_with("+ ")
            || (line.len() > 2 && line.chars().next().unwrap().is_ascii_digit()
                && line.contains(". "))
        {
            let ordered = line.chars().next().unwrap().is_ascii_digit();
            let start = i;
            while i < lines.len() {
                let l = lines[i];
                let is_list_item = l.starts_with("- ")
                    || l.starts_with("* ")
                    || l.starts_with("+ ")
                    || (l.len() > 2 && l.chars().next().unwrap().is_ascii_digit()
                        && l.contains(". "))
                    || l.starts_with("  ");
                if l.trim().is_empty() || !is_list_item {
                    break;
                }
                i += 1;
            }
            let source = lines[start..i].join("\n");
            blocks.push(TextBlock {
                start_line: start,
                end_line: i - 1,
                source,
                kind: BlockKind::List(ordered),
            });
        } else if line.contains('|') && i + 1 < lines.len()
            && is_table_separator(lines[i + 1])
        {
            let start = i;
            while i < lines.len() && lines[i].contains('|') {
                i += 1;
            }
            let source = lines[start..i].join("\n");
            blocks.push(TextBlock {
                start_line: start,
                end_line: i - 1,
                source,
                kind: BlockKind::Table,
            });
        } else if line == "---" || line == "***" || line == "___" {
            blocks.push(TextBlock {
                start_line: i,
                end_line: i,
                source: line.to_string(),
                kind: BlockKind::Rule,
            });
            i += 1;
        } else {
            let start = i;
            while i < lines.len()
                && !lines[i].trim().is_empty()
                && !lines[i].starts_with('#')
                && !lines[i].starts_with("```")
                && !lines[i].starts_with('>')
                && lines[i] != "---"
                && lines[i] != "***"
            {
                i += 1;
            }
            let source = lines[start..i].join("\n");
            blocks.push(TextBlock {
                start_line: start,
                end_line: i - 1,
                source,
                kind: BlockKind::Paragraph,
            });
        }
    }
    blocks
}

/// 判断是否为 Markdown 表格分隔行（如 |---|---|）
fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('|') {
        return false;
    }
    trimmed.chars().all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
}

/// 渲染富文本块 - 根据块类型应用对应主题样式
pub fn render_rich_block(ui: &mut egui::Ui, block: &TextBlock, theme: &Theme) {
    match &block.kind {
        BlockKind::Heading(level) => {
            let idx = (*level as usize - 1).min(5);
            let text = block.source.trim_start_matches('#').trim_start();
            let size = theme.heading.sizes[idx];
            let color = theme.heading.colors[idx];
            let mut rt = egui::RichText::new(text).size(size).color(color);
            if theme.heading.bold {
                rt = rt.strong();
            }
            ui.label(rt);
            if let Some(sep_color) = theme.heading.separator_colors[idx] {
                let rect = ui.available_rect_before_wrap();
                let y = rect.min.y;
                let stroke = egui::Stroke::new(
                    if idx == 0 { 2.0 } else { 1.0 },
                    sep_color,
                );
                ui.painter().hline(rect.x_range(), y, stroke);
                ui.add_space(4.0);
            }
        }
        BlockKind::Paragraph => {
            render_inline(ui, &block.source, theme);
        }
        BlockKind::CodeBlock(_lang) => {
            let code = block.source.lines()
                .skip(1)
                .take_while(|l| !l.starts_with("```"))
                .collect::<Vec<_>>()
                .join("\n");
            egui::Frame::default()
                .fill(theme.code.block_bg)
                .rounding(theme.code.block_rounding)
                .inner_margin(theme.code.block_padding)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&code)
                            .monospace()
                            .size(theme.font.monospace_size)
                            .color(theme.code.block_text),
                    );
                });
        }
        BlockKind::Quote => {
            let text: String = block.source.lines()
                .map(|l| l.strip_prefix('>').unwrap_or(l).trim_start())
                .collect::<Vec<_>>()
                .join("\n");
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(theme.quote.bar_width, ui.available_height().max(18.0)),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.quote.bar_color);
                ui.add_space(theme.quote.padding);
                egui::Frame::default()
                    .fill(theme.quote.bg_color)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&text)
                                .italics()
                                .color(theme.quote.text_color),
                        );
                    });
            });
        }
        BlockKind::List(ordered) => {
            for (i, line) in block.source.lines().enumerate() {
                let text = if *ordered {
                    line.splitn(2, ". ").nth(1).unwrap_or(line)
                } else {
                    line.trim_start_matches(|c| c == '-' || c == '*' || c == '+')
                        .trim_start()
                };
                ui.horizontal(|ui| {
                    ui.add_space(theme.list.indent);
                    let marker = if *ordered {
                        format!("{}.", i + 1)
                    } else {
                        "\u{2022}".to_string()
                    };
                    ui.label(
                        egui::RichText::new(&marker).color(theme.list.marker_color),
                    );
                    render_inline(ui, text, theme);
                });
                ui.add_space(theme.list.spacing);
            }
        }
        BlockKind::Table => {
            let rows: Vec<Vec<&str>> = block.source.lines()
                .filter(|l| !is_table_separator(l))
                .map(|l| {
                    l.trim().trim_matches('|')
                        .split('|')
                        .map(|cell| cell.trim())
                        .collect()
                })
                .collect();

            if !rows.is_empty() {
                let col_count = rows[0].len();
                egui::Grid::new(format!("table_{}", block.start_line))
                    .min_col_width(60.0)
                    .spacing(egui::vec2(theme.table.cell_padding, 2.0))
                    .show(ui, |ui| {
                        for (row_idx, row) in rows.iter().enumerate() {
                            for col_idx in 0..col_count {
                                let cell = row.get(col_idx).unwrap_or(&"");
                                if row_idx == 0 {
                                    ui.label(
                                        egui::RichText::new(*cell)
                                            .strong()
                                            .color(theme.table.header_text),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new(*cell)
                                            .color(theme.base.text),
                                    );
                                }
                            }
                            ui.end_row();
                        }
                    });
            }
        }
        BlockKind::Rule => {
            let rect = ui.available_rect_before_wrap();
            let y = rect.center().y;
            ui.painter().hline(
                rect.x_range(),
                y,
                egui::Stroke::new(theme.rule.thickness, theme.rule.color),
            );
            ui.add_space(theme.rule.thickness + 8.0);
        }
        BlockKind::Empty => {
            ui.add_space(8.0);
        }
    }
}

/// 行内格式渲染 - 解析加粗、斜体、行内代码、链接等行内 Markdown 语法
fn render_inline(ui: &mut egui::Ui, text: &str, theme: &Theme) {
    let mut job = egui::text::LayoutJob::default();
    let mut chars = text.chars().peekable();
    let mut current = String::new();
    let default_fmt = egui::TextFormat {
        font_id: egui::FontId::proportional(theme.font.base_size),
        color: theme.base.text,
        ..Default::default()
    };

    while let Some(ch) = chars.next() {
        if ch == '*' && chars.peek() == Some(&'*') {
            chars.next();
            if !current.is_empty() {
                job.append(&current, 0.0, default_fmt.clone());
                current.clear();
            }
            let mut bold_text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '*' {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    }
                    bold_text.push(c);
                } else {
                    bold_text.push(chars.next().unwrap());
                }
            }
            let mut fmt = default_fmt.clone();
            fmt.font_id = egui::FontId::proportional(theme.font.base_size);
            job.append(&bold_text, 0.0, fmt);
        } else if ch == '*' {
            if !current.is_empty() {
                job.append(&current, 0.0, default_fmt.clone());
                current.clear();
            }
            let mut italic_text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '*' {
                    chars.next();
                    break;
                }
                italic_text.push(chars.next().unwrap());
            }
            job.append(&italic_text, 0.0, egui::TextFormat {
                font_id: egui::FontId::proportional(theme.font.base_size),
                italics: true,
                color: theme.base.text,
                ..Default::default()
            });
        } else if ch == '`' {
            if !current.is_empty() {
                job.append(&current, 0.0, default_fmt.clone());
                current.clear();
            }
            let mut code_text = String::new();
            while let Some(&c) = chars.peek() {
                if c == '`' {
                    chars.next();
                    break;
                }
                code_text.push(chars.next().unwrap());
            }
            job.append(&code_text, 0.0, egui::TextFormat {
                font_id: egui::FontId::monospace(theme.font.monospace_size),
                background: theme.code.inline_bg,
                color: theme.code.inline_text,
                ..Default::default()
            });
        } else if ch == '[' {
            let mut link_text = String::new();
            let mut found_link = false;
            while let Some(&c) = chars.peek() {
                if c == ']' {
                    chars.next();
                    if chars.peek() == Some(&'(') {
                        chars.next();
                        while let Some(&u) = chars.peek() {
                            if u == ')' { chars.next(); break; }
                            chars.next();
                        }
                        found_link = true;
                    }
                    break;
                }
                link_text.push(chars.next().unwrap());
            }
            if found_link {
                if !current.is_empty() {
                    job.append(&current, 0.0, default_fmt.clone());
                    current.clear();
                }
                job.append(&link_text, 0.0, egui::TextFormat {
                    font_id: egui::FontId::proportional(theme.font.base_size),
                    color: theme.link.color,
                    underline: egui::Stroke::new(1.0, theme.link.color),
                    ..Default::default()
                });
            } else {
                current.push('[');
                current.push_str(&link_text);
                current.push(']');
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        job.append(&current, 0.0, default_fmt);
    }
    ui.label(job);
}
