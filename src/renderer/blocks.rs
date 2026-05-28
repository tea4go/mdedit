use egui;
use crate::theme::Theme;
use super::Block;

pub fn render_block(ui: &mut egui::Ui, block: &Block, theme: &Theme) {
    match block {
        Block::Heading { level, text } => {
            let size = theme.heading_sizes[(*level as usize - 1).min(5)];
            ui.label(egui::RichText::new(text).size(size).strong());
            if *level <= 2 {
                ui.separator();
            }
        }
        Block::Paragraph { text } => {
            render_inline_text(ui, text, theme);
        }
        Block::CodeBlock { code, .. } => {
            egui::Frame::default()
                .fill(theme.code_bg)
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(code.trim_end())
                            .monospace()
                            .color(theme.text_color),
                    );
                });
        }
        Block::Quote { text } => {
            ui.horizontal(|ui| {
                let rect = ui.available_rect_before_wrap();
                let bar_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(3.0, ui.spacing().interact_size.y),
                );
                ui.painter()
                    .rect_filled(bar_rect, 0.0, theme.quote_bar_color);
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new(text).italics().color(theme.muted_color),
                );
            });
        }
        Block::List { ordered, items } => {
            for (i, item) in items.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    let marker = if *ordered {
                        format!("{}.", i + 1)
                    } else {
                        "\u{2022}".to_string()
                    };
                    ui.label(&marker);
                    ui.label(item.as_str());
                });
            }
        }
        Block::Rule => {
            ui.separator();
        }
    }
}

fn render_inline_text(ui: &mut egui::Ui, text: &str, _theme: &Theme) {
    ui.label(text);
}
