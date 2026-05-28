use egui;
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct TextBlock {
    pub start_line: usize,
    pub end_line: usize,
    pub source: String,
    pub kind: BlockKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    CodeBlock(String),
    Quote,
    List(bool),
    Rule,
    Empty,
}

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

pub fn render_rich_block(ui: &mut egui::Ui, block: &TextBlock, theme: &Theme) {
    match &block.kind {
        BlockKind::Heading(level) => {
            let text = block.source.trim_start_matches('#').trim_start();
            let size = theme.heading_sizes[(*level as usize - 1).min(5)];
            ui.label(egui::RichText::new(text).size(size).strong());
            if *level <= 2 {
                ui.separator();
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
                .fill(theme.code_bg)
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&code)
                            .monospace()
                            .color(theme.text_color),
                    );
                });
        }
        BlockKind::Quote => {
            let text: String = block.source.lines()
                .map(|l| l.strip_prefix('>').unwrap_or(l).trim_start())
                .collect::<Vec<_>>()
                .join("\n");
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(3.0, 16.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.quote_bar_color);
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(&text).italics().color(theme.muted_color),
                );
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
                    ui.add_space(16.0);
                    let marker = if *ordered {
                        format!("{}.", i + 1)
                    } else {
                        "\u{2022}".to_string()
                    };
                    ui.label(&marker);
                    ui.label(text);
                });
            }
        }
        BlockKind::Rule => {
            ui.separator();
        }
        BlockKind::Empty => {
            ui.add_space(8.0);
        }
    }
}

fn render_inline(ui: &mut egui::Ui, text: &str, _theme: &Theme) {
    let mut job = egui::text::LayoutJob::default();
    let mut chars = text.chars().peekable();
    let mut current = String::new();
    let default_fmt = egui::TextFormat::default();

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
            fmt.font_id = egui::FontId::proportional(14.0);
            fmt.font_id = egui::FontId::new(14.0, egui::FontFamily::Proportional);
            job.append(&bold_text, 0.0, egui::TextFormat {
                font_id: egui::FontId::proportional(14.0),
                color: default_fmt.color,
                ..Default::default()
            });
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
                font_id: egui::FontId::proportional(14.0),
                italics: true,
                color: default_fmt.color,
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
                font_id: egui::FontId::monospace(13.0),
                background: egui::Color32::from_gray(50),
                color: egui::Color32::from_gray(230),
                ..Default::default()
            });
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        job.append(&current, 0.0, default_fmt);
    }
    ui.label(job);
}
