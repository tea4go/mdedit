use std::path::{Path, PathBuf};

use eframe::egui;
use crate::config::AppConfig;
use crate::css_loader;
use crate::document::Document;
use crate::editor::{self, TextBlock};
use crate::outline::{self, OutlineItem};
use crate::theme::Theme;

const CSS_THEME_DIR: &str =
    r"C:\Users\tony\AppData\Roaming\WhaleTerm\mynotes\files\markdown-theme";

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub struct MdEditApp {
    document: Document,
    outline_items: Vec<OutlineItem>,
    show_outline: bool,
    theme: Theme,
    theme_mode: ThemeMode,
    scroll_to_line: Option<usize>,
    active_block: Option<usize>,
    editing_text: String,
    last_window_pos: Option<(f32, f32)>,
    last_window_size: Option<(f32, f32)>,
    last_maximized: bool,
    first_frame: bool,
}

impl MdEditApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        initial_file: Option<(PathBuf, String)>,
        cfg: &AppConfig,
    ) -> Self {
        Self::configure_fonts(&cc.egui_ctx);

        let (document, outline_items) = if let Some((path, content)) = initial_file {
            let document = Document::from_file(path, content);
            let outline_items = outline::extract_outline(document.content());
            (document, outline_items)
        } else {
            (Document::new(), Vec::new())
        };

        let theme_mode = if cfg.theme == "dark" {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        let theme = Self::load_css_theme(theme_mode);

        Self {
            document,
            outline_items,
            show_outline: true,
            theme,
            theme_mode,
            scroll_to_line: None,
            active_block: None,
            editing_text: String::new(),
            last_window_pos: None,
            last_window_size: None,
            last_maximized: false,
            first_frame: true,
        }
    }

    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let font_paths = if cfg!(target_os = "windows") {
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

        for path in font_paths {
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

        ctx.set_fonts(fonts);
    }

    fn load_css_theme(mode: ThemeMode) -> Theme {
        let filename = match mode {
            ThemeMode::Light => "light.css",
            ThemeMode::Dark => "dark.css",
        };
        let path = Path::new(CSS_THEME_DIR).join(filename);
        css_loader::load_theme_from_css(&path).unwrap_or_else(|| {
            match mode {
                ThemeMode::Light => Theme::light(),
                ThemeMode::Dark => Theme::dark(),
            }
        })
    }

    fn switch_theme(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        self.theme = Self::load_css_theme(mode);
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
        // 首帧检查：如果窗口不在可见区域，移到主屏幕
        if self.first_frame {
            self.first_frame = false;
            let need_reposition = ctx.input(|i| {
                if let Some(outer) = i.viewport().outer_rect {
                    let monitor_w = i.viewport().monitor_size
                        .map(|s| s.x).unwrap_or(1920.0);
                    let monitor_h = i.viewport().monitor_size
                        .map(|s| s.y).unwrap_or(1080.0);
                    // 窗口完全在可见区域外
                    outer.min.x >= monitor_w || outer.min.y >= monitor_h
                        || outer.max.x <= 0.0 || outer.max.y <= 0.0
                } else {
                    false
                }
            });
            if need_reposition {
                ctx.send_viewport_cmd(
                    egui::ViewportCommand::OuterPosition(egui::pos2(100.0, 100.0))
                );
            }
        }

        self.handle_shortcuts(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
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
                    if self.theme_mode != prev_mode {
                        self.switch_theme(self.theme_mode);
                        ui.close_menu();
                    }
                });
            });
        });

        if self.show_outline {
            egui::SidePanel::left("outline_panel")
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.heading("大纲");
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for item in &self.outline_items {
                            let indent = (item.level as f32 - 1.0) * 12.0;
                            ui.horizontal(|ui| {
                                ui.add_space(indent);
                                let btn = ui.button(&item.title);
                                if btn.clicked() {
                                    self.scroll_to_line = Some(item.line);
                                }
                            });
                        }
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("editor_scroll")
                .show(ui, |ui| {
                    self.render_editor(ui);
                });
        });

        // 追踪窗口状态用于退出时保存
        ctx.input(|i| {
            if let Some(rect) = i.viewport().inner_rect {
                self.last_window_size = Some((rect.width(), rect.height()));
            }
            if let Some(rect) = i.viewport().outer_rect {
                self.last_window_pos = Some((rect.min.x, rect.min.y));
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
            },
        };
        cfg.save();
    }
}

impl MdEditApp {
    fn render_editor(&mut self, ui: &mut egui::Ui) {
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

        if blocks.is_empty() {
            let content = self.document.buffer.as_mut_string();
            let resp = ui.add(
                egui::TextEdit::multiline(content)
                    .font(egui::TextStyle::Monospace)
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
                        .font(egui::TextStyle::Monospace)
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
}
