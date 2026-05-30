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
    pub menu_bg: Color32,
    pub menu_text: Color32,
    pub sidebar_bg: Color32,
    pub sidebar_text: Color32,
    pub sidebar_hover_bg: Color32,
    pub sidebar_active_bg: Color32,
    pub sidebar_active_text: Color32,
    pub content_bg: Color32,
    pub border: Color32,
    pub divider: Color32,
}
