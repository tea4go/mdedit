//! 应用主模块 - MdEditApp 核心逻辑
//!
//! 整合所有子模块，管理应用状态、事件处理和界面布局。
//! 主要职责：
//! - 初始化主题、字体、编辑模式
//! - 处理快捷键和工具栏操作
//! - 管理文档生命周期（新建、打开、保存）
//! - 渲染三栏布局（Ribbon + 左面板 + 编辑区 + 状态栏）

use std::path::{Path, PathBuf};

use eframe::egui;
use crate::config::AppConfig;
use crate::css_loader;

/// 递归复制目录（用于文件树粘贴操作）
fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
use crate::document::Document;
use crate::editor::{self, TextBlock};
use crate::outline::{self, OutlineItem, OutlineState, NumberFormat};
use crate::theme::{Theme, UiTheme, ExtraTheme, CodeStyle, HeadingStyle, QuoteStyle, TableStyle, LinkStyle};
use crate::toolbar::{self, ToolbarAction, ToolbarState};
use crate::file_tree::{self, FileTreeState, FileTreeAction};
use crate::search::{self, SearchBarState, SearchTreeState};
use crate::auto_save::AutoSaveState;

const CSS_THEME_DIR: &str =
    r"C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme";

/// 主题模式（浅色/深色/跟随系统）
#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

/// 编辑模式（原始编辑/即时渲染预览）
#[derive(Clone, Copy, PartialEq)]
pub enum EditMode {
    Raw,
    Preview,
}

/// 左面板标签页类型
#[derive(Clone, Copy, PartialEq)]
pub enum LeftPanelTab {
    Tree,     // 文件目录
    Outline,  // 大纲导航
    Search,   // 全文搜索
}

/// 笔记字体配置（从 WhaleTerm preferences 加载）
struct NoteFontConfig {
    family: Vec<String>,
    size: f32,
    bold: bool,
}

/// UI 字体配置（菜单栏等 UI 元素使用）
struct ConfigFontConfig {
    family: Vec<String>,
    size: f32,
    bold: bool,
}

/// 在系统字体目录和用户字体目录中查找字体文件数据
fn find_font_data(font_name: &str) -> Option<Vec<u8>> {
    // Windows 系统字体目录
    let font_dir = std::path::Path::new(r"C:\Windows\Fonts");
    // 用户字体目录
    let user_font_dir = std::path::Path::new(&std::env::var("LOCALAPPDATA").unwrap_or_default())
        .join("Fonts");

    // 字体名称到文件名的映射（常见中文字体）
    let name_map: &[(&str, &str)] = &[
        ("LXGW WenKai", "LXGWWenKai"),
        ("LXGW WenKai Mono", "LXGWWenKaiMono"),
        ("Microsoft YaHei", "msyh"),
        ("Microsoft YaHei Mono", "msyh"),
        ("SimSun", "simsun"),
        ("SimHei", "simhei"),
        ("PingFang SC", "PingFang"),
        ("Noto Sans CJK SC", "NotoSansCJKsc"),
    ];

    // 先在映射表中查找
    let file_prefix = name_map.iter()
        .find(|(name, _)| font_name.eq_ignore_ascii_case(name))
        .map(|(_, prefix)| prefix.to_string())
        .unwrap_or_else(|| font_name.replace(' ', ""));

    let extensions = ["ttf", "ttc", "otf"];

    // 搜索系统字体目录和用户字体目录
    let user_font_dir = user_font_dir.as_path();
    for dir in [&font_dir, user_font_dir] {
        for ext in &extensions {
            // 尝试精确匹配
            let filename = format!("{}.{}", file_prefix, ext);
            let path = dir.join(&filename);
            if path.exists() {
                return std::fs::read(&path).ok();
            }
        }

        // 遍历目录寻找包含字体名的文件
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let name_lower = name.to_lowercase();
                let prefix_lower = file_prefix.to_lowercase().replace(' ', "");
                if name_lower.starts_with(&prefix_lower)
                    && (name_lower.ends_with(".ttf") || name_lower.ends_with(".ttc") || name_lower.ends_with(".otf"))
                {
                    return std::fs::read(entry.path()).ok();
                }
            }
        }
    }
    None
}

/// 从 WhaleTerm preferences.json 加载笔记字体配置
fn load_note_font_config() -> NoteFontConfig {
    let path = PathBuf::from(r"C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return NoteFontConfig {
            family: Vec::new(),
            size: 15.0,
            bold: false,
        },
    };
    let root: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return NoteFontConfig {
            family: Vec::new(),
            size: 15.0,
            bold: false,
        },
    };
    let note = match root.get("note") {
        Some(n) => n,
        None => return NoteFontConfig {
            family: Vec::new(),
            size: 15.0,
            bold: false,
        },
    };
    let family = note.get("fontFamily")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let size = note.get("fontSize").and_then(|v| v.as_f64()).unwrap_or(15.0) as f32;
    let bold = note.get("fontBold").and_then(|v| v.as_str()).map(|s| s == "bold").unwrap_or(false);
    NoteFontConfig { family, size, bold }
}

/// 从 WhaleTerm preferences.json 加载 UI 字体配置
fn load_config_font_config() -> ConfigFontConfig {
    let path = PathBuf::from(r"C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return ConfigFontConfig {
            family: Vec::new(),
            size: 12.0,
            bold: false,
        },
    };
    let root: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return ConfigFontConfig {
            family: Vec::new(),
            size: 12.0,
            bold: false,
        },
    };
    let config = match root.get("config") {
        Some(n) => n,
        None => return ConfigFontConfig {
            family: Vec::new(),
            size: 12.0,
            bold: false,
        },
    };
    let family = config.get("defaultFontFamily")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let size = config.get("defaultFontSize").and_then(|v| v.as_f64()).unwrap_or(12.0) as f32;
    let bold = config.get("defaultFontBold").and_then(|v| v.as_str()).map(|s| s == "bold").unwrap_or(false);
    ConfigFontConfig { family, size, bold }
}

/// 解析十六进制颜色字符串为 egui Color32
/// 支持 #RGB、#RRGGBB、#RRGGBBAA 格式
fn parse_hex_color(val: &str) -> Option<egui::Color32> {
    let hex = val.trim().strip_prefix('#')?;
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(egui::Color32::from_rgb(r, g, b))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(egui::Color32::from_rgb(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(egui::Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

/// 读取 WhaleTerm preferences.json 文件
fn read_preferences() -> Option<serde_json::Value> {
    let path = PathBuf::from(r"C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json");
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 从 WhaleTerm preferences.json 加载笔记内容主题
fn load_note_theme(mode: ThemeMode) -> Option<Theme> {
    let root = read_preferences()?;
    let key = match mode {
        ThemeMode::Light | ThemeMode::Auto => "noteThemeLight",
        ThemeMode::Dark => "noteThemeDark",
    };
    let nt = root.get(key)?;

    let get_color = |field: &str| -> egui::Color32 {
        nt.get(field)
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color)
            .unwrap_or(egui::Color32::GRAY)
    };
    let get_f32 = |field: &str, default: f32| -> f32 {
        nt.get(field).and_then(|v| v.as_f64()).map(|v| v as f32).unwrap_or(default)
    };

    let h1 = get_color("noteH1Color");
    let h2 = get_color("noteH2Color");
    let h3 = get_color("noteH3Color");
    let h4 = get_color("noteH4Color");

    let base_theme = match mode {
        ThemeMode::Light | ThemeMode::Auto => Theme::light(),
        ThemeMode::Dark => Theme::dark(),
    };

    let note_link_underline = nt.get("noteLinkUnderline")
        .and_then(|v| v.as_str())
        .map(|s| s == "underline")
        .unwrap_or(true);
    let note_code_style = nt.get("noteCodeStyle")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(Theme {
        name: match mode {
            ThemeMode::Light | ThemeMode::Auto => "light",
            ThemeMode::Dark => "dark",
        },
        base: base_theme.base,
        heading: HeadingStyle {
            colors: [h1, h2, h3, h4, h4, h4],
            ..base_theme.heading
        },
        code: CodeStyle {
            inline_bg: get_color("noteMarkerBackgroundColor"),
            inline_text: get_color("noteMarkerTextColor"),
            block_bg: get_color("noteCodeBackgroundColor"),
            block_border_color: get_color("noteCodeBorderColor"),
            block_rounding: get_f32("noteCodeBorderRadius", 4.0),
            block_style: note_code_style,
            ..base_theme.code
        },
        quote: QuoteStyle {
            bar_color: get_color("noteQuoteBorderColor"),
            bg_color: get_color("noteQuoteBackgroundColor"),
            text_color: get_color("noteQuoteTextColor"),
            bar_width: get_f32("noteQuoteBorderWidth", 4.0),
            ..base_theme.quote
        },
        table: TableStyle {
            header_bg: get_color("noteTableHeaderBgColor"),
            row_bg: get_color("noteTableBgColor"),
            alt_row_bg: get_color("noteTableEvenRowBgColor"),
            border_color: get_color("noteTableBorderColor"),
            border_radius: get_f32("noteTableBorderRadius", 4.0),
            ..base_theme.table
        },
        link: LinkStyle {
            color: get_color("noteLinkColor"),
            underline: note_link_underline,
        },
        ..base_theme
    })
}

/// 从 WhaleTerm preferences.json 加载应用 UI 主题
fn load_ui_theme(mode: ThemeMode) -> UiTheme {
    let root = match read_preferences() {
        Some(r) => r,
        None => return UiTheme::default_for(mode),
    };
    let key = match mode {
        ThemeMode::Light | ThemeMode::Auto => "themeLight",
        ThemeMode::Dark => "themeDark",
    };
    let theme_obj = match root.get(key) {
        Some(t) => t,
        None => return UiTheme::default_for(mode),
    };

    let get = |field: &str| -> egui::Color32 {
        theme_obj.get(field)
            .and_then(|v| v.as_str())
            .and_then(parse_hex_color)
            .unwrap_or(egui::Color32::GRAY)
    };

    UiTheme {
        // === 应用基础 ===
        menu_bg: get("appBgColor"),
        menu_text: get("appHeaderTextColor"),
        text_color: get("textColor"),
        text_active_color: get("textActiveColor"),
        border: get("borderColor"),
        divider: get("appDividerColor"),
        split_color: get("appSplitColor"),

        // === 侧边栏 ===
        sidebar_bg: get("appSiderBarBgColor"),
        sidebar_hover_bg: get("appSideHoverBgColor"),
        sidebar_active_text_color: get("appSideTextActiveColor"),
        sidebar_text: get("appSideTextColor"),

        // === 状态栏 ===
        status_bar_bg: get("appStatusBarBgColor"),
        status_bar_text: get("appStatusBarTextColor"),
        status_bar_text_hover: get("appStatusBarTextHoverColor"),

        // === 左侧列表 ===
        left_list_bg: get("appLeftListBgColor"),
        left_list_bg_hover: get("appLeftListBgColorHover"),
        sidebar_active_bg: get("appLeftListBgColorActive"),
        sidebar_active_text: get("appLeftListTextColorActive"),
        search_title_bg: get("appSearchTitleBgColor"),

        // === 右侧内容区域 ===
        content_bg: get("appContentNoteBgColor"),
        content_term_bg: get("appContentTermBgColor"),
        content_chat_bg: get("appContentChatBgColor"),
        content_chat_divider: get("appContentChatDividerColor"),
        content_tran_bg: get("appContentTranBgColor"),

        // === 弹出层 ===
        dialog_bg: get("dialogBgColor"),
        dialog_border: get("dialogBorderColor"),
        dialog_divider: get("dialogDividerColor"),
        dialog_text: get("dialogTextColor"),
        dialog_text_active: get("dialogTextActiveColor"),

        // === 下拉菜单 ===
        drop_down_text: get("dropDownColor"),
        drop_down_bg: get("dropDownBgColor"),
        drop_down_active_text: get("dropDownActiveColor"),
        drop_down_active_bg: get("dropDownActiveBgColor"),

        // === AI Chat 消息 ===
        chat_send_bg: get("appContentChatSendBgColor"),
        chat_send_border: get("appContentChatSendBorderColor"),
        chat_reply_bg: get("appContentChatReplyBgColor"),
        chat_reply_border: get("appContentChatReplyBorderColor"),

        // === 输入框 ===
        input_bg: get("inputContentBgColor"),
        input_border: get("inputContentBorderColor"),

        // === 表格 ===
        table_bg: get("tableBgColor"),
        table_border: get("tableBorderColor"),
        table_header_bg: get("tableHeaderBgColor"),
        table_even_row_bg: get("tableEvenRowBgColor"),
    }
}

/// 加载扩展主题（工具栏、大纲等额外样式）
fn load_extra_theme(mode: ThemeMode) -> ExtraTheme {
    match mode {
        // 文档 5.1 节 - 亮色扩展色
        ThemeMode::Light | ThemeMode::Auto => ExtraTheme {
            tab_icon_color: egui::Color32::from_rgb(0x00, 0x00, 0x00),
            active_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
            search_icon_color: egui::Color32::from_rgb(0x7C, 0x86, 0x8F),
            edit_disabled_color: egui::Color32::from_rgb(0xC2, 0xC2, 0xC2),

            note_tab_header_border: egui::Color32::from_rgb(0xD9, 0xD9, 0xD9),
            note_toolbar_header_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
            note_search_num_bg_color: egui::Color32::from_rgb(0xD8, 0xD8, 0xD8),
            outline_hover_color: egui::Color32::from_rgb(0x42, 0x85, 0xF4),

            table_th_bg: egui::Color32::from_rgb(0xEB, 0xEB, 0xEB),
            table_td_bg: egui::Color32::from_rgb(0xF9, 0xF9, 0xF9),
            table_hover_color: egui::Color32::from_rgb(0xEE, 0xEE, 0xEE),

            progress_free_bg: egui::Color32::from_rgb(0xEA, 0xEA, 0xEA),
            expand_table_bg: egui::Color32::WHITE,

            info_title_btn_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCB),
            info_title_btn_border_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCB),
            info_title_btn_hover_bg_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCB),
        },
        // 文档 5.2 节 - 暗色扩展色
        ThemeMode::Dark => ExtraTheme {
            tab_icon_color: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
            active_color: egui::Color32::WHITE,
            search_icon_color: egui::Color32::from_rgb(0xCC, 0xCC, 0xCB),
            edit_disabled_color: egui::Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x61),

            note_tab_header_border: egui::Color32::from_rgb(0x10, 0x5C, 0x5D),
            note_toolbar_header_bg: egui::Color32::from_rgb(0x02, 0x38, 0x48),
            note_search_num_bg_color: egui::Color32::from_rgb(0x01, 0x53, 0x67),
            outline_hover_color: egui::Color32::WHITE,

            table_th_bg: egui::Color32::from_rgb(0x05, 0x37, 0x47),
            table_td_bg: egui::Color32::from_rgb(0x00, 0x30, 0x3F),
            table_hover_color: egui::Color32::from_rgb(0x03, 0x3C, 0x4F),

            progress_free_bg: egui::Color32::WHITE,
            expand_table_bg: egui::Color32::from_rgb(0x00, 0x27, 0x33),

            info_title_btn_color: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
            info_title_btn_border_color: egui::Color32::from_rgb(0x00, 0x5A, 0x6F),
            info_title_btn_hover_bg_color: egui::Color32::TRANSPARENT,
        },
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::default_for(ThemeMode::Light)
    }
}

impl UiTheme {
    fn default_for(mode: ThemeMode) -> Self {
        match mode {
            // Default Light Modern (文档 2.3 节)
            ThemeMode::Light | ThemeMode::Auto => UiTheme {
                menu_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                menu_text: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                text_color: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                text_active_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                divider: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                split_color: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                sidebar_bg: egui::Color32::WHITE,
                sidebar_hover_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                sidebar_active_text_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                sidebar_text: egui::Color32::from_rgb(0x66, 0x66, 0x66),
                status_bar_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                status_bar_text: egui::Color32::from_rgb(0x66, 0x66, 0x66),
                status_bar_text_hover: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                left_list_bg: egui::Color32::WHITE,
                left_list_bg_hover: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                sidebar_active_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                sidebar_active_text: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                search_title_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                content_bg: egui::Color32::WHITE,
                content_term_bg: egui::Color32::WHITE,
                content_chat_bg: egui::Color32::WHITE,
                content_chat_divider: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                content_tran_bg: egui::Color32::WHITE,
                dialog_bg: egui::Color32::WHITE,
                dialog_border: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                dialog_divider: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                dialog_text: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                dialog_text_active: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                drop_down_text: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                drop_down_bg: egui::Color32::WHITE,
                drop_down_active_text: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                drop_down_active_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                chat_send_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                chat_send_border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                chat_reply_bg: egui::Color32::WHITE,
                chat_reply_border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                input_bg: egui::Color32::WHITE,
                input_border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                table_bg: egui::Color32::WHITE,
                table_border: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                table_header_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                table_even_row_bg: egui::Color32::from_rgb(0xF9, 0xF9, 0xF9),
            },
            // Solarized Dark (文档 2.2 节)
            ThemeMode::Dark => UiTheme {
                menu_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                menu_text: egui::Color32::WHITE,
                text_color: egui::Color32::WHITE,
                text_active_color: egui::Color32::WHITE,
                border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                divider: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                split_color: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_hover_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_active_text_color: egui::Color32::WHITE,
                sidebar_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                status_bar_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                status_bar_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                status_bar_text_hover: egui::Color32::WHITE,
                left_list_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                left_list_bg_hover: egui::Color32::from_rgb(0x09, 0x49, 0x5E),
                sidebar_active_bg: egui::Color32::from_rgb(0x09, 0x47, 0x71),
                sidebar_active_text: egui::Color32::WHITE,
                search_title_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                content_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                content_term_bg: egui::Color32::BLACK,
                content_chat_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                content_chat_divider: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                content_tran_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                dialog_bg: egui::Color32::from_rgb(0x00, 0x22, 0x2B),
                dialog_border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                dialog_divider: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                dialog_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                dialog_text_active: egui::Color32::WHITE,
                drop_down_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                drop_down_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                drop_down_active_text: egui::Color32::WHITE,
                drop_down_active_bg: egui::Color32::from_rgb(0x09, 0x49, 0x5E),
                chat_send_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                chat_send_border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                chat_reply_bg: egui::Color32::from_rgb(0x00, 0x22, 0x2B),
                chat_reply_border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                input_bg: egui::Color32::from_rgb(0x00, 0x22, 0x2B),
                input_border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                table_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                table_border: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                table_header_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                table_even_row_bg: egui::Color32::from_rgb(0x00, 0x22, 0x2B),
            },
        }
    }
}

/// MdEdit 应用主结构体 - 持有所有运行时状态
pub struct MdEditApp {
    document: Document,
    outline_items: Vec<OutlineItem>,
    show_outline: bool,
    theme: Theme,
    theme_mode: ThemeMode,
    ui_theme: UiTheme,
    extra_theme: ExtraTheme,
    edit_mode: EditMode,
    ui_font_size: f32,
    ui_font_bold: bool,
    scroll_to_line: Option<usize>,
    active_block: Option<usize>,
    editing_text: String,
    last_window_pos: Option<(f32, f32)>,
    last_window_size: Option<(f32, f32)>,
    last_maximized: bool,
    target_physical_pos: Option<(f32, f32)>,
    frame_count: u32,

    // 布局状态
    left_panel_width: f32,
    active_tab: LeftPanelTab,
    left_panel_resizing: bool,

    // 文件树
    file_tree: FileTreeState,

    // 搜索
    search_bar: SearchBarState,
    search_tree: SearchTreeState,

    // 大纲状态
    outline_state: OutlineState,

    // 自动保存
    auto_save: AutoSaveState,
    auto_save_current_time: f64,

    // 字体缩放
    zoom_show_until: f64,
}

impl MdEditApp {
    /// 创建应用实例 - 初始化主题、字体、编辑模式等
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_file: Option<(PathBuf, String)>,
        cfg: &AppConfig,
        target_physical_pos: Option<(f32, f32)>,
    ) -> Self {
        let note_font = load_note_font_config();
        let config_font = load_config_font_config();
        Self::configure_fonts(&cc.egui_ctx, &note_font, &config_font);

        let (document, outline_items) = if let Some((path, content)) = initial_file {
            let document = Document::from_file(path, content);
            let outline_items = outline::extract_outline(document.content());
            (document, outline_items)
        } else {
            (Document::new(), Vec::new())
        };

        let theme_mode = if cfg.theme == "dark" {
            ThemeMode::Dark
        } else if cfg.theme == "auto" {
            ThemeMode::Auto
        } else {
            ThemeMode::Light
        };
        let effective = match theme_mode {
            ThemeMode::Auto => if Self::is_system_dark() { ThemeMode::Dark } else { ThemeMode::Light },
            _ => theme_mode,
        };
        let mut theme = load_note_theme(effective)
            .or_else(|| Some(Self::load_css_theme(effective)))
            .unwrap_or_else(|| match effective {
                ThemeMode::Light => Theme::light(),
                ThemeMode::Dark => Theme::dark(),
                ThemeMode::Auto => Theme::light(),
            });
        theme.font.base_size = note_font.size;
        theme.font.monospace_size = note_font.size;
        let ui_theme = load_ui_theme(effective);
        let extra_theme = load_extra_theme(effective);

        // 设置 egui visuals 匹配主题
        match effective {
            ThemeMode::Dark => cc.egui_ctx.set_visuals(egui::Visuals::dark()),
            _ => cc.egui_ctx.set_visuals(egui::Visuals::light()),
        }

        let edit_mode = if cfg.edit_mode == "raw" {
            EditMode::Raw
        } else {
            EditMode::Preview
        };

        Self {
            document,
            outline_items,
            show_outline: true,
            theme,
            theme_mode,
            ui_theme,
            extra_theme,
            edit_mode,
            ui_font_size: config_font.size,
            ui_font_bold: config_font.bold,
            scroll_to_line: None,
            active_block: None,
            editing_text: String::new(),
            last_window_pos: None,
            last_window_size: None,
            last_maximized: false,
            target_physical_pos,
            frame_count: 0,

            left_panel_width: 250.0,
            active_tab: LeftPanelTab::Outline,
            left_panel_resizing: false,

            file_tree: FileTreeState::new(),
            search_bar: SearchBarState::new(),
            search_tree: SearchTreeState::new(),
            outline_state: OutlineState::new(),
            auto_save: AutoSaveState::new(),
            auto_save_current_time: 0.0,
            zoom_show_until: 0.0,
        }
    }

    /// 配置字体 - 加载笔记字体和 UI 字体，回退到系统 CJK 字体
    fn configure_fonts(ctx: &egui::Context, note_font: &NoteFontConfig, config_font: &ConfigFontConfig) {
        let mut fonts = egui::FontDefinitions::default();

        // 加载 note 字体（编辑区和大纲面板使用）
        for family_name in &note_font.family {
            if let Some(font_data) = find_font_data(family_name) {
                fonts.font_data.insert(
                    "note_font".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                for family in [&egui::FontFamily::Proportional, &egui::FontFamily::Monospace] {
                    let list = fonts.families.get_mut(family).unwrap();
                    list.insert(0, "note_font".to_owned());
                }
                break;
            }
        }

        // 加载 config 字体（UI 菜单栏等使用），加入 Proportional 族作为备选
        for family_name in &config_font.family {
            if let Some(font_data) = find_font_data(family_name) {
                fonts.font_data.insert(
                    "ui_font".to_owned(),
                    egui::FontData::from_owned(font_data).into(),
                );
                fonts.families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .push("ui_font".to_owned());
                break;
            }
        }

        // 如果 note 字体未加载成功，回退到系统默认 CJK 字体
        if !fonts.font_data.contains_key("note_font") {
            let fallback_paths = if cfg!(target_os = "windows") {
                vec![
                    "C:\\Windows\\Fonts\\msyh.ttc",
                    "C:\\Windows\\Fonts\\simsun.ttc",
                ]
            } else if cfg!(target_os = "macos") {
                vec![
                    "/System/Library/Fonts/PingFang.ttc",
                    "/System/Library/Fonts/STHeiti Light.ttc",
                ]
            } else {
                vec![
                    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
                ]
            };

            for path in fallback_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "cjk_font".to_owned(),
                        egui::FontData::from_owned(font_data).into(),
                    );
                    fonts.families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .push("cjk_font".to_owned());
                    fonts.families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .push("cjk_font".to_owned());
                    break;
                }
            }
        }

        ctx.set_fonts(fonts);
    }

    /// 从 CSS 文件加载主题（WhaleTerm 主题目录下的 light.css / dark.css）
    fn load_css_theme(mode: ThemeMode) -> Theme {
        let filename = match mode {
            ThemeMode::Light | ThemeMode::Auto => "light.css",
            ThemeMode::Dark => "dark.css",
        };
        let path = Path::new(CSS_THEME_DIR).join(filename);
        css_loader::load_theme_from_css(&path).unwrap_or_else(|| {
            match mode {
                ThemeMode::Light | ThemeMode::Auto => Theme::light(),
                ThemeMode::Dark => Theme::dark(),
            }
        })
    }

    /// 检测系统是否为深色模式（仅 Windows，通过注册表查询）
    fn is_system_dark() -> bool {
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command",
                "(Get-ItemProperty 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize').AppsUseLightTheme"])
            .output();
        match output {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                s == "0"
            },
            Err(_) => false,
        }
    }

    /// 获取实际生效的主题模式（Auto 模式下根据系统设置决定）
    fn effective_mode(&self) -> (ThemeMode, ThemeMode) {
        match self.theme_mode {
            ThemeMode::Auto => {
                let effective = if Self::is_system_dark() { ThemeMode::Dark } else { ThemeMode::Light };
                (ThemeMode::Auto, effective)
            },
            other => (other, other),
        }
    }

    /// 切换主题模式并重新加载所有主题相关资源
    fn switch_theme(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        let effective = match mode {
            ThemeMode::Auto => if Self::is_system_dark() { ThemeMode::Dark } else { ThemeMode::Light },
            _ => mode,
        };
        let font_size = self.theme.font.base_size;
        let mut theme = load_note_theme(effective)
            .or_else(|| Some(Self::load_css_theme(effective)))
            .unwrap_or_else(|| match effective {
                ThemeMode::Light => Theme::light(),
                ThemeMode::Dark => Theme::dark(),
                ThemeMode::Auto => Theme::light(),
            });
        theme.font.base_size = font_size;
        theme.font.monospace_size = font_size;
        self.theme = theme;
        self.ui_theme = load_ui_theme(effective);
        self.extra_theme = load_extra_theme(effective);
    }

    /// 更新大纲内容（文档变更后调用）
    fn update_outline(&mut self) {
        self.outline_items = outline::extract_outline(self.document.content());
    }

    /// 处理全局快捷键（Ctrl+S/O/N/B/I/F/E/+/- 等）
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);
        let current_time = ctx.input(|i| i.time);
        if ctrl {
            if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                if shift {
                    self.save_file_as();
                } else {
                    self.save_file();
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::O)) {
                self.open_file();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                self.new_file();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::B)) {
                self.toggle_format("**");
            }
            if ctx.input(|i| i.key_pressed(egui::Key::I)) {
                self.toggle_format("*");
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                self.search_bar.visible = !self.search_bar.visible;
                if self.search_bar.visible {
                    self.search_bar.query.clear();
                    self.search_bar.total_matches = 0;
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::E)) {
                // Ctrl+E 切换编辑模式
                match self.edit_mode {
                    EditMode::Raw => self.edit_mode = EditMode::Preview,
                    EditMode::Preview => self.edit_mode = EditMode::Raw,
                }
            }
            // 字体缩放
            let zoom_changed = ctx.input(|i| {
                let mut changed = false;
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    self.theme.font.base_size = (self.theme.font.base_size + 1.0).min(32.0);
                    self.theme.font.monospace_size = self.theme.font.base_size;
                    changed = true;
                }
                if i.key_pressed(egui::Key::Minus) {
                    self.theme.font.base_size = (self.theme.font.base_size - 1.0).max(12.0);
                    self.theme.font.monospace_size = self.theme.font.base_size;
                    changed = true;
                }
                changed
            });
            if zoom_changed {
                self.zoom_show_until = current_time + 3.0;
            }
        }
    }

    /// 新建空白文档
    fn new_file(&mut self) {
        self.document = Document::new();
        self.outline_items.clear();
    }

    /// 标记文档已修改并触发自动保存计时
    fn mark_modified(&mut self) {
        self.document.modified = true;
        if self.auto_save_current_time > 0.0 {
            self.auto_save.on_edit(self.auto_save_current_time);
        }
    }

    /// 打开文件（通过系统文件选择对话框）
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                // 如果文件树没有数据目录，用文件所在目录初始化
                if self.file_tree.data_dir.is_none() {
                    if let Some(parent) = path.parent() {
                        self.file_tree.set_data_dir(parent.to_path_buf());
                    }
                }
                self.document = Document::from_file(path, content);
                self.update_outline();
            }
        }
    }

    /// 保存文件（未关联路径时弹出另存为对话框）
    fn save_file(&mut self) {
        let path = if let Some(ref p) = self.document.path {
            p.clone()
        } else {
            match rfd::FileDialog::new()
                .add_filter("Markdown", &["md", "markdown"])
                .save_file()
            {
                Some(p) => {
                    self.document.path = Some(p.clone());
                    p
                }
                None => return,
            }
        };
        if std::fs::write(&path, self.document.content()).is_ok() {
            self.document.modified = false;
        }
    }

    /// 另存为（始终弹出文件选择对话框）
    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .save_file()
        {
            self.document.path = Some(path.clone());
            if std::fs::write(&path, self.document.content()).is_ok() {
                self.document.modified = false;
            }
        }
    }

    /// 切换行内格式标记（加粗/斜体/删除线/行内代码）
    fn toggle_format(&mut self, marker: &str) {
        if let Some(_idx) = self.active_block {
            let text = &mut self.editing_text;
            if text.starts_with(marker) && text.ends_with(marker) && text.len() > marker.len() * 2 {
                let inner = text[marker.len()..text.len() - marker.len()].to_string();
                *text = inner;
            } else {
                *text = format!("{}{}{}", marker, text, marker);
            }
        }
    }

    /// 处理工具栏按钮操作
    fn handle_toolbar_action(&mut self, action: ToolbarAction, _ctx: &egui::Context) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::ToggleMode(mode) => {
                if self.edit_mode != mode {
                    if mode == EditMode::Raw {
                        if self.active_block.is_some() {
                            let snap = self.document.content().to_string();
                            let blocks = editor::split_blocks(&snap);
                            self.commit_edit(&blocks);
                            self.active_block = None;
                        }
                    }
                    self.edit_mode = mode;
                }
            }
            ToolbarAction::Undo => {
                // TODO: 阶段7集成撤销重做
            }
            ToolbarAction::Redo => {
                // TODO: 阶段7集成撤销重做
            }
            ToolbarAction::Bold => self.toggle_format("**"),
            ToolbarAction::Italic => self.toggle_format("*"),
            ToolbarAction::Strike => self.toggle_format("~~"),
            ToolbarAction::InlineCode => self.toggle_format("`"),
            ToolbarAction::Heading => self.insert_at_line_start("# "),
            ToolbarAction::Quote => self.insert_at_line_start("> "),
            ToolbarAction::UnorderedList => self.insert_at_line_start("- "),
            ToolbarAction::OrderedList => self.insert_at_line_start("1. "),
            ToolbarAction::CheckList => self.insert_at_line_start("- [ ] "),
            ToolbarAction::CodeBlock => {
                self.editing_text = format!("```\n{}\n```", self.editing_text);
            }
            ToolbarAction::HorizontalRule => {
                self.editing_text = format!("{}\n---\n", self.editing_text);
            }
            ToolbarAction::Table => {
                self.editing_text = format!("{}\n|  |  |\n|---|---|\n|  |  |\n", self.editing_text);
            }
            ToolbarAction::Link => {
                self.editing_text = format!("[{}](url)", self.editing_text);
            }
            ToolbarAction::ToggleOutline => {
                self.show_outline = !self.show_outline;
            }
            ToolbarAction::ToggleSearch => {
                // TODO: 阶段5搜索功能
            }
            ToolbarAction::Indent => self.insert_at_line_start("    "),
            ToolbarAction::Outdent => {
                let text = &mut self.editing_text;
                if text.starts_with("    ") {
                    *text = text[4..].to_string();
                }
            }
            ToolbarAction::FontColor | ToolbarAction::BgColor => {
                // TODO: 阶段8颜色选择器
            }
            ToolbarAction::AttachFile => {
                // TODO: 阶段8附件功能
            }
        }
    }

    /// 在当前编辑块每行首插入/移除前缀
    fn insert_at_line_start(&mut self, prefix: &str) {
        if self.active_block.is_some() {
            let text = &mut self.editing_text;
            *text = text.lines()
                .map(|line| {
                    if line.starts_with(prefix) {
                        line[prefix.len()..].to_string()
                    } else {
                        format!("{}{}", prefix, line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
    }

    /// 生成窗口标题（文件名 + 修改标记 + 应用名）
    fn title(&self) -> String {
        let name = self.document.path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "未命名".to_string());
        let modified = if self.document.modified { " *" } else { "" };
        format!("{}{} - mdedit", name, modified)
    }
}

impl eframe::App for MdEditApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        // 第2帧：通过 ViewportCommand 设置窗口位置（物理坐标 / ppp）
        if self.frame_count == 2 {
            if let Some((px, py)) = self.target_physical_pos.take() {
                let ppp = ctx.pixels_per_point();
                let pos = egui::pos2(px / ppp, py / ppp);
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            }

            // 设置 UI 字体样式（菜单栏、按钮等）
            {
                let mut style = (*ctx.style()).clone();
                for (key, size) in [
                    (egui::TextStyle::Body, self.ui_font_size),
                    (egui::TextStyle::Button, self.ui_font_size),
                    (egui::TextStyle::Small, self.ui_font_size - 1.0),
                ] {
                    if let Some(entry) = style.text_styles.get_mut(&key) {
                        entry.size = size;
                    }
                }
                ctx.set_style(style);
            }
        }

        self.handle_shortcuts(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));

        // 自动保存检查
        let current_time = ctx.input(|i| i.time);
        if current_time > 0.0 && self.auto_save.check(current_time) && self.document.modified {
            if let Some(ref path) = self.document.path {
                if std::fs::write(path, self.document.content()).is_ok() {
                    self.document.modified = false;
                }
            }
        }
        self.auto_save_current_time = current_time;

        // === 菜单栏（保留在顶部，用于文件/视图操作） ===
        let menu_bg = self.ui_theme.menu_bg;
        let menu_text = self.ui_theme.menu_text;

        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::default()
                .fill(menu_bg)
                .inner_margin(egui::Margin::symmetric(4.0, 2.0)))
            .show(ctx, |ui| {
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = menu_text;
            ui.visuals_mut().widgets.inactive.fg_stroke.color = menu_text;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = menu_text;
            ui.visuals_mut().widgets.active.fg_stroke.color = menu_text;
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("新建 (Ctrl+N)").clicked() {
                        self.new_file();
                        ui.close_menu();
                    }
                    if ui.button("打开 (Ctrl+O)").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    if ui.button("保存 (Ctrl+S)").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("另存为 (Ctrl+Shift+S)").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                });
                ui.menu_button("视图", |ui| {
                    ui.label("主题");
                    let prev_mode = self.theme_mode;
                    ui.radio_value(&mut self.theme_mode, ThemeMode::Light, "浅色");
                    ui.radio_value(&mut self.theme_mode, ThemeMode::Dark, "深色");
                    ui.radio_value(&mut self.theme_mode, ThemeMode::Auto, "跟随系统");
                    if self.theme_mode != prev_mode {
                        self.switch_theme(self.theme_mode);
                        let (_, effective) = self.effective_mode();
                        match effective {
                            ThemeMode::Dark => ctx.set_visuals(egui::Visuals::dark()),
                            _ => ctx.set_visuals(egui::Visuals::light()),
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("编辑模式");
                    let prev_edit_mode = self.edit_mode;
                    ui.radio_value(&mut self.edit_mode, EditMode::Preview, "预览编辑");
                    ui.radio_value(&mut self.edit_mode, EditMode::Raw, "原始编辑");
                    if self.edit_mode != prev_edit_mode {
                        if self.edit_mode == EditMode::Raw {
                            if self.active_block.is_some() {
                                let snap = self.document.content().to_string();
                                let blocks = editor::split_blocks(&snap);
                                self.commit_edit(&blocks);
                                self.active_block = None;
                            }
                        }
                        ui.close_menu();
                    }
                });
            });
        });

        // === 编辑器工具栏 (35px) ===
        let is_dark = matches!(self.effective_mode().1, ThemeMode::Dark);
        let tb_bg = if is_dark {
            egui::Color32::from_rgb(0x1D, 0x21, 0x25)
        } else {
            egui::Color32::from_rgb(0xF6, 0xF8, 0xFA)
        };
        let tb_icon = if is_dark {
            egui::Color32::from_rgb(0xB9, 0xB9, 0xB9)
        } else {
            egui::Color32::from_rgb(0x58, 0x60, 0x69)
        };
        let tb_hover = if is_dark {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(0x42, 0x85, 0xF4)
        };
        let tb_sep = egui::Color32::from_rgba_unmultiplied(128, 128, 128, 80);

        let mut toolbar_action = ToolbarAction::None;
        egui::TopBottomPanel::top("editor_toolbar")
            .exact_height(35.0)
            .frame(egui::Frame::default()
                .fill(tb_bg)
                .inner_margin(egui::Margin::symmetric(5.0, 5.0)))
            .show(ctx, |ui| {
                let tb_state = ToolbarState {
                    show_outline: self.show_outline,
                };
                toolbar_action = toolbar::render_toolbar(
                    ui, &tb_state, self.edit_mode,
                    tb_icon, tb_hover, tb_sep,
                );
            });

        // 处理工具栏动作
        self.handle_toolbar_action(toolbar_action, ctx);

        // === 编辑器搜索栏 (浮动) ===
        let search_action = search::render_search_bar(
            ctx, &mut self.search_bar,
            self.ui_theme.content_bg,
            self.ui_theme.input_border,
            self.ui_theme.text_color,
            self.ui_font_size,
        );
        match search_action {
            search::SearchBarAction::QueryChanged => {
                self.search_bar.count_matches(self.document.content());
            }
            search::SearchBarAction::NextMatch => {
                if self.search_bar.total_matches > 0 {
                    self.search_bar.current_match = (self.search_bar.current_match + 1) % self.search_bar.total_matches;
                }
            }
            search::SearchBarAction::PrevMatch => {
                if self.search_bar.total_matches > 0 {
                    self.search_bar.current_match = if self.search_bar.current_match == 0 {
                        self.search_bar.total_matches - 1
                    } else {
                        self.search_bar.current_match - 1
                    };
                }
            }
            search::SearchBarAction::Replace => {
                if !self.search_bar.query.is_empty() {
                    let content = self.document.content().to_string();
                    let matches = self.search_bar.matches(&content);
                    if !matches.is_empty() {
                        let idx = self.search_bar.current_match.min(matches.len() - 1);
                        let (start, end) = matches[idx];
                        let new_content = format!("{}{}{}", &content[..start], self.search_bar.replace_text, &content[end..]);
                        *self.document.buffer.as_mut_string() = new_content;
                        self.mark_modified();
                        self.search_bar.count_matches(self.document.content());
                    }
                }
            }
            search::SearchBarAction::ReplaceAll => {
                if !self.search_bar.query.is_empty() {
                    let content = self.document.content().to_string();
                    let matches = self.search_bar.matches(&content);
                    if !matches.is_empty() {
                        let new_content = if self.search_bar.case_sensitive {
                            content.replace(&self.search_bar.query, &self.search_bar.replace_text)
                        } else {
                            regex::RegexBuilder::new(&regex::escape(&self.search_bar.query))
                                .case_insensitive(true)
                                .build()
                                .map(|re| re.replace_all(&content, self.search_bar.replace_text.as_str()).to_string())
                                .unwrap_or(content)
                        };
                        *self.document.buffer.as_mut_string() = new_content;
                        self.mark_modified();
                        self.search_bar.count_matches(self.document.content());
                    }
                }
            }
            search::SearchBarAction::None => {}
        }

        // === 三栏布局：Ribbon + 左面板 + 编辑区 ===
        let ribbon_width = 48.0;
        let min_left_width = 150.0;

        // Ribbon 窄侧栏
        let ribbon_bg = self.ui_theme.sidebar_bg;
        let ribbon_text = self.ui_theme.sidebar_text;
        let ribbon_active = self.ui_theme.text_active_color;
        let ribbon_hover = self.ui_theme.sidebar_hover_bg;
        egui::SidePanel::left("ribbon")
            .exact_width(ribbon_width)
            .resizable(false)
            .frame(egui::Frame::none().fill(ribbon_bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.visuals_mut().widgets.inactive.fg_stroke.color = ribbon_text;
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = ribbon_active;
                    ui.visuals_mut().widgets.hovered.bg_fill = ribbon_hover;
                    ui.visuals_mut().widgets.active.fg_stroke.color = ribbon_active;

                    let tab_color = |active: bool| if active { ribbon_active } else { ribbon_text };
                    // 目录按钮 - 文件夹图标
                    if ui.add(egui::Button::new(
                        egui::RichText::new("\u{1F4C1}").size(self.ui_font_size + 2.0).color(tab_color(self.active_tab == LeftPanelTab::Tree))
                    ).frame(self.active_tab == LeftPanelTab::Tree)).clicked() {
                        self.active_tab = LeftPanelTab::Tree;
                        self.show_outline = true;
                    }
                    // 大纲按钮 - 导航图标
                    if ui.add(egui::Button::new(
                        egui::RichText::new("\u{2630}").size(self.ui_font_size + 2.0).color(tab_color(self.active_tab == LeftPanelTab::Outline))
                    ).frame(self.active_tab == LeftPanelTab::Outline)).clicked() {
                        self.active_tab = LeftPanelTab::Outline;
                        self.show_outline = true;
                    }
                    // 搜索按钮 - 放大镜图标
                    if ui.add(egui::Button::new(
                        egui::RichText::new("\u{1F50D}").size(self.ui_font_size + 2.0).color(tab_color(self.active_tab == LeftPanelTab::Search))
                    ).frame(self.active_tab == LeftPanelTab::Search)).clicked() {
                        self.active_tab = LeftPanelTab::Search;
                        self.show_outline = true;
                    }

                    ui.add_space(12.0);

                    if ui.add(egui::Button::new(
                        egui::RichText::new("+").size(self.ui_font_size + 2.0).color(ribbon_text)
                    ).frame(false)).clicked() {
                        self.new_file();
                    }
                    if ui.add(egui::Button::new(
                        egui::RichText::new("...").size(self.ui_font_size).color(ribbon_text)
                    ).frame(false)).clicked() {
                        self.open_file();
                    }
                });
            });

        // 可拖拽左面板
        if self.show_outline {
            let left_bg = self.ui_theme.left_list_bg;
            let split_color = self.ui_theme.split_color;
            let tab_text_active = self.ui_theme.text_active_color;
            let tab_text = self.ui_theme.sidebar_text;
            let font_size = self.theme.font.base_size;

            egui::SidePanel::left("left_panel")
                .default_width(self.left_panel_width)
                .resizable(true)
                .min_width(min_left_width)
                .frame(egui::Frame::none().fill(left_bg))
                .show(ctx, |ui| {
                    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = tab_text;
                    ui.visuals_mut().widgets.inactive.fg_stroke.color = tab_text;

                    // Tab 栏 (32px 高)
                    let tab_bar_rect = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 32.0),
                        egui::Sense::hover(),
                    );
                    let tab_bar_rect = tab_bar_rect.0;

                    // 底边框
                    ui.painter().line_segment(
                        [tab_bar_rect.left_bottom(), tab_bar_rect.right_bottom()],
                        egui::Stroke::new(1.0, split_color),
                    );

                    // Tab 按钮
                    let tabs = [
                        (LeftPanelTab::Tree, "目录"),
                        (LeftPanelTab::Outline, "大纲"),
                        (LeftPanelTab::Search, "搜索"),
                    ];
                    let tab_font_size = font_size + 3.0;
                    let mut tab_x = tab_bar_rect.left() + 8.0;
                    for (tab, label) in &tabs {
                        let is_active = self.active_tab == *tab;
                        let galley = ui.painter().layout_no_wrap(
                            label.to_string(),
                            egui::FontId::proportional(tab_font_size),
                            if is_active { tab_text_active } else { tab_text },
                        );
                        let text_width = galley.size().x + 12.0;
                        let text_pos = egui::pos2(tab_x, tab_bar_rect.center().y - galley.size().y / 2.0);

                        let click_rect = egui::Rect::from_min_max(
                            egui::pos2(tab_x - 4.0, tab_bar_rect.top()),
                            egui::pos2(tab_x + text_width + 4.0, tab_bar_rect.bottom()),
                        );
                        let response = ui.interact(click_rect, ui.id().with(label), egui::Sense::click());
                        if response.clicked() {
                            self.active_tab = *tab;
                        }

                        let color = if is_active { tab_text_active } else { tab_text };
                        ui.painter().galley(text_pos, galley, color);

                        if is_active {
                            let indicator_rect = egui::Rect::from_min_max(
                                egui::pos2(tab_x - 2.0, tab_bar_rect.bottom() - 3.0),
                                egui::pos2(tab_x + text_width + 2.0, tab_bar_rect.bottom()),
                            );
                            ui.painter().rect_filled(indicator_rect, 0.0, tab_text_active);
                        }

                        tab_x += text_width + 12.0;
                    }

                    // 内容区域
                    match self.active_tab {
                        LeftPanelTab::Tree => {
                            if self.file_tree.data_dir.is_none() {
                                ui.vertical(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("未设置数据目录").size(font_size).color(tab_text));
                                    if ui.button("选择目录...").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .pick_folder()
                                        {
                                            self.file_tree.set_data_dir(path);
                                        }
                                    }
                                });
                            } else {
                                // 文件树标题栏 - 显示目录名+操作按钮，对齐原版截图
                                let dir_name = self.file_tree.data_dir.as_ref()
                                    .and_then(|p| p.file_name())
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "目录".to_string());
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(&dir_name).size(font_size + 2.0).strong().color(tab_text));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add(egui::Button::new(
                                            egui::RichText::new("\u{1F504}").size(font_size).color(tab_text)
                                        ).frame(false)).clicked() {
                                            self.file_tree.refresh();
                                        }
                                    });
                                });
                                ui.separator();

                                let hover_bg = self.ui_theme.left_list_bg_hover;
                                let active_bg = self.ui_theme.sidebar_active_bg;
                                let active_text = self.ui_theme.sidebar_active_text;
                                let tree_result = file_tree::render_file_tree(
                                    ui, &mut self.file_tree,
                                    self.ui_theme.sidebar_text,
                                    hover_bg, active_bg, active_text,
                                    font_size,
                                );
                                // 处理文件树事件
                                if let Some(path) = tree_result.clicked_file {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        self.document = Document::from_file(path, content);
                                        self.update_outline();
                                    }
                                }
                                match tree_result.action {
                                    FileTreeAction::NewFile(dir) => {
                                        let name = "新建文件.md";
                                        let path = dir.join(name);
                                        let mut i = 1;
                                        let mut final_path = path.clone();
                                        while final_path.exists() {
                                            final_path = dir.join(format!("新建文件{}.md", i));
                                            i += 1;
                                        }
                                        if let Err(e) = std::fs::write(&final_path, "") {
                                            eprintln!("创建文件失败: {}", e);
                                        }
                                        self.file_tree.refresh();
                                    }
                                    FileTreeAction::NewFolder(dir) => {
                                        let name = "新建文件夹";
                                        let mut final_path = dir.join(name);
                                        let mut i = 1;
                                        while final_path.exists() {
                                            final_path = dir.join(format!("新建文件夹{}", i));
                                            i += 1;
                                        }
                                        if let Err(e) = std::fs::create_dir_all(&final_path) {
                                            eprintln!("创建文件夹失败: {}", e);
                                        }
                                        self.file_tree.refresh();
                                    }
                                    FileTreeAction::Delete(path) => {
                                        if path.is_dir() {
                                            let _ = std::fs::remove_dir_all(&path);
                                        } else {
                                            let _ = std::fs::remove_file(&path);
                                        }
                                        if self.document.path.as_ref() == Some(&path) {
                                            self.document = Document::new();
                                            self.outline_items.clear();
                                        }
                                        self.file_tree.refresh();
                                    }
                                    FileTreeAction::Rename(_path) => {
                                        // TODO: 重命名对话框
                                    }
                                    FileTreeAction::Refresh => {
                                        self.file_tree.refresh();
                                    }
                                    FileTreeAction::Cut(path) => {
                                        self.file_tree.clipboard_is_dir = path.is_dir();
                                        self.file_tree.clipboard_path = Some(path);
                                        self.file_tree.clipboard_is_cut = true;
                                    }
                                    FileTreeAction::Copy(path) => {
                                        self.file_tree.clipboard_is_dir = path.is_dir();
                                        self.file_tree.clipboard_path = Some(path);
                                        self.file_tree.clipboard_is_cut = false;
                                    }
                                    FileTreeAction::Paste(target_dir) => {
                                        if let Some(ref src) = self.file_tree.clipboard_path {
                                            let file_name = src.file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            let mut dest = target_dir.join(&file_name);
                                            // 避免同名
                                            if dest.exists() && dest != *src {
                                                let stem = dest.file_stem()
                                                    .map(|s| s.to_string_lossy().to_string())
                                                    .unwrap_or_default();
                                                let ext = dest.extension()
                                                    .map(|e| format!(".{}", e.to_string_lossy()))
                                                    .unwrap_or_default();
                                                let mut i = 1;
                                                while target_dir.join(format!("{}({}){}", stem, i, ext)).exists() {
                                                    i += 1;
                                                }
                                                dest = target_dir.join(format!("{}({}){}", stem, i, ext));
                                            }
                                            if dest != *src {
                                                if self.file_tree.clipboard_is_cut {
                                                    let _ = std::fs::rename(src, &dest);
                                                } else {
                                                    if self.file_tree.clipboard_is_dir {
                                                        let _ = copy_dir_recursive(src, &dest);
                                                    } else {
                                                        let _ = std::fs::copy(src, &dest);
                                                    }
                                                }
                                                self.file_tree.refresh();
                                            }
                                        }
                                    }
                                    FileTreeAction::ImportMarkdown(dir) => {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Markdown", &["md", "markdown"])
                                            .pick_file()
                                        {
                                            let file_name = path.file_name()
                                                .map(|n| n.to_string_lossy().to_string())
                                                .unwrap_or_default();
                                            let dest = dir.join(&file_name);
                                            if dest != path {
                                                let _ = std::fs::copy(&path, &dest);
                                                self.file_tree.refresh();
                                            }
                                        }
                                    }
                                    FileTreeAction::SearchContent => {
                                        self.active_tab = LeftPanelTab::Search;
                                        self.search_tree.query.clear();
                                    }
                                    FileTreeAction::OpenAiChat => {
                                        // TODO: AI 聊天功能
                                    }
                                    FileTreeAction::None => {}
                                }
                            }
                        }
                        LeftPanelTab::Outline => {
                            let oh = self.extra_theme.outline_hover_color;
                            let st = self.ui_theme.sidebar_text;
                            let at = self.ui_theme.sidebar_active_text;
                            let _ab = self.ui_theme.sidebar_active_bg;
                            ui.visuals_mut().widgets.hovered.bg_fill = oh;
                            ui.visuals_mut().widgets.hovered.fg_stroke.color = st;
                            ui.visuals_mut().widgets.active.fg_stroke.color = st;

                            // 头部操作栏：展开级别 + 编号切换
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("展开:").size(font_size * 0.85).color(st));
                                for lvl in [1u8, 2, 3] {
                                    let active = self.outline_state.expand_level == lvl;
                                    let color = if active { at } else { st };
                                    if ui.add(egui::Button::new(
                                        egui::RichText::new(&lvl.to_string()).size(font_size * 0.85).color(color)
                                    ).frame(active)).clicked() {
                                        self.outline_state.expand_level = lvl;
                                    }
                                }
                                if ui.add(egui::Button::new(
                                    egui::RichText::new("All").size(font_size * 0.85).color(if self.outline_state.expand_level >= 6 { at } else { st })
                                ).frame(self.outline_state.expand_level >= 6)).clicked() {
                                    self.outline_state.expand_level = 6;
                                }
                                ui.separator();
                                let num_text = if self.outline_state.show_numbers { "1." } else { "#" };
                                if ui.add(egui::Button::new(
                                    egui::RichText::new(num_text).size(font_size * 0.85).color(if self.outline_state.show_numbers { at } else { st })
                                ).frame(self.outline_state.show_numbers)).clicked() {
                                    self.outline_state.show_numbers = !self.outline_state.show_numbers;
                                }
                            });
                            ui.separator();

                            // 大纲内容
                            let row_height = font_size * 3.0 - 4.0;
                            egui::ScrollArea::vertical()
                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                .show(ui, |ui| {
                                for (idx, item) in self.outline_items.iter().enumerate() {
                                    if !self.outline_state.is_visible(&self.outline_items, idx) { continue; }

                                    let indent = self.outline_state.indent(item.level, font_size);
                                    let item_font_size = self.outline_state.font_size(item.level, font_size);
                                    let number = self.outline_state.generate_number(&self.outline_items, idx);

                                    let label = if number.is_empty() {
                                        item.title.clone()
                                    } else {
                                        format!("{} {}", number, item.title)
                                    };

                                    let response = ui.allocate_response(
                                        egui::vec2(ui.available_width(), row_height),
                                        egui::Sense::click(),
                                    );
                                    let row_rect = response.rect;

                                    if response.hovered() {
                                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                        ui.painter().rect_filled(row_rect, 0.0, oh);
                                    }

                                    let text_x = row_rect.min.x + indent;
                                    ui.painter().text(
                                        egui::pos2(text_x, row_rect.center().y),
                                        egui::Align2::LEFT_CENTER,
                                        &label,
                                        egui::FontId::proportional(item_font_size),
                                        st,
                                    );

                                    if response.clicked() {
                                        self.scroll_to_line = Some(item.line);
                                    }
                                    // 右键菜单
                                    response.context_menu(|ui| {
                                            if ui.button("展开到 1 级").clicked() {
                                                self.outline_state.expand_level = 1;
                                                ui.close_menu();
                                            }
                                            if ui.button("展开到 2 级").clicked() {
                                                self.outline_state.expand_level = 2;
                                                ui.close_menu();
                                            }
                                            if ui.button("展开到 3 级").clicked() {
                                                self.outline_state.expand_level = 3;
                                                ui.close_menu();
                                            }
                                            ui.separator();
                                            if ui.button("全部展开").clicked() {
                                                self.outline_state.expand_level = 6;
                                                ui.close_menu();
                                            }
                                            ui.separator();
                                            let num_label = if self.outline_state.show_numbers { "隐藏编号" } else { "显示编号" };
                                            if ui.button(num_label).clicked() {
                                                self.outline_state.show_numbers = !self.outline_state.show_numbers;
                                                ui.close_menu();
                                            }
                                            if self.outline_state.show_numbers {
                                                ui.horizontal(|ui| {
                                                    ui.label("格式:");
                                                    for (fmt, label) in [(NumberFormat::Dot, "1."), (NumberFormat::None, "1"), (NumberFormat::Comma, "1、")] {
                                                        let active = self.outline_state.number_format == fmt;
                                                        if ui.add(egui::Button::new(
                                                            egui::RichText::new(label).size(font_size * 0.85).color(if active { at } else { st })
                                                        ).frame(active)).clicked() {
                                                            self.outline_state.number_format = fmt;
                                                        }
                                                    }
                                                });
                                            }
                                        });
                                }
                            });
                        }
                        LeftPanelTab::Search => {
                            let hover_bg = self.ui_theme.left_list_bg_hover;
                            let search_result = search::render_search_tree(
                                ui, &mut self.search_tree,
                                self.file_tree.data_dir.as_deref(),
                                self.ui_theme.sidebar_text,
                                font_size,
                                hover_bg,
                            );
                            if let Some((path, line)) = search_result.open_file {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    self.document = Document::from_file(path, content);
                                    self.update_outline();
                                    self.scroll_to_line = Some(line);
                                }
                            }
                        }
                    }

                    // 底部工具栏 - 对齐原版截图: [新建文件] [新建文件夹] | [搜索框] [搜索文件] [搜索内容]
                    let bottom_split = self.ui_theme.split_color;
                    let _bottom_bg = self.ui_theme.left_list_bg;
                    ui.separator();
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let btn_color = self.ui_theme.text_color;
                        let icon_size = self.ui_font_size + 2.0;

                        // 新建文件按钮
                        if ui.add(egui::Button::new(
                            egui::RichText::new("\u{1F4C4}+").size(icon_size).color(btn_color)
                        ).frame(false)).clicked() {
                            if let Some(dir) = self.file_tree.data_dir.clone() {
                                let name = "新建文件.md";
                                let mut final_path = dir.join(name);
                                let mut i = 1;
                                while final_path.exists() {
                                    final_path = dir.join(format!("新建文件{}.md", i));
                                    i += 1;
                                }
                                let _ = std::fs::write(&final_path, "");
                                self.file_tree.refresh();
                            }
                        }

                        // 新建文件夹按钮
                        if ui.add(egui::Button::new(
                            egui::RichText::new("\u{1F4C2}+").size(icon_size).color(btn_color)
                        ).frame(false)).clicked() {
                            if let Some(dir) = self.file_tree.data_dir.clone() {
                                let mut final_path = dir.join("新建文件夹");
                                let mut i = 1;
                                while final_path.exists() {
                                    final_path = dir.join(format!("新建文件夹{}", i));
                                    i += 1;
                                }
                                let _ = std::fs::create_dir_all(&final_path);
                                self.file_tree.refresh();
                            }
                        }

                        // 分隔线
                        let sep_rect = ui.available_rect_before_wrap();
                        let sep_x = sep_rect.min.x;
                        let sep_h = 19.0;
                        let sep_y = sep_rect.center().y - sep_h / 2.0;
                        ui.add_space(2.0);
                        ui.painter().line_segment(
                            [egui::pos2(sep_x + 1.0, sep_y), egui::pos2(sep_x + 1.0, sep_y + sep_h)],
                            egui::Stroke::new(1.0, bottom_split),
                        );
                        ui.add_space(4.0);

                        // 搜索框
                        let search_query = &mut self.search_tree.query;
                        let resp = ui.add(
                            egui::TextEdit::singleline(search_query)
                                .font(egui::FontId::proportional(self.ui_font_size))
                                .desired_width(ui.available_width() - 50.0)
                                .hint_text("搜索..."),
                        );

                        // 搜索按钮
                        if ui.add(egui::Button::new(
                            egui::RichText::new("\u{1F50D}").size(20.0).color(btn_color)
                        ).frame(false)).clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            if let Some(dir) = self.file_tree.data_dir.clone() {
                                let q = search_query.clone();
                                self.search_tree.search(&q, &dir);
                                self.active_tab = LeftPanelTab::Search;
                            }
                        }
                    });
                });
        }

        let editor_bg = if self.edit_mode == EditMode::Raw {
            self.ui_theme.content_bg
        } else {
            self.theme.base.background
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::default()
                .fill(editor_bg)
                .inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("editor_scroll")
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    self.render_editor(ui);
                });
        });

        // === 状态栏 ===
        let sb_bg = self.ui_theme.status_bar_bg;
        let sb_text = self.ui_theme.status_bar_text;
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .frame(egui::Frame::default().fill(sb_bg).inner_margin(egui::Margin::symmetric(8.0, 2.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.title().replace(" - mdedit", ""))
                        .size(self.ui_font_size - 1.0).color(sb_text));
                    ui.separator();
                    let mode = match self.edit_mode {
                        EditMode::Raw => "SV",
                        EditMode::Preview => "IR",
                    };
                    ui.label(egui::RichText::new(mode).size(self.ui_font_size - 1.0).color(sb_text));
                    ui.separator();
                    let char_count = self.document.content().chars().count();
                    let line_count = self.document.content().lines().count();
                    ui.label(egui::RichText::new(format!("{} 字符, {} 行", char_count, line_count))
                        .size(self.ui_font_size - 1.0).color(sb_text));
                    if self.document.modified {
                        ui.separator();
                        ui.label(egui::RichText::new("已修改").size(self.ui_font_size - 1.0)
                            .color(egui::Color32::from_rgb(0xF0, 0xA0, 0x20)));
                    }
                });
            });

        // === 字体缩放浮动提示 ===
        let current_time = ctx.input(|i| i.time);
        if current_time < self.zoom_show_until {
            egui::Area::new(egui::Id::new("zoom_indicator"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-20.0, 50.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let frame = egui::Frame::default()
                        .fill(self.ui_theme.sidebar_bg)
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0));
                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("-").frame(false)).clicked() {
                                self.theme.font.base_size = (self.theme.font.base_size - 1.0).max(12.0);
                                self.theme.font.monospace_size = self.theme.font.base_size;
                            }
                            ui.label(egui::RichText::new(format!("{:.0}px", self.theme.font.base_size))
                                .size(self.ui_font_size).color(self.ui_theme.text_color));
                            if ui.add(egui::Button::new("+").frame(false)).clicked() {
                                self.theme.font.base_size = (self.theme.font.base_size + 1.0).min(32.0);
                                self.theme.font.monospace_size = self.theme.font.base_size;
                            }
                            if ui.add(egui::Button::new("重置").frame(false)).clicked() {
                                self.theme.font.base_size = 15.0;
                                self.theme.font.monospace_size = 15.0;
                            }
                        });
                    });
                });
        }

        // 追踪窗口状态用于退出时保存
        let ppp = ctx.pixels_per_point();
        ctx.input(|i| {
            if let Some(rect) = i.viewport().inner_rect {
                // inner_rect 是 egui points，with_inner_size 也接受 egui points
                self.last_window_size = Some((rect.width(), rect.height()));
            }
            if let Some(rect) = i.viewport().outer_rect {
                // outer_rect 是 egui points，恢复时通过 OuterPosition(物理/ppp) 设置
                // 所以保存物理像素 = egui points * ppp
                self.last_window_pos = Some((rect.min.x * ppp, rect.min.y * ppp));
            }
            self.last_maximized = i.viewport().maximized.unwrap_or(false);
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_config();
    }

    fn on_exit(&mut self) {
        self.save_config();
    }
}

impl MdEditApp {
    /// 保存应用配置到 config.ini（窗口位置/大小/主题/编辑模式）
    fn save_config(&self) {
        let cfg = AppConfig {
            window_x: self.last_window_pos.map(|(x, _)| x),
            window_y: self.last_window_pos.map(|(_, y)| y),
            window_width: self.last_window_size.map(|(w, _)| w),
            window_height: self.last_window_size.map(|(_, h)| h),
            maximized: self.last_maximized,
            theme: match self.theme_mode {
                ThemeMode::Light => "light".to_string(),
                ThemeMode::Dark => "dark".to_string(),
                ThemeMode::Auto => "auto".to_string(),
            },
            edit_mode: match self.edit_mode {
                EditMode::Raw => "raw".to_string(),
                EditMode::Preview => "preview".to_string(),
            },
        };
        cfg.save();
    }
}

impl MdEditApp {
    /// 渲染编辑器主区域（预览模式：富文本块 + 可点击编辑；原始模式：纯文本）
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        if self.edit_mode == EditMode::Raw {
            self.render_raw_editor(ui);
            return;
        }

        let content_snapshot = self.document.content().to_string();
        let blocks = editor::split_blocks(&content_snapshot);

        let scroll_target = self.scroll_to_line.take();
        if let Some(target_line) = scroll_target {
            for (idx, block) in blocks.iter().enumerate() {
                if block.start_line <= target_line && target_line <= block.end_line {
                    self.active_block = Some(idx);
                    self.editing_text = block.source.clone();
                    break;
                }
            }
        }

        let edit_font = egui::FontId::monospace(self.theme.font.monospace_size);

        if blocks.is_empty() {
            let content = self.document.buffer.as_mut_string();
            let resp = ui.add(
                egui::TextEdit::multiline(content)
                    .font(edit_font.clone())
                    .desired_width(f32::INFINITY)
                    .frame(false)
                    .hint_text("输入 Markdown..."),
            );
            if resp.changed() {
                self.mark_modified();
                self.outline_items = outline::extract_outline(self.document.content());
            }
            return;
        }

        let mut new_active: Option<usize> = self.active_block;
        let mut content_changed = false;

        for (idx, block) in blocks.iter().enumerate() {
            let is_active = self.active_block == Some(idx);
            let should_scroll = scroll_target.map_or(false, |t| block.start_line <= t && t <= block.end_line);

            // 在 block 渲染前分配锚点用于滚动
            if should_scroll {
                let anchor = ui.allocate_response(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.scroll_to_rect(anchor.rect, Some(egui::Align::TOP));
            }

            if is_active {
                let resp = ui.add(
                    egui::TextEdit::multiline(&mut self.editing_text)
                        .font(edit_font.clone())
                        .desired_width(f32::INFINITY)
                        .frame(false),
                );
                if resp.changed() {
                    self.commit_edit(&blocks);
                    content_changed = true;
                }
                if resp.lost_focus()
                    && !ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    self.commit_edit(&blocks);
                    new_active = None;
                }
            } else {
                let resp = ui.scope(|ui| {
                    editor::render_rich_block(ui, block, &self.theme);
                }).response;

                if resp.interact(egui::Sense::click()).clicked() {
                    if let Some(prev) = self.active_block {
                        if prev < blocks.len() {
                            self.commit_edit(&blocks);
                        }
                    }
                    new_active = Some(idx);
                    self.editing_text = block.source.clone();
                }
            }
        }

        if new_active != self.active_block {
            self.active_block = new_active;
        }
        if content_changed {
            self.outline_items = outline::extract_outline(self.document.content());
        }
    }

    /// 提交当前编辑块的修改到文档缓冲区
    fn commit_edit(&mut self, blocks: &[TextBlock]) {
        if let Some(idx) = self.active_block {
            if idx < blocks.len() {
                let block = &blocks[idx];
                let content = self.document.content().to_string();
                let lines: Vec<&str> = content.lines().collect();
                let mut new_lines: Vec<String> = Vec::new();
                for (i, line) in lines.iter().enumerate() {
                    if i < block.start_line || i > block.end_line {
                        new_lines.push(line.to_string());
                    } else if i == block.start_line {
                        new_lines.push(self.editing_text.clone());
                    }
                }
                let new_content = new_lines.join("\n");
                *self.document.buffer.as_mut_string() = new_content;
                self.mark_modified();
            }
        }
    }

    /// 渲染原始编辑模式（纯 Markdown 文本编辑器）
    fn render_raw_editor(&mut self, ui: &mut egui::Ui) {
        let content = self.document.buffer.as_mut_string();
        let font_id = egui::FontId::monospace(self.theme.font.monospace_size);
        let text_color = self.ui_theme.text_color;
        ui.visuals_mut().override_text_color = Some(text_color);

        // raw 模式不支持按行滚动，消费掉避免残留
        self.scroll_to_line = None;

        let resp = ui.add(
            egui::TextEdit::multiline(content)
                .font(font_id)
                .desired_width(f32::INFINITY)
                .frame(false)
                .hint_text("输入 Markdown..."),
        );
        if resp.changed() {
            self.mark_modified();
            self.outline_items = outline::extract_outline(self.document.content());
        }
    }
}
