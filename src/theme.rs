use egui::Color32;

pub struct Theme {
    pub name: &'static str,
    pub base: BaseColors,
    pub heading: HeadingStyle,
    pub code: CodeStyle,
    pub quote: QuoteStyle,
    pub table: TableStyle,
    pub link: LinkStyle,
    pub list: ListStyle,
    pub rule: RuleStyle,
    pub font: FontStyle,
}

pub struct BaseColors {
    pub background: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub selection: Color32,
}

pub struct HeadingStyle {
    pub sizes: [f32; 6],
    pub colors: [Color32; 6],
    pub separator_colors: [Option<Color32>; 6],
    pub bold: bool,
}

pub struct CodeStyle {
    pub inline_bg: Color32,
    pub inline_text: Color32,
    pub inline_rounding: f32,
    pub block_bg: Color32,
    pub block_text: Color32,
    pub block_rounding: f32,
    pub block_padding: f32,
    pub block_border_color: Color32,
    pub block_style: String,
}

pub struct QuoteStyle {
    pub bar_color: Color32,
    pub bar_width: f32,
    pub text_color: Color32,
    pub bg_color: Color32,
    pub padding: f32,
}

pub struct TableStyle {
    pub header_bg: Color32,
    pub header_text: Color32,
    pub row_bg: Color32,
    pub alt_row_bg: Color32,
    pub border_color: Color32,
    pub cell_padding: f32,
    pub border_radius: f32,
}

pub struct LinkStyle {
    pub color: Color32,
    pub underline: bool,
}

pub struct ListStyle {
    pub marker_color: Color32,
    pub indent: f32,
    pub spacing: f32,
}

pub struct RuleStyle {
    pub color: Color32,
    pub thickness: f32,
}

pub struct FontStyle {
    pub base_size: f32,
    pub line_height: f32,
    pub monospace_size: f32,
}

impl Theme {
    pub fn light() -> Self {
        let text = Color32::from_rgb(44, 62, 80);
        Self {
            name: "light",
            base: BaseColors {
                background: Color32::from_rgb(249, 249, 245),
                text,
                muted: Color32::from_rgb(127, 140, 141),
                border: Color32::from_rgb(209, 213, 218),
                selection: Color32::from_rgb(173, 216, 230),
            },
            heading: HeadingStyle {
                sizes: [28.0, 24.0, 20.0, 18.0, 16.0, 14.0],
                colors: [text; 6],
                separator_colors: [
                    Some(Color32::from_rgb(41, 149, 217)),
                    Some(Color32::from_rgb(209, 213, 218)),
                    None, None, None, None,
                ],
                bold: true,
            },
            code: CodeStyle {
                inline_bg: Color32::from_rgb(238, 238, 238),
                inline_text: Color32::from_rgb(199, 37, 78),
                inline_rounding: 3.0,
                block_bg: Color32::from_rgb(238, 238, 238),
                block_text: Color32::from_rgb(51, 51, 51),
                block_rounding: 4.0,
                block_padding: 14.0,
                block_border_color: Color32::from_rgb(209, 213, 218),
                block_style: String::new(),
            },
            quote: QuoteStyle {
                bar_color: Color32::from_rgb(221, 221, 221),
                bar_width: 4.0,
                text_color: Color32::from_rgb(106, 115, 125),
                bg_color: Color32::from_rgb(248, 248, 248),
                padding: 12.0,
            },
            table: TableStyle {
                header_bg: Color32::from_rgb(240, 240, 240),
                header_text: text,
                row_bg: Color32::WHITE,
                alt_row_bg: Color32::from_rgb(246, 248, 250),
                border_color: Color32::from_rgb(209, 213, 218),
                cell_padding: 6.0,
                border_radius: 4.0,
            },
            link: LinkStyle {
                color: Color32::from_rgb(91, 164, 229),
                underline: true,
            },
            list: ListStyle {
                marker_color: Color32::from_rgb(41, 149, 217),
                indent: 20.0,
                spacing: 4.0,
            },
            rule: RuleStyle {
                color: Color32::from_rgb(221, 221, 221),
                thickness: 2.0,
            },
            font: FontStyle {
                base_size: 15.0,
                line_height: 1.6,
                monospace_size: 13.0,
            },
        }
    }
    pub fn dark() -> Self {
        let text = Color32::from_rgb(205, 214, 244);
        Self {
            name: "dark",
            base: BaseColors {
                background: Color32::from_rgb(30, 30, 46),
                text,
                muted: Color32::from_rgb(147, 153, 178),
                border: Color32::from_rgb(88, 91, 112),
                selection: Color32::from_rgb(69, 71, 90),
            },
            heading: HeadingStyle {
                sizes: [28.0, 24.0, 20.0, 18.0, 16.0, 14.0],
                colors: [text; 6],
                separator_colors: [
                    Some(Color32::from_rgb(137, 180, 250)),
                    Some(Color32::from_rgb(88, 91, 112)),
                    None, None, None, None,
                ],
                bold: true,
            },
            code: CodeStyle {
                inline_bg: Color32::from_rgb(49, 50, 68),
                inline_text: Color32::from_rgb(243, 139, 168),
                inline_rounding: 3.0,
                block_bg: Color32::from_rgb(49, 50, 68),
                block_text: Color32::from_rgb(205, 214, 244),
                block_rounding: 4.0,
                block_padding: 14.0,
                block_border_color: Color32::from_rgb(88, 91, 112),
                block_style: String::new(),
            },
            quote: QuoteStyle {
                bar_color: Color32::from_rgb(137, 180, 250),
                bar_width: 4.0,
                text_color: Color32::from_rgb(166, 173, 200),
                bg_color: Color32::from_rgb(36, 36, 54),
                padding: 12.0,
            },
            table: TableStyle {
                header_bg: Color32::from_rgb(49, 50, 68),
                header_text: text,
                row_bg: Color32::from_rgb(30, 30, 46),
                alt_row_bg: Color32::from_rgb(36, 36, 54),
                border_color: Color32::from_rgb(88, 91, 112),
                cell_padding: 6.0,
                border_radius: 4.0,
            },
            link: LinkStyle {
                color: Color32::from_rgb(137, 180, 250),
                underline: true,
            },
            list: ListStyle {
                marker_color: Color32::from_rgb(137, 180, 250),
                indent: 20.0,
                spacing: 4.0,
            },
            rule: RuleStyle {
                color: Color32::from_rgb(88, 91, 112),
                thickness: 2.0,
            },
            font: FontStyle {
                base_size: 15.0,
                line_height: 1.6,
                monospace_size: 13.0,
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

pub struct UiTheme {
    // === 应用基础 ===
    pub menu_bg: Color32,                  // appBgColor
    pub menu_text: Color32,                // appHeaderTextColor
    pub text_color: Color32,               // textColor
    pub text_active_color: Color32,        // textActiveColor
    pub border: Color32,                   // borderColor
    pub divider: Color32,                  // appDividerColor
    pub split_color: Color32,              // appSplitColor

    // === 侧边栏 ===
    pub sidebar_bg: Color32,               // appSiderBarBgColor
    pub sidebar_hover_bg: Color32,         // appSideHoverBgColor
    pub sidebar_active_text_color: Color32, // appSideTextActiveColor
    pub sidebar_text: Color32,             // appSideTextColor

    // === 状态栏 ===
    pub status_bar_bg: Color32,            // appStatusBarBgColor
    pub status_bar_text: Color32,          // appStatusBarTextColor
    pub status_bar_text_hover: Color32,    // appStatusBarTextHoverColor

    // === 左侧列表 ===
    pub left_list_bg: Color32,             // appLeftListBgColor
    pub left_list_bg_hover: Color32,       // appLeftListBgColorHover
    pub sidebar_active_bg: Color32,        // appLeftListBgColorActive
    pub sidebar_active_text: Color32,      // appLeftListTextColorActive
    pub search_title_bg: Color32,          // appSearchTitleBgColor

    // === 右侧内容区域 ===
    pub content_bg: Color32,               // appContentNoteBgColor
    pub content_term_bg: Color32,          // appContentTermBgColor
    pub content_chat_bg: Color32,          // appContentChatBgColor
    pub content_chat_divider: Color32,     // appContentChatDividerColor
    pub content_tran_bg: Color32,          // appContentTranBgColor

    // === 弹出层 ===
    pub dialog_bg: Color32,                // dialogBgColor
    pub dialog_border: Color32,            // dialogBorderColor
    pub dialog_divider: Color32,           // dialogDividerColor
    pub dialog_text: Color32,              // dialogTextColor
    pub dialog_text_active: Color32,       // dialogTextActiveColor

    // === 下拉菜单 ===
    pub drop_down_text: Color32,           // dropDownColor
    pub drop_down_bg: Color32,             // dropDownBgColor
    pub drop_down_active_text: Color32,    // dropDownActiveColor
    pub drop_down_active_bg: Color32,      // dropDownActiveBgColor

    // === AI Chat 消息 ===
    pub chat_send_bg: Color32,             // appContentChatSendBgColor
    pub chat_send_border: Color32,         // appContentChatSendBorderColor
    pub chat_reply_bg: Color32,            // appContentChatReplyBgColor
    pub chat_reply_border: Color32,        // appContentChatReplyBorderColor

    // === 输入框 ===
    pub input_bg: Color32,                 // inputContentBgColor
    pub input_border: Color32,             // inputContentBorderColor

    // === 表格 ===
    pub table_bg: Color32,                 // tableBgColor
    pub table_border: Color32,             // tableBorderColor
    pub table_header_bg: Color32,          // tableHeaderBgColor
    pub table_even_row_bg: Color32,        // tableEvenRowBgColor
}

pub struct ExtraTheme {
    // === 通用 ===
    pub tab_icon_color: Color32,
    pub active_color: Color32,
    pub search_icon_color: Color32,
    pub edit_disabled_color: Color32,

    // === 笔记 ===
    pub note_tab_header_border: Color32,
    pub note_toolbar_header_bg: Color32,
    pub note_search_num_bg_color: Color32,
    pub outline_hover_color: Color32,

    // === 表格 ===
    pub table_th_bg: Color32,
    pub table_td_bg: Color32,
    pub table_hover_color: Color32,

    // === 进度条 ===
    pub progress_free_bg: Color32,
    pub expand_table_bg: Color32,

    // === 信息面板 ===
    pub info_title_btn_color: Color32,
    pub info_title_btn_border_color: Color32,
    pub info_title_btn_hover_bg_color: Color32,
}
