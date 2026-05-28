use eframe::egui;
use crate::document::Document;
use crate::editor::Editor;
use crate::outline::{self, OutlineItem};
use crate::renderer;
use crate::theme::Theme;

pub struct MdEditApp {
    document: Document,
    editor: Editor,
    outline_items: Vec<OutlineItem>,
    show_outline: bool,
    theme: Theme,
    scroll_to_line: Option<usize>,
}

impl MdEditApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            document: Document::new(),
            editor: Editor::new(),
            outline_items: Vec::new(),
            show_outline: true,
            theme: Theme::default(),
            scroll_to_line: None,
        }
    }

    fn update_outline(&mut self) {
        self.outline_items = outline::extract_outline(self.document.content());
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        if ctrl {
            if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                self.save_file();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::O)) {
                self.open_file();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                self.new_file();
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
                });
                ui.menu_button("视图", |ui| {
                    if ui.checkbox(&mut self.show_outline, "大纲面板").clicked() {
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                let content = self.document.buffer.as_mut_string();
                let response = ui.add(
                    egui::TextEdit::multiline(content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .frame(false),
                );
                if response.changed() {
                    self.document.modified = true;
                    self.outline_items =
                        outline::extract_outline(self.document.content());
                }
            });
        });
    }
}
