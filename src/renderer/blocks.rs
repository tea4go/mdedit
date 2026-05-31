//! 块级元素渲染 - 将解析后的 Block 渲染为 egui 界面元素

use egui;
use crate::theme::Theme;
use super::Block;

/// 渲染单个 Markdown 块级元素
/// 根据块类型（标题、段落、代码块、引用、列表、分割线）应用对应样式
pub fn render_block(ui: &mut egui::Ui, block: &Block, theme: &Theme) {
    match block {
        Block::Heading { level, text } => {
            let idx = (*level as usize - 1).min(5);
            let size = theme.heading.sizes[idx];
            let color = theme.heading.colors[idx];
            let mut rt = egui::RichText::new(text).size(size).color(color);
            if theme.heading.bold {
                rt = rt.strong();
            }
            ui.label(rt);
            if let Some(sep_color) = theme.heading.separator_colors[idx] {
                let rect = ui.available_rect_before_wrap();
                let stroke = egui::Stroke::new(
                    if idx == 0 { 2.0 } else { 1.0 },
                    sep_color,
                );
                ui.painter().hline(rect.x_range(), rect.min.y, stroke);
                ui.add_space(4.0);
            }
        }
        Block::Paragraph { text } => {
            render_inline_text(ui, text, theme);
        }
        Block::CodeBlock { code, .. } => {
            egui::Frame::default()
                .fill(theme.code.block_bg)
                .rounding(theme.code.block_rounding)
                .inner_margin(theme.code.block_padding)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(code.trim_end())
                            .monospace()
                            .size(theme.font.monospace_size)
                            .color(theme.code.block_text),
                    );
                });
        }
        Block::Quote { text } => {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(theme.quote.bar_width, 18.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 0.0, theme.quote.bar_color);
                ui.add_space(theme.quote.padding);
                ui.label(
                    egui::RichText::new(text)
                        .italics()
                        .color(theme.quote.text_color),
                );
            });
        }
        Block::List { ordered, items } => {
            for (i, item) in items.iter().enumerate() {
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
                    ui.label(item.as_str());
                });
            }
        }
        Block::Rule => {
            let rect = ui.available_rect_before_wrap();
            ui.painter().hline(
                rect.x_range(),
                rect.center().y,
                egui::Stroke::new(theme.rule.thickness, theme.rule.color),
            );
            ui.add_space(theme.rule.thickness + 8.0);
        }
    }
}

/// 渲染行内纯文本（简单版本，不含格式解析）
fn render_inline_text(ui: &mut egui::Ui, text: &str, theme: &Theme) {
    ui.label(egui::RichText::new(text).color(theme.base.text));
}
