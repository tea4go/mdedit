use std::path::{Path, PathBuf};

use eframe::egui;
use crate::config::AppConfig;
use crate::css_loader;
use crate::document::Document;
use crate::editor::{self, TextBlock};
use crate::outline::{self, OutlineItem};
use crate::theme::{Theme, UiTheme, ExtraTheme, CodeStyle, HeadingStyle, QuoteStyle, TableStyle, LinkStyle};

const CSS_THEME_DIR: &str =
    r"C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme";

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EditMode {
    Raw,
    Preview,
}

struct NoteFontConfig {
    family: Vec<String>,
    size: f32,
    bold: bool,
}

struct ConfigFontConfig {
    family: Vec<String>,
    size: f32,
    bold: bool,
}

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

fn read_preferences() -> Option<serde_json::Value> {
    let path = PathBuf::from(r"C:\Users\tony\AppData\Roaming\WhaleTerm\preferences.json");
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

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
        },
        ..base_theme
    })
}

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
        menu_bg: get("appBgColor"),
        menu_text: get("appHeaderTextColor"),
        sidebar_bg: get("appSiderBarBgColor"),
        sidebar_text: get("appSideTextColor"),
        sidebar_hover_bg: get("appSideHoverBgColor"),
        sidebar_active_bg: get("appLeftListBgColorActive"),
        sidebar_active_text: get("appLeftListTextColorActive"),
        content_bg: get("appContentNoteBgColor"),
        border: get("borderColor"),
        divider: get("appDividerColor"),
        text_color: get("textColor"),
        text_active_color: get("textActiveColor"),
        split_color: get("appSplitColor"),
        sidebar_active_text_color: get("appSideTextActiveColor"),
        input_bg: get("inputContentBgColor"),
        input_border: get("inputContentBorderColor"),
        drop_down_text: get("dropDownColor"),
        drop_down_bg: get("dropDownBgColor"),
        drop_down_active_text: get("dropDownActiveColor"),
        drop_down_active_bg: get("dropDownActiveBgColor"),
    }
}

fn load_extra_theme(mode: ThemeMode) -> ExtraTheme {
    match mode {
        ThemeMode::Light | ThemeMode::Auto => ExtraTheme {
            outline_hover_color: egui::Color32::from_rgb(0x42, 0x85, 0xF4),
            note_toolbar_header_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
            active_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
        },
        ThemeMode::Dark => ExtraTheme {
            outline_hover_color: egui::Color32::WHITE,
            note_toolbar_header_bg: egui::Color32::from_rgb(0x02, 0x38, 0x48),
            active_color: egui::Color32::WHITE,
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
            ThemeMode::Light | ThemeMode::Auto => UiTheme {
                menu_bg: egui::Color32::from_rgb(0xF5, 0xF5, 0xF5),
                menu_text: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                sidebar_bg: egui::Color32::WHITE,
                sidebar_text: egui::Color32::from_rgb(0x66, 0x66, 0x66),
                sidebar_hover_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                sidebar_active_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
                sidebar_active_text: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                content_bg: egui::Color32::WHITE,
                border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                divider: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                text_color: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                text_active_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                split_color: egui::Color32::from_rgb(0xE0, 0xE0, 0xE0),
                sidebar_active_text_color: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                input_bg: egui::Color32::WHITE,
                input_border: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                drop_down_text: egui::Color32::from_rgb(0x33, 0x33, 0x33),
                drop_down_bg: egui::Color32::WHITE,
                drop_down_active_text: egui::Color32::from_rgb(0x00, 0x7A, 0xCC),
                drop_down_active_bg: egui::Color32::from_rgb(0xE3, 0xF2, 0xFD),
            },
            ThemeMode::Dark => UiTheme {
                menu_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                menu_text: egui::Color32::WHITE,
                sidebar_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                sidebar_hover_bg: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_active_bg: egui::Color32::from_rgb(0x09, 0x47, 0x71),
                sidebar_active_text: egui::Color32::WHITE,
                content_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                divider: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                text_color: egui::Color32::WHITE,
                text_active_color: egui::Color32::WHITE,
                split_color: egui::Color32::from_rgb(0x07, 0x36, 0x42),
                sidebar_active_text_color: egui::Color32::WHITE,
                input_bg: egui::Color32::from_rgb(0x00, 0x22, 0x2B),
                input_border: egui::Color32::from_rgb(0x1A, 0x77, 0x78),
                drop_down_text: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
                drop_down_bg: egui::Color32::from_rgb(0x00, 0x2B, 0x36),
                drop_down_active_text: egui::Color32::WHITE,
                drop_down_active_bg: egui::Color32::from_rgb(0x09, 0x49, 0x5E),
            },
        }
    }
}

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
}

impl MdEditApp {
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
        }
    }

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

    fn effective_mode(&self) -> (ThemeMode, ThemeMode) {
        match self.theme_mode {
            ThemeMode::Auto => {
                let effective = if Self::is_system_dark() { ThemeMode::Dark } else { ThemeMode::Light };
                (ThemeMode::Auto, effective)
            },
            other => (other, other),
        }
    }

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

    fn update_outline(&mut self) {
        self.outline_items = outline::extract_outline(self.document.content());
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        let shift = ctx.input(|i| i.modifiers.shift);
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
        }
    }

    fn new_file(&mut self) {
        self.document = Document::new();
        self.outline_items.clear();
    }

    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .pick_file()
        {
            if let Ok(content) = std::fs::read_to_string(&path) {
                self.document = Document::from_file(path, content);
                self.update_outline();
            }
        }
    }

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
                    if ui.checkbox(&mut self.show_outline, "大纲面板").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
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

        if self.show_outline {
            let font_size = self.theme.font.base_size;
            let sb = self.ui_theme.sidebar_bg;
            let st = self.ui_theme.sidebar_text;
            let oh = self.extra_theme.outline_hover_color;
            egui::SidePanel::left("outline_panel")
                .default_width(200.0)
                .frame(egui::Frame::none().fill(sb))
                .show(ctx, |ui| {
                    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = st;
                    ui.visuals_mut().widgets.inactive.fg_stroke.color = st;
                    ui.visuals_mut().widgets.hovered.bg_fill = oh;
                    ui.visuals_mut().widgets.hovered.fg_stroke.color = st;
                    ui.visuals_mut().widgets.active.fg_stroke.color = st;
                    ui.label(egui::RichText::new("大纲").size(font_size * 1.2).strong().color(st));
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                        for item in &self.outline_items {
                            let indent = (item.level as f32 - 1.0) * 12.0;
                            ui.horizontal(|ui| {
                                ui.add_space(indent);
                                let btn = ui.add(egui::Button::new(
                                    egui::RichText::new(&item.title).size(font_size).color(st)
                                ).fill(egui::Color32::TRANSPARENT));
                                if btn.clicked() {
                                    self.scroll_to_line = Some(item.line);
                                }
                            });
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
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        if self.edit_mode == EditMode::Raw {
            self.render_raw_editor(ui);
            return;
        }

        let content_snapshot = self.document.content().to_string();
        let blocks = editor::split_blocks(&content_snapshot);

        if let Some(target_line) = self.scroll_to_line.take() {
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
                self.document.modified = true;
                self.outline_items = outline::extract_outline(self.document.content());
            }
            return;
        }

        let mut new_active: Option<usize> = self.active_block;
        let mut content_changed = false;

        for (idx, block) in blocks.iter().enumerate() {
            let is_active = self.active_block == Some(idx);

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
                self.document.modified = true;
            }
        }
    }

    fn render_raw_editor(&mut self, ui: &mut egui::Ui) {
        let content = self.document.buffer.as_mut_string();
        let font_id = egui::FontId::monospace(self.theme.font.monospace_size);
        let text_color = self.ui_theme.text_color;
        ui.visuals_mut().override_text_color = Some(text_color);
        let resp = ui.add(
            egui::TextEdit::multiline(content)
                .font(font_id)
                .desired_width(f32::INFINITY)
                .frame(false)
                .hint_text("输入 Markdown..."),
        );
        if resp.changed() {
            self.document.modified = true;
            self.outline_items = outline::extract_outline(self.document.content());
        }
    }
}
