use eframe::egui;

use crate::app::EditMode;

#[derive(Clone, Copy, PartialEq)]
pub enum ToolbarAction {
    None,
    ToggleMode(EditMode),
    Undo,
    Redo,
    Heading,
    Bold,
    Italic,
    Strike,
    Link,
    UnorderedList,
    OrderedList,
    CheckList,
    Outdent,
    Indent,
    Quote,
    HorizontalRule,
    CodeBlock,
    InlineCode,
    Table,
    ToggleOutline,
    FontColor,
    BgColor,
    AttachFile,
    ToggleSearch,
}

pub struct ToolbarState {
    pub show_outline: bool,
}

struct ToolbarButton {
    label: &'static str,
    tooltip: &'static str,
    action: ToolbarAction,
    width: f32,
}

fn toolbar_buttons() -> Vec<ToolbarButton> {
    vec![
        // 模式切换
        ToolbarButton { label: "SV", tooltip: "源码编辑", action: ToolbarAction::ToggleMode(EditMode::Raw), width: 28.0 },
        ToolbarButton { label: "IR", tooltip: "即时渲染", action: ToolbarAction::ToggleMode(EditMode::Preview), width: 28.0 },
        // 分隔线（action=None 表示分隔线）
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 撤销/重做
        ToolbarButton { label: "\u{21A9}", tooltip: "撤销", action: ToolbarAction::Undo, width: 25.0 },
        ToolbarButton { label: "\u{21AA}", tooltip: "重做", action: ToolbarAction::Redo, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 格式
        ToolbarButton { label: "H", tooltip: "标题", action: ToolbarAction::Heading, width: 25.0 },
        ToolbarButton { label: "B", tooltip: "加粗", action: ToolbarAction::Bold, width: 25.0 },
        ToolbarButton { label: "I", tooltip: "斜体", action: ToolbarAction::Italic, width: 25.0 },
        ToolbarButton { label: "S", tooltip: "删除线", action: ToolbarAction::Strike, width: 25.0 },
        ToolbarButton { label: "\u{1F517}", tooltip: "链接", action: ToolbarAction::Link, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 列表
        ToolbarButton { label: "\u{2022}", tooltip: "无序列表", action: ToolbarAction::UnorderedList, width: 25.0 },
        ToolbarButton { label: "1.", tooltip: "有序列表", action: ToolbarAction::OrderedList, width: 25.0 },
        ToolbarButton { label: "\u{2611}", tooltip: "任务列表", action: ToolbarAction::CheckList, width: 25.0 },
        ToolbarButton { label: "\u{2190}", tooltip: "减少缩进", action: ToolbarAction::Outdent, width: 25.0 },
        ToolbarButton { label: "\u{2192}", tooltip: "增加缩进", action: ToolbarAction::Indent, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 块级
        ToolbarButton { label: ">", tooltip: "引用", action: ToolbarAction::Quote, width: 25.0 },
        ToolbarButton { label: "\u{2014}", tooltip: "分割线", action: ToolbarAction::HorizontalRule, width: 25.0 },
        ToolbarButton { label: "</>", tooltip: "代码块", action: ToolbarAction::CodeBlock, width: 30.0 },
        ToolbarButton { label: "`", tooltip: "行内代码", action: ToolbarAction::InlineCode, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 表格
        ToolbarButton { label: "\u{229E}", tooltip: "表格", action: ToolbarAction::Table, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 视图
        ToolbarButton { label: "\u{2630}", tooltip: "大纲", action: ToolbarAction::ToggleOutline, width: 25.0 },
        ToolbarButton { label: "A", tooltip: "字体颜色", action: ToolbarAction::FontColor, width: 25.0 },
        ToolbarButton { label: "\u{25A0}", tooltip: "背景颜色", action: ToolbarAction::BgColor, width: 25.0 },
        ToolbarButton { label: "|", tooltip: "", action: ToolbarAction::None, width: 8.0 },
        // 附件和搜索
        ToolbarButton { label: "\u{1F4CE}", tooltip: "插入附件", action: ToolbarAction::AttachFile, width: 25.0 },
        ToolbarButton { label: "\u{1F50D}", tooltip: "搜索 Ctrl+F", action: ToolbarAction::ToggleSearch, width: 25.0 },
    ]
}

pub fn render_toolbar(
    ui: &mut egui::Ui,
    state: &ToolbarState,
    current_mode: EditMode,
    icon_color: egui::Color32,
    hover_color: egui::Color32,
    separator_color: egui::Color32,
) -> ToolbarAction {
    let mut result = ToolbarAction::None;
    let buttons = toolbar_buttons();

    ui.horizontal(|ui| {
        for btn in &buttons {
            if btn.label == "|" {
                // 分隔线
                let rect = ui.allocate_exact_size(
                    egui::vec2(btn.width, 18.0),
                    egui::Sense::hover(),
                ).0;
                let y = rect.center().y;
                ui.painter().line_segment(
                    [egui::pos2(rect.center().x, y - 8.0), egui::pos2(rect.center().x, y + 8.0)],
                    egui::Stroke::new(1.0, separator_color),
                );
                continue;
            }

            // 模式按钮高亮
            let is_mode_active = match btn.action {
                ToolbarAction::ToggleMode(mode) => current_mode == mode,
                ToolbarAction::ToggleOutline => state.show_outline,
                _ => false,
            };

            let fg = if is_mode_active { hover_color } else { icon_color };

            let response = ui.add_sized(
                [btn.width, 25.0],
                egui::Button::new(
                    egui::RichText::new(btn.label).size(13.0).color(fg)
                ).frame(is_mode_active),
            );

            if response.clicked() && btn.action != ToolbarAction::None {
                result = btn.action;
            }

            if !btn.tooltip.is_empty() {
                response.on_hover_text(btn.tooltip);
            }
        }
    });

    result
}
